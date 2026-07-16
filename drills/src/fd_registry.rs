//! drill:fd_registry —— 填 register / unregister / get / get_mut。
//!
//! 已給:Token 打包/解包、結構、new、len/is_empty。
//! 要填:四個核心方法。核心不變量一句話:**`gens[fd]` 只在成功 unregister
//! 時 +1;token 的 gen 對不上就當作查無此人**——這一個欄位就是在擋
//! 「fd 回收後,舊 event dispatch 到新住戶」的經典 bug。
//! 設計取捨見 `docs/fd_registry.md`。

/// 進出 `epoll_event.data`(u64)的身份憑證:`(gen << 32) | fd`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token(u64);

impl Token {
    pub fn to_raw(self) -> u64 {
        self.0
    }

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
    /// index = fd。
    slots: Vec<Option<T>>,
    /// slot 世代:unregister 成功時 +1。
    gens: Vec<u32>,
    /// 活著的登記數。
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

    /// spec:登記 fd,回 Token。
    /// 1. assert fd <= u32::MAX as usize(塞得進低 32 bits)。
    /// 2. fd 超出 slots 長度 → resize_with(fd+1, None)、gens 補 0。
    /// 3. assert slot 是 None(活著的 fd 不重號;撞到 = caller bug,
    ///    panic 訊息含 "already registered")。
    /// 4. 放入 value、len += 1,回 Token((gens[fd] << 32) | fd)。
    pub fn register(&mut self, fd: usize, value: T) -> Token {
        todo!("spec: assert 範圍與空 slot; 增長; 放入; 打包 (gen<<32)|fd")
    }

    /// spec:移除並取回。
    /// gen 對不上(或 fd 越界)→ None,什麼都不動。
    /// 取出 Some 時:gens[fd] wrapping_add(1)、len -= 1。
    /// (gen 只在**成功移除**時 bump——stale unregister 不能動現任住戶。)
    pub fn unregister(&mut self, token: Token) -> Option<T> {
        todo!("spec: 驗 gen; take; 成功才 bump gen 與 len")
    }

    /// spec:O(1) 查表。gen 對不上 → None;gen 相符但 slot 為 None
    /// (從未登記的 slot、偽造 token)也要安全回 None。
    pub fn get(&self, token: Token) -> Option<&T> {
        todo!("spec: gens.get(fd) 比對後 as_ref")
    }

    /// spec:get 的可變版。
    pub fn get_mut(&mut self, token: Token) -> Option<&mut T> {
        todo!("spec: 同 get,as_mut")
    }

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

    /// 核心 boundary:fd 重用——舊 token 必死、新 token 必活。
    /// 先在紙上 trace:register(5) 時 gens[5] 是多少?unregister 後呢?
    #[test]
    #[ignore = "填完 register/unregister/get 後移除"]
    fn fd_reuse_stale_token_dies() {
        let mut r = FdRegistry::new();
        let t1 = r.register(5, "A");
        assert_eq!(r.unregister(t1), Some("A"));
        let t2 = r.register(5, "B");
        assert_eq!(r.get(t1), None);
        assert_eq!(r.get(t2), Some(&"B"));
        assert_eq!(r.unregister(t1), None);
        assert_eq!(r.len(), 1);
    }

    /// token 經 u64 往返(模擬 epoll_event.data)後仍解析。
    #[test]
    #[ignore = "填完 register/get 後移除"]
    fn token_u64_roundtrip() {
        let mut r = FdRegistry::new();
        let t = r.register(3, 30);
        assert_eq!(r.get(Token::from_raw(t.to_raw())), Some(&30));
    }

    /// boundary:空表偽造 token、gen=0 但從未登記的 slot——安全回 None。
    #[test]
    #[ignore = "填完 get/unregister 後移除"]
    fn forged_tokens_are_safe() {
        let mut r: FdRegistry<i32> = FdRegistry::new();
        assert_eq!(r.get(Token::from_raw(5)), None);
        assert_eq!(r.unregister(Token::from_raw(5)), None);
        r.register(10, 1);
        assert_eq!(r.get(Token::from_raw(3)), None);
    }

    /// double-register 活著的 fd 要 panic。
    #[test]
    #[ignore = "填完 register 後移除"]
    #[should_panic(expected = "already registered")]
    fn register_occupied_panics() {
        let mut r = FdRegistry::new();
        r.register(5, 1);
        r.register(5, 2);
    }
}
