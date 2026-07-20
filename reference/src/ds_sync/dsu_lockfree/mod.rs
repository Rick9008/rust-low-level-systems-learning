//! # dsu_lockfree —— 無鎖 union-find(CAS parent + 隨機 priority + path halving)
//!
//! ## [Clarify]
//! 解決:多執行緒共享的連通性維護,union/find/connected 全部 `&self` 無鎖。
//! 典型場景:並行圖演算法(平行 Kruskal / 連通元件)、叢集 membership 合併。
//! Constraints:元素 `0..n` 固定(不支援並發擴容);只合不拆;
//! `connected == false` 是快照語意(見 core 的 doc)。
//!
//! ## [Abstract]
//! 與單執行緒版(`crate::ds::dsu`)同介面精神:索引世界、外部映射自理。
//! 差異只在簽名:`&mut self` → `&self`——並發化的本體就是這一步。
//!
//! ## [Iterate]
//! 演進線:`Mutex<Dsu>`(正確、但每個 find 都串行)→ 想無鎖化 union-by-rank
//! → 卡死:parent + rank 兩處寫無法單 CAS → **丟 rank,換固定隨機 priority**
//! (Jayanti–Tarjan):link 塌縮成單字 CAS,expected value 就是「root 的定義」。
//! path compression(兩趟)→ path halving(單趟、每步一個可放棄的 CAS hint)。
//!
//! ## [Trade-offs]
//! - 免 generation tag:parent 單調向根、root 資格一去不返 ⇒ 舊 expected
//!   永久失效,ABA 無從發生(對照 arena_lockfree 必須掛 gen)。
//! - rank → 隨機 priority:攤銷界從 α(n) 弱化為期望 O(log n) 樹高
//!   (halving 疊加後實務近常數);買到單 CAS link。
//! - ordering 用 Acquire/Release:就「樹結構正確」而言 Relaxed 也夠
//!   (CAS 的 per-location 原子性就足以維持不變量),但 caller 常拿
//!   connected 的結果去守外部資料,保守給 hb 邊,面試先聲明再降級。
//! - components 是 Relaxed 計數:讀值=快照,不能當同步旗標。
//!
//! ## [Dry-Run]
//! 單執行緒:trace 見 `union_find_basic_trace`;並發:煙霧測試
//! (n-1 次成功合併的全域守恆)+ **loom 窮舉**(`tests/loom_dsu.rs`):
//! 同 pair 雙 union 恰一人贏、鏈式 union 收斂、find halving 與 union 交錯。
//!
//! Production 對照:平行圖處理框架(如 GBBS)的 concurrent union-find。

mod core_impl;

pub use core_impl::DsuLockFree;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// [Dry-Run] 手 trace(n=4,方向由 priority 決定、斷言與方向無關):
    ///   union(0,1):find(0)——load parent[0]==0 ⇒ 根,回 0;find(1) 同理回 1;
    ///   priority 定向後 CAS(parent[lo]: lo→hi) 成功,components 4→3。
    ///   再 union(0,1):find 兩邊回到同一個根(輸家那格已指向贏家),
    ///   rx==ry ⇒ false(冪等)。components 不動。
    #[test]
    fn union_find_basic_trace() {
        let d = DsuLockFree::new(4);
        assert_eq!(d.components(), 4);
        assert!(d.union(0, 1));
        assert_eq!(d.components(), 3);
        assert!(!d.union(0, 1)); // 冪等
        assert_eq!(d.components(), 3);
        assert!(d.connected(0, 1));
        assert!(!d.connected(0, 2));
        assert_eq!(d.find(0), d.find(1));
    }

    /// boundary:n=1 退化與自反操作。
    #[test]
    fn boundary_single_and_self_union() {
        let d = DsuLockFree::new(1);
        assert!(!d.union(0, 0));
        assert!(d.connected(0, 0));
        assert_eq!(d.components(), 1);
    }

    /// 鏈狀合併後全部同根,components 精確遞減。
    #[test]
    fn chain_unions_all_connected() {
        let d = DsuLockFree::new(8);
        for i in 0..7 {
            assert!(d.union(i, i + 1));
        }
        assert_eq!(d.components(), 1);
        for i in 0..8 {
            assert!(d.connected(0, i));
            assert_eq!(d.find(i), d.find(0));
        }
    }

    /// 並發煙霧測試:8 執行緒搶著做同一批 union(i, i+1)。
    /// 全域守恆不變量:每條邊只會被「真的合併」一次 ⇒
    /// 所有執行緒回傳 true 的總數恰為 n-1(components 也收斂到 1)。
    /// (窮舉版證明在 tests/loom_dsu.rs。)
    #[test]
    fn concurrent_unions_merge_count_conserved() {
        const N: u32 = 512;
        let d = Arc::new(DsuLockFree::new(N as usize));
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let d = Arc::clone(&d);
                thread::spawn(move || {
                    let mut wins = 0usize;
                    for i in 0..N - 1 {
                        if d.union(i, i + 1) {
                            wins += 1;
                        }
                    }
                    wins
                })
            })
            .collect();
        let total: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
        assert_eq!(total, (N - 1) as usize);
        assert_eq!(d.components(), 1);
        for i in 0..N {
            assert!(d.connected(0, i));
        }
    }
}
