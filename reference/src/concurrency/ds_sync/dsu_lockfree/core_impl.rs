//! lock-free DSU(union-find)核心演算法:CAS parent + 隨機 priority linking + path halving。
//!
//! 與 spsc / arena 相同的雙重 include 架構:lib 走 std、`tests/loom_dsu.rs` 走 loom。
//! 只准用 `crate::sync_shim` 的同步原語。

use crate::sync_shim as sync;
use sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// splitmix64 finalizer:把 index 打散成固定的偽隨機值。
/// priority 是 index 的純函式,不佔任何空間,lib / loom 兩邊天然一致。
fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// linking 的全序:(hash, index) 字典序。hash 相撞就比 index,
/// 全序 ⇒ link 方向固定 ⇒ 不可能串成環。
fn priority(i: u32) -> (u64, u32) {
    (splitmix64(u64::from(i)), i)
}

/// 無鎖 union-find。所有操作 `&self`(對照單執行緒版 `crate::ds::dsu` 的 `&mut self`)。
pub struct DsuLockFree {
    /// parent[i] == i ⇔ i 是根。parent 只會「往根的方向」單調前進、
    /// 且 root 資格一旦失去永不復得——這條單調性是整個演算法
    /// 免 generation tag 的原因(對照 arena_lockfree:head 可以指回
    /// 舊索引,必須掛 gen 防 ABA;這裡舊 expected value 失效即永久失效)。
    parent: Box<[AtomicU32]>,
    /// 集合數,成功 link 時 -1。讀值是快照:拿到手時可能已有新 union 完成。
    components: AtomicUsize,
}

impl DsuLockFree {
    /// n 個 singleton。O(n)。索引空間 u32。
    pub fn new(n: usize) -> Self {
        assert!(
            u32::try_from(n).is_ok(),
            "index space is u32 (match the arena convention)"
        );
        Self {
            parent: (0..n as u32).map(AtomicU32::new).collect(),
            components: AtomicUsize::new(n),
        }
    }

    pub fn len(&self) -> usize {
        self.parent.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parent.is_empty()
    }

    /// 目前集合數(快照語意)。O(1)。
    pub fn components(&self) -> usize {
        self.components.load(Ordering::Relaxed)
    }

    /// 查代表(根)。期望 O(log n) 單次(隨機 link 的樹高界),
    /// halving 疊加後攤銷接近 α;競爭下可能多走幾步。
    ///
    /// path halving:讓 x 直接改指祖父(路徑壓一半)。CAS 失敗**不重試**——
    /// 這個寫入只是效能 hint:失敗代表別人已把 parent[x] 推得更遠
    /// (parent 單調向根),放棄不影響任何正確性。
    /// (對照 `union` 的 link CAS:那個寫入承載連通性,失敗必須整段重來。)
    pub fn find(&self, mut x: u32) -> u32 {
        loop {
            let p = self.parent[x as usize].load(Ordering::Acquire);
            if p == x {
                return x;
            }
            let gp = self.parent[p as usize].load(Ordering::Acquire);
            if gp == p {
                // 這次 load 目擊「p 是根」——本次 find 的線性化見證點。
                return p;
            }
            // halving hint:x → 祖父。gp 必在 x 的祖先鏈上(祖先只會更靠近根),
            // 所以就算 CAS 失敗,直接從 gp 繼續爬也正確。
            let _ = self.parent[x as usize].compare_exchange(
                p,
                gp,
                Ordering::Release,
                Ordering::Relaxed,
            );
            x = gp;
        }
    }

    /// 合併 x, y 所在集合;回傳是否真的發生合併(false = 本來就同集合)。
    /// lock-free 非 wait-free:link CAS 失敗 ⇔ 別人成功了 → 重新 find 重試。
    pub fn union(&self, x: u32, y: u32) -> bool {
        loop {
            let rx = self.find(x);
            let ry = self.find(y);
            if rx == ry {
                return false; // 連通性單調:一旦同集合永遠同集合,false 安全
            }
            // 固定隨機 priority 定向:小的掛到大的下面。這是丟棄 union-by-rank
            // 換來的原子性:rank 要「parent + rank 兩處寫入」一起原子(單 CAS
            // 做不到);priority 是 index 的常數函式,link 只剩一個字的寫入。
            let (lo, hi) = if priority(rx) < priority(ry) {
                (rx, ry)
            } else {
                (ry, rx)
            };
            // expected value = lo 自己:「parent[lo] == lo ⇔ lo 是根」的定義
            // 同時就是 CAS 的 guard——find 之後 lo 若被別人 link 走,這裡自動失敗。
            // 成功 Release:與 find/connected 的 Acquire load 建立 happens-before,
            // 讓「連通了」這個事實可以拿去守外部資料。
            if self.parent[lo as usize]
                .compare_exchange(lo, hi, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                // Relaxed 夠:components 是統計計數,不當同步旗標用。
                self.components.fetch_sub(1, Ordering::Relaxed);
                return true;
            }
        }
    }

    /// 同集合查詢。線性化論證(Jayanti–Tarjan 的複查技巧):
    /// 1. find(x)=rx 之後複查 parent[rx]==rx 仍成立——「rx 停止當根」是
    ///    單向事件,所以從 find(x) 到複查的**整段期間** rx 都是 x 的根。
    /// 2. find(y)=ry 的見證點落在這段期間內:該瞬間 y 的根是 ry ≠ rx
    ///    ⇒ 那一刻兩者確實不連通,false 可線性化。
    ///
    /// 複查失敗(rx 已被 link 走)= 證據過期 → 重來。
    /// 注意語意弱化:false 只代表「曾有一瞬間不連通」,回傳途中可能已被 union。
    pub fn connected(&self, x: u32, y: u32) -> bool {
        loop {
            let rx = self.find(x);
            let ry = self.find(y);
            if rx == ry {
                return true;
            }
            if self.parent[rx as usize].load(Ordering::Acquire) == rx {
                return false;
            }
        }
    }
}
