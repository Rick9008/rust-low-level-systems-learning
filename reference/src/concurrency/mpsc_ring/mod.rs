//! # mpsc_ring —— Vyukov 的單消費退化(退化表的實體)
//!
//! [Clarify]
//! - 多生產、單消費、固定容量的無鎖佇列:任意執行緒 `try_push`(滿時 Err
//!   歸還),**單一** consumer `try_pop`(空時 None,非阻塞)。
//! - 單消費是型別強制(`Consumer` 不可 Clone、`try_pop` 拿 `&mut self`),
//!   不是文件約定——本模組的全部紅利都掛在這上面。
//! - 對照選型:要 unbounded / push 端 wait-free → [`crate::concurrency::mpsc_list`];
//!   要多 consumer → [`crate::concurrency::mpmc_ring`]。
//!
//! [Abstract]
//! - 就是 [`crate::concurrency::mpmc_ring`] 把消費端退化:producer 側
//!   (CAS 取號 + per-slot seq 發布)**一字不差**,consumer 側兩件事降級:
//!   ①pop 的 CAS head 換成 plain store(單寫者);
//!   ②**head 連 atomic 都不是**——Vyukov 協定裡 producer 從不讀 head
//!   (滿的判定走槽位 seq),head 其實是 consumer 的私有狀態,
//!   只是剛好住在共享結構裡。「共享」與「並發存取」不是同一件事,
//!   這個欄位就是證據。
//! - per-slot seq 一個都省不掉:縫(佔位→發布)在生產側,
//!   與 consumer 數量無關。
//!
//! [Iterate]
//! - V0(壞):沿用 spsc 的「consumer Acquire 讀 tail 判空」——多 producer
//!   下 tail 是取號機不是發布訊號,會讀到佔了號還沒寫完的垃圾
//!   (mpmc_ring 的第二刀,這裡同樣躲不掉)。
//! - V1(壞):head 保留 atomic + CAS——功能正確,但把「單 consumer」
//!   這個型別事實白白丟掉:每次 pop 多付一次 RMW,還誤導讀者以為
//!   producer 會讀 head。
//! - V2(本版):head 降級為 `UnsafeCell<usize>`。loom 的 UnsafeCell
//!   存取追蹤會證明它真的無競爭(誰不信誰把 Consumer 改成 Clone 試試)。
//!
//! [Trade-offs]
//! - vs mpmc_ring:pop 少一次 CAS(~20ns 級)+ head 免 false-sharing 陪葬;
//!   代價是失去多 consumer 的可能性——**這不是效能取捨,是需求分岔**。
//! - vs mpsc_list:bounded(backpressure 內建、零配置)vs unbounded
//!   (push wait-free、絕不擋 wake)。runtime 的 remote wake 要後者;
//!   有硬容量上限的工作佇列要前者。
//! - vs spsc_ring:多了 producer 側的 CAS 與 seq;端點真的只有一對時
//!   spsc 仍是上界。退化表全景:docs/concurrency/mpmc_ring.md。
//!
//! [Dry-Run]
//! - 邊界測試 [`tests::cap_floor_and_lap_hand_trace`] 手走 cap 下限與第二圈;
//!   溢位 wrap 用 `channel_with_start` 拉進測試。
//! - `tests/loom_mpsc_ring.rs` 窮舉 2P1C 與滿載重試;consumer 側的
//!   「無競爭」由 loom 的 UnsafeCell 存取追蹤背書。

mod core_impl;

#[doc(hidden)]
pub use core_impl::channel_with_start;
pub use core_impl::{Consumer, MpscRing, Producer, channel};

#[cfg(test)]
mod tests {
    use super::{channel, channel_with_start};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    /// boundary:cap 下限 + seq 三態走一整圈(逐步帳同 mpmc_ring 的
    /// hand-trace,消費側差異:head 前進是 plain store,無 CAS)。
    ///
    /// new(1) → cap=2(三態塌縮防護)。push(7)、push(8) 填滿;
    /// push(9) 撞上一圈未消化的槽(dif<0)→ Err(9);
    /// pop×2 依取號序吐 7、8,各自把 seq 跳到下一圈(pos+cap);
    /// pop 第三次:slot0.seq==2、head+1==3,dif=-1 → None;
    /// push(9) 進第二圈的 slot0(seq==2==pos ✓)→ pop 得 9。
    #[test]
    fn cap_floor_and_lap_hand_trace() {
        let (tx, mut rx) = channel(1);
        assert_eq!(tx.capacity(), 2, "cap=1 必須上調到 2(三態塌縮)");
        tx.try_push(7).unwrap();
        tx.try_push(8).unwrap();
        assert_eq!(tx.try_push(9), Err(9));
        assert_eq!(rx.try_pop(), Some(7));
        assert_eq!(rx.try_pop(), Some(8));
        assert_eq!(rx.try_pop(), None);
        tx.try_push(9).unwrap(); // 第二圈重用 slot0
        assert_eq!(rx.try_pop(), Some(9));
    }

    /// boundary:計數器溢位——從 usize::MAX-1 起跑,跨 0 的 wrap 點
    /// FIFO 不亂(自由跑計數器 + 2 的冪 mask 的圈數算術)。
    #[test]
    fn counter_overflow_wrap_is_seamless() {
        let (tx, mut rx) = channel_with_start(2, usize::MAX - 1);
        for i in 0..8 {
            tx.try_push(i).unwrap();
            assert_eq!(rx.try_pop(), Some(i));
        }
    }

    /// 每次 drop +1(u64 沒解構子,漏 drop 看不出來)。
    #[derive(Debug)]
    struct DropSpy(Arc<AtomicUsize>);
    impl Drop for DropSpy {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// boundary:帶著未消費元素 drop——排空回收,剛好一次。
    #[test]
    fn drop_reclaims_unconsumed() {
        let n = Arc::new(AtomicUsize::new(0));
        {
            let (tx, mut rx) = channel(4);
            for _ in 0..3 {
                tx.try_push(DropSpy(n.clone())).unwrap();
            }
            drop(rx.try_pop()); // 消費 1 → +1
        } // 剩 2 由 MpscRing::drop 回收
        assert_eq!(n.load(Ordering::Relaxed), 3);
    }

    /// 4P1C 煙霧測試:不丟不重 + 每個 producer 各自保序。
    /// (真正的證明在 tests/loom_mpsc_ring.rs。)
    #[test]
    fn four_producers_smoke() {
        const PER: u64 = 25_000;
        let (tx, mut rx) = channel(8);
        let mut handles = Vec::new();
        for pid in 0..4u64 {
            let tx = tx.clone();
            handles.push(thread::spawn(move || {
                for i in 0..PER {
                    let mut item = (pid << 32) | i;
                    while let Err(back) = tx.try_push(item) {
                        item = back;
                        thread::yield_now();
                    }
                }
            }));
        }
        drop(tx);
        let mut got = 0u64;
        let mut last = [None::<u64>; 4];
        let mut all = Vec::new();
        while got < 4 * PER {
            match rx.try_pop() {
                Some(v) => {
                    let (pid, i) = ((v >> 32) as usize, v & 0xffff_ffff);
                    if let Some(prev) = last[pid] {
                        assert!(i > prev, "producer {pid} 亂序:{prev} 後來 {i}");
                    }
                    last[pid] = Some(i);
                    all.push(v);
                    got += 1;
                }
                None => thread::yield_now(),
            }
        }
        for h in handles {
            h.join().unwrap();
        }
        all.sort_unstable();
        let expect: Vec<u64> = (0..4u64)
            .flat_map(|pid| (0..PER).map(move |i| (pid << 32) | i))
            .collect();
        assert_eq!(all, expect);
    }
}
