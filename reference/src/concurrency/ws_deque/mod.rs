//! # ws_deque —— Chase–Lev work-stealing deque(SeqCst fence 的教科書實戰位)
//!
//! [Clarify]
//! - 單 owner、多 stealer 的雙端佇列:owner 在 bottom 端 LIFO push/pop
//!   (剛產生的任務最熱,cache 局部性),stealers 在 top 端 FIFO steal
//!   (偷最舊的,與 owner 的熱端錯開)。tokio/rayon 的 per-worker run queue
//!   就是這一格(語意 A「競爭消費」的王者,見 lockfree 家族手冊)。
//! - 教學版兩個簡化:固定容量(滿了 Err;工業版 buffer 會長,舊 buffer 靠
//!   epoch 回收)、**值裝箱**(見 [Trade-offs]——這一刀把教科書版的
//!   官方資料競爭換成一次配置)。
//!
//! [Abstract]
//! - 三個共享變數:`bottom`(owner 單寫)、`top`(CAS 推進)、槽位陣列。
//!   99% 的操作無競爭:owner 在自己端 push/pop,stealer 偷另一端。
//!   唯一的戰場是**最後一件**:owner pop 與 stealer steal 指向同一槽
//!   ——用「CAS top 決鬥」裁決,誰贏誰拿走。
//! - **SB litmus 就在正中央**:owner pop 先降 bottom 再讀 top;steal 先讀
//!   top 再讀 bottom。兩邊都是「先寫自己、再讀對方」——教科書的
//!   store-buffering 形狀。沒有 SeqCst fence,雙方可以互相看不見對方的寫,
//!   同時認定「最後一件是我的」→ double-take。與 `signal_pipeline`
//!   掛牌握手是同一個 litmus、同一帖藥(fence(SeqCst) 對)。
//!
//! [Iterate]
//! - V0(壞):pop 不降 bottom 直接比大小再取——比較與取走非原子,
//!   stealer 在中間偷走 → double-take。「先降 bottom 預定、fence、再看 top」
//!   的順序就是在把決鬥窗口壓到只剩最後一件。
//! - V1(壞):fence 換成 Acquire/Release——SB 形狀下不夠(兩邊的 store
//!   都可能停在 store buffer),loom 直接打爆。SeqCst 在這裡不是保險,
//!   是正確性(整個 repo 第二個非它不可的位置,第一個是 signal_pipeline)。
//! - V2(本版):值裝箱讓「偷看」成為原子指標 load。教科書版 inline 值的
//!   「先讀、輸了再丟」在槽位跨圈重寫時是正式資料競爭(Lê et al. 承認,
//!   crossbeam 用 epoch 處理)——裝箱後 UB 消失,loom 驗得動。
//!
//! [Trade-offs]
//! - vs `mpmc_ring`:不是同一題——deque 的重點是 owner 端**零競爭**
//!   (無 CAS 的 push/pop 快路徑),把競爭全推給偷竊(理論上少見)。
//!   全局共享佇列(mpmc)每 op 都在搶;work-stealing 只有失衡時才搶。
//! - LIFO owner × FIFO steal 不是隨便選的:LIFO 吃 cache 熱度,
//!   FIFO 偷走的是最舊(最可能已冷、且常是最大顆的)任務——兩端的
//!   訪問模式天然錯開,決鬥只剩最後一件。
//! - 值裝箱:+每 push 一次配置(~20–50ns);−教科書版 UB。工業版
//!   (crossbeam-deque)inline 值 + epoch 才是終極型態——「怎麼安全地
//!   偷看一個可能正被覆寫的槽」正是 reclamation 問題的另一張臉。
//! - `Steal::Retry ≠ Empty`:輸了決鬥必須重試,當作空會漏工作
//!   (與 `mpsc_list` 的 Inconsistent 同哲學:把「不確定」交給 caller)。
//!
//! [Dry-Run]
//! - 邊界測試 [`tests::lifo_fifo_hand_trace`] 手走兩端語意;
//!   [`tests::last_item_duel_single_thread`] 走最後一件的規範化路徑。
//! - `tests/loom_ws_deque.rs` 窮舉:最後一件決鬥恰一人贏、
//!   push/steal 並發不丟不重、mid-stream drop 不洩漏。

mod core_impl;

pub use core_impl::{Owner, Steal, Stealer, WsDeque, deque};

#[cfg(test)]
mod tests {
    use super::{Steal, deque};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    /// boundary:兩端語意手走。
    ///
    /// push 1,2,3(bottom: 0→3)。
    /// 1. owner pop → 3(LIFO:自己端拿最新)。
    /// 2. steal → 1(FIFO:偷走最舊)。
    /// 3. owner pop → 2(此刻它同時是最新與最舊——決鬥路徑,
    ///    單執行緒下 CAS 必贏)。
    /// 4. 兩端皆空:pop → None,steal → Empty。
    #[test]
    fn lifo_fifo_hand_trace() {
        let (mut owner, stealer) = deque(4);
        owner.push(1).unwrap();
        owner.push(2).unwrap();
        owner.push(3).unwrap();
        assert_eq!(owner.pop(), Some(3)); // 1
        assert_eq!(stealer.steal(), Steal::Item(1)); // 2
        assert_eq!(owner.pop(), Some(2)); // 3
        assert_eq!(owner.pop(), None); // 4
        assert_eq!(stealer.steal(), Steal::Empty);
    }

    /// boundary:最後一件的規範化——pop 完(不論輸贏)bottom 回到 top 之上,
    /// deque 可繼續用(不會殘留負 size)。
    #[test]
    fn last_item_duel_single_thread() {
        let (mut owner, stealer) = deque(2);
        owner.push(10).unwrap();
        assert_eq!(owner.pop(), Some(10)); // 單執行緒:決鬥必贏
        // 空了之後兩端還能正常工作
        owner.push(11).unwrap();
        assert_eq!(stealer.steal(), Steal::Item(11));
        assert_eq!(stealer.steal(), Steal::Empty);
        assert_eq!(owner.pop(), None);
    }

    /// boundary:滿(固定容量教學版)與排空後重用。
    #[test]
    fn full_then_reuse() {
        let (mut owner, _stealer) = deque(2);
        owner.push(1).unwrap();
        owner.push(2).unwrap();
        assert_eq!(owner.push(3), Err(3)); // 滿:歸還
        assert_eq!(owner.pop(), Some(2));
        owner.push(4).unwrap(); // 釋放一格後重用
        assert_eq!(owner.pop(), Some(4));
    }

    /// 每次 drop +1。
    #[derive(Debug)]
    struct DropSpy(Arc<AtomicUsize>);
    impl Drop for DropSpy {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// boundary:帶著未取走元素 drop——[top, bottom) 全數回收、剛好一次。
    #[test]
    fn drop_reclaims_remaining() {
        let n = Arc::new(AtomicUsize::new(0));
        {
            let (mut owner, stealer) = deque(4);
            for _ in 0..3 {
                owner.push(DropSpy(n.clone())).unwrap();
            }
            match stealer.steal() {
                Steal::Item(spy) => drop(spy), // 取走 1 → +1
                other => panic!("預期 Item,得到 {other:?}"),
            }
        } // 剩 2 由 Drop 回收
        assert_eq!(n.load(Ordering::Relaxed), 3);
    }

    /// 1 owner + 2 stealers 煙霧測試:owner 一邊 push 一邊 pop,
    /// stealers 狂偷;總帳不丟不重。
    /// (真正的證明在 tests/loom_ws_deque.rs。)
    #[test]
    fn owner_and_stealers_smoke() {
        const N: u64 = 30_000;
        let (mut owner, stealer) = deque(64);
        let taken = Arc::new(AtomicUsize::new(0));
        let sum = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let s = stealer.clone();
            let taken = Arc::clone(&taken);
            let sum = Arc::clone(&sum);
            handles.push(thread::spawn(move || {
                loop {
                    match s.steal() {
                        Steal::Item(v) => {
                            sum.fetch_add(v as usize, Ordering::Relaxed);
                            taken.fetch_add(1, Ordering::Relaxed);
                        }
                        Steal::Retry => {}
                        Steal::Empty => {
                            if taken.load(Ordering::Relaxed) as u64 == N {
                                break;
                            }
                            thread::yield_now();
                        }
                    }
                }
            }));
        }
        let mut i = 1u64; // 1..=N,總和公式可驗
        let mut pushed = 0u64;
        while pushed < N {
            match owner.push(i) {
                Ok(()) => {
                    pushed += 1;
                    i += 1;
                }
                Err(_) => {
                    // 滿:自己消化一件(LIFO 端),帳一樣記
                    if let Some(v) = owner.pop() {
                        sum.fetch_add(v as usize, Ordering::Relaxed);
                        taken.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }
        // 收尾:owner 把殘餘的自己清完
        while let Some(v) = owner.pop() {
            sum.fetch_add(v as usize, Ordering::Relaxed);
            taken.fetch_add(1, Ordering::Relaxed);
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(taken.load(Ordering::Relaxed) as u64, N);
        assert_eq!(sum.load(Ordering::Relaxed) as u64, N * (N + 1) / 2);
    }
}
