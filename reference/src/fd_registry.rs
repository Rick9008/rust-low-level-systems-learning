//! # fd_registry —— event loop 的 interest table:generational slot map
//!
//! ## [Clarify]
//! 解決:event loop 手上只有 kernel 還回來的一個整數(epoll 是
//! `epoll_event.data` 的 u64),要 O(1) 找回「這個 fd 對應的 handler /
//! 連線狀態」。fd 的特性:小、密集、**會回收**——`close(5)` 之後,
//! 下一次 accept 很可能又拿到 5。經典 bug:fd 5 關掉、號碼被新連線佔走,
//! event queue 裡還躺著舊 5 的 event → dispatch 到新住戶。只在高 churn
//! 下現形,單元測試抓不到,production 半夜爆。
//! Constraints:std-only、單執行緒(event loop 內部結構);fd < 2^32。
//!
//! ## [Abstract]
//! 泛化為「**caller 指定 index 的 generational slot map**」:
//! - 與 slab 的差別:多 generation——slab 的 key 重用後無法識別 stale key;
//! - 與 slotmap 的差別:key(= fd)由 caller 給、不是容器發的——kernel
//!   已經替你發好號了。
//!
//! value 泛型 `T`:event loop 放 handler / 連線狀態,mini-runtime 放 `Waker`。
//!
//! ## [Iterate]
//! 1. naive:`HashMap<u64, T>`——O(1) 但 hash + probe 常數大、cache 差;
//!    且 fd 重用 bug **完全沒解**(舊 key 查得到新住戶)。
//! 2. `Vec<Option<T>>` by fd——一次 array load,fd 密集所以空間不浪費;
//!    stale 問題仍在。
//! 3. 加上 generation:`gens[fd]` 在 unregister 時遞增;token 打包
//!    `(gen << 32) | fd`——stale token 的 gen 對不上,[`FdRegistry::get`]
//!    自然回 `None`。kernel 的 u64 座位恰好放得下整個 token。
//!
//! ## [Trade-offs]
//! - `Vec<Option<T>>` vs `HashMap`:array load ~1ns vs hash+probe 數十 ns
//!   (見 docs/cost-model.md);代價是空間 O(max_fd) 而非 O(live)——
//!   fd 密集(RLIMIT_NOFILE 級別)所以可接受。
//! - [`FdRegistry::register`] 撞到已佔用 slot 直接 panic:kernel 不會重發
//!   活著的 fd,double-register 是 caller bug——靜默覆蓋只是把 bug 往後推。
//! - generation 是 u32、wrapping:同一 fd 經歷 2^32 次 register/unregister
//!   後理論上 stale token 可能 false-match——誠實邊界。與
//!   [`arena_lockfree`](crate::arena_lockfree) 的 generation 同款取捨:
//!   那邊防 CAS 的 ABA,這邊防 stale dispatch,同一個
//!   「index 會回收,持有者要驗明正身」問題。
//! - 所有操作 O(1)(register 觸發 Vec 增長時攤銷 O(1))。
//!
//! ## [Dry-Run]
//! 見 [`tests`] 的 `boundary_fd_reuse_stale_token`——那個半夜爆的 bug
//! 的可執行版本,逐步 trace 在測試的 doc comment 裡。

/// 進出 `epoll_event.data`(u64)的身份憑證:`(gen << 32) | fd`。
///
/// 打包讓 kernel 免費幫你攜帶「fd + 這是第幾代住戶」的完整身份;
/// readiness event 回來時,一個 u64 就足以判斷事件是否過期。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token(u64);

impl Token {
    /// 塞進 `epoll_event.data` 用。
    pub fn to_raw(self) -> u64 {
        self.0
    }

    /// 從 `epoll_event.data` 還原。偽造 / 過期的 raw 值是安全的:
    /// 查表時 gen 或 slot 對不上,一律回 `None`。
    pub fn from_raw(raw: u64) -> Self {
        Token(raw)
    }

    fn fd_index(self) -> usize {
        (self.0 & 0xFFFF_FFFF) as usize
    }

    fn generation(self) -> u32 {
        (self.0 >> 32) as u32
    }
}

/// caller 指定 index(= fd)的 generational slot map。
pub struct FdRegistry<T> {
    /// index = fd。fd 密集,直接 index、一次 array load。
    slots: Vec<Option<T>>,
    /// slot 的世代:unregister 時 +1,舊 token 就再也對不上。
    gens: Vec<u32>,
    /// 活著的登記數(≠ slots.len())。
    len: usize,
}

impl<T> FdRegistry<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            gens: Vec::new(),
            len: 0,
        }
    }

    /// 登記 fd 的狀態,回傳可塞進 u64 的 [`Token`]。O(1) 攤銷。
    ///
    /// # Panics
    /// - fd 已有活著的登記:kernel 不會重發活著的 fd,double-register
    ///   是 caller bug。
    /// - fd ≥ 2^32:塞不進 token 的低 32 bits(真實 fd 遠小於此)。
    pub fn register(&mut self, fd: usize, value: T) -> Token {
        assert!(fd <= u32::MAX as usize, "fd {fd} 超出 token 打包範圍(2^32)");
        if fd >= self.slots.len() {
            self.slots.resize_with(fd + 1, || None);
            self.gens.resize(fd + 1, 0);
        }
        assert!(
            self.slots[fd].is_none(),
            "fd {fd} already registered——活著的 fd 不會重號,這是 caller bug"
        );
        self.slots[fd] = Some(value);
        self.len += 1;
        Token(((self.gens[fd] as u64) << 32) | fd as u64)
    }

    /// 移除並取回。stale token(gen 對不上 / 從未登記)→ `None`,
    /// **不影響現任住戶**。成功移除才 bump generation。O(1)。
    pub fn unregister(&mut self, token: Token) -> Option<T> {
        let fd = token.fd_index();
        if self.gens.get(fd).copied() != Some(token.generation()) {
            return None;
        }
        let value = self.slots[fd].take();
        if value.is_some() {
            self.gens[fd] = self.gens[fd].wrapping_add(1);
            self.len -= 1;
        }
        value
    }

    /// O(1) 查表。stale token → `None`(過期事件自然被丟棄——這一行
    /// 就是整個模組存在的理由)。
    pub fn get(&self, token: Token) -> Option<&T> {
        let fd = token.fd_index();
        if self.gens.get(fd).copied() != Some(token.generation()) {
            return None;
        }
        self.slots[fd].as_ref()
    }

    /// [`FdRegistry::get`] 的可變版(dispatch 時 handler 通常要 `&mut`)。
    pub fn get_mut(&mut self, token: Token) -> Option<&mut T> {
        let fd = token.fd_index();
        if self.gens.get(fd).copied() != Some(token.generation()) {
            return None;
        }
        self.slots[fd].as_mut()
    }

    /// 活著的登記數。
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T> Default for FdRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [Dry-Run] fd 重用的 stale token trace——半夜爆的 bug 的可執行版本:
    /// - register(5, "A") → gens[5]=0,t1 = (0<<32)|5,len=1
    /// - unregister(t1)   → 取回 "A",gens[5]=1,len=0
    /// - register(5, "B") → kernel 把 5 發給新連線;t2 = (1<<32)|5,len=1
    /// - get(t1):gens[5]=1 ≠ t1.gen=0 → None(舊 event 不會打到 B)
    /// - get(t2):gen 相符 → Some("B")
    /// - unregister(t1):stale → None,len 仍 1(不會誤刪新住戶)
    #[test]
    fn boundary_fd_reuse_stale_token() {
        let mut r = FdRegistry::new();
        let t1 = r.register(5, "A");
        assert_eq!(r.unregister(t1), Some("A"));
        assert_eq!(r.len(), 0);

        let t2 = r.register(5, "B"); // 同一個 fd 號碼再登記(kernel 回收)
        assert_eq!(r.get(t1), None, "stale token 不准查到新住戶");
        assert_eq!(r.get(t2), Some(&"B"));
        assert_eq!(r.unregister(t1), None, "stale unregister 是 no-op");
        assert_eq!(r.len(), 1, "新住戶不受 stale 操作影響");
        assert_eq!(r.get(t2), Some(&"B"));
    }

    /// token 打包是文件化的 ABI:(gen << 32) | fd,經 u64 往返後仍解析。
    /// trace:fd=3 第一代 → raw = 3;unregister 後再登記 → raw = (1<<32)|3。
    #[test]
    fn token_packs_gen_and_fd_into_u64() {
        let mut r = FdRegistry::new();
        let t1 = r.register(3, 30);
        assert_eq!(t1.to_raw(), 3); // gen=0
        r.unregister(t1);
        let t2 = r.register(3, 31);
        assert_eq!(t2.to_raw(), (1u64 << 32) | 3); // gen=1

        // 模擬 epoll 往返:塞進 u64 座位、拿回來查表。
        let roundtrip = Token::from_raw(t2.to_raw());
        assert_eq!(r.get(roundtrip), Some(&31));
    }

    /// boundary:空 registry + 偽造 token——查不到、不 panic。
    /// 包含「gen 恰好等於預設 0、但 slot 從未登記」的路徑(slot None)。
    #[test]
    fn boundary_forged_token_on_empty_and_sparse() {
        let mut r: FdRegistry<i32> = FdRegistry::new();
        assert_eq!(r.get(Token::from_raw(5)), None); // 空表:fd 越界
        assert_eq!(r.unregister(Token::from_raw(5)), None);

        r.register(10, 1); // 增長到 11 個 slot;fd 3 從未登記
        assert_eq!(r.get(Token::from_raw(3)), None, "gen=0 相符但 slot 是空的");
        assert!(!r.is_empty());
    }

    /// fd 跳躍增長:register(0) 後直接 register(100)——中間 slot 全 None,
    /// len 只數活著的。
    #[test]
    fn sparse_growth_len_counts_live_only() {
        let mut r = FdRegistry::new();
        let t0 = r.register(0, "zero");
        let t100 = r.register(100, "hundred");
        assert_eq!(r.len(), 2);
        assert_eq!(r.get(t0), Some(&"zero"));
        assert_eq!(r.get(t100), Some(&"hundred"));
    }

    /// get_mut:dispatch 端拿 &mut 改狀態。
    #[test]
    fn get_mut_mutates_in_place() {
        let mut r = FdRegistry::new();
        let t = r.register(7, vec![1, 2]);
        r.get_mut(t).unwrap().push(3);
        assert_eq!(r.get(t), Some(&vec![1, 2, 3]));
    }

    /// double-register 活著的 fd = caller bug,大聲 panic。
    #[test]
    #[should_panic(expected = "already registered")]
    fn register_occupied_slot_panics() {
        let mut r = FdRegistry::new();
        r.register(5, 1);
        r.register(5, 2);
    }

    /// 規模 sanity:千 fd 高 churn——偶數位全部換代,舊 token 全滅、
    /// 新 token 全活,奇數位不受影響。
    #[test]
    fn thousand_fd_churn() {
        let mut r = FdRegistry::new();
        let gen0: Vec<Token> = (0..1000).map(|fd| r.register(fd, fd)).collect();
        for fd in (0..1000).step_by(2) {
            assert_eq!(r.unregister(gen0[fd]), Some(fd));
        }
        let gen1: Vec<Token> = (0..1000)
            .step_by(2)
            .map(|fd| r.register(fd, fd + 10_000))
            .collect();

        assert_eq!(r.len(), 1000);
        for fd in (0..1000).step_by(2) {
            assert_eq!(r.get(gen0[fd]), None, "換代後舊 token 必死");
        }
        for (i, fd) in (0..1000).step_by(2).enumerate() {
            assert_eq!(r.get(gen1[i]), Some(&(fd + 10_000)));
        }
        for fd in (1..1000).step_by(2) {
            assert_eq!(r.get(gen0[fd]), Some(&fd), "奇數位第一代仍有效");
        }
    }
}
