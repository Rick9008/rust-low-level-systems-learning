//! # spsc_ring —— 單生產者單消費者的無鎖環形佇列
//!
//! ## [Clarify]
//! 解決:一條執行緒生產、一條消費(遙測上報、audio pipeline、網卡輪詢),
//! 要求 push/pop 無鎖 O(1)、無 syscall、延遲可預測(奈秒級)。
//! Constraints:**恰好一個 producer、一個 consumer**(由型別強制,見下);
//! 容量固定、上取 2 的冪;std-only。
//! 對照:MPMC 才需要 CAS 重試迴圈;SPSC 每個 index 只有一方寫,
//! load/store + acquire/release 就夠——這是它比 MPMC 快的本質原因。
//!
//! ## [Abstract]
//! 滿/空時的等待策略(spin、yield、park)不進本模組——回 `Err`/`None`,
//! caller 自己決定;面試時聲明「backpressure 策略是呼叫端的事」往前走。
//!
//! ## [Iterate]
//! 演進線:[`crate::ring_buffer`](head+len,單執行緒)→ 本模組
//! (自由跑計數器 + acquire/release,雙執行緒)。len 為什麼不能用了、
//! 為什麼換自由跑計數器,見 `docs/spsc_ring.md`。
//!
//! ## [Trade-offs]
//! - `#[repr(align(64))]` 把 head/tail 隔進不同 cache line:
//!   ~112B 空間換掉 false sharing(不隔的話兩核互踢 cache line,吞吐掉 10×)。
//! - 槽位用 `UnsafeCell<MaybeUninit<T>>`:跨執行緒移交值的必要之惡,
//!   每個 unsafe 的不變量都註在使用點;安全版(Mutex<VecDeque>)慢 ~100×。
//! - 容量上取 2 的冪:浪費最多一半空間,換 `& mask` 一條指令選槽
//!   ——以及 usize 溢位時索引連續(非 2 的冪會在溢位點跳格,是正確性問題)。
//!
//! ## [Dry-Run]
//! 單執行緒測試:滿/空/wrap/**計數器溢位**(用測試後門把 head/tail 起點設在
//! usize::MAX-1 附近)。並發正確性:**loom 窮舉**見 `tests/loom_spsc.rs`
//! ——兩執行緒的雙元素傳遞,所有 interleaving 下不丟、不重、不讀到未初始化。
//!
//! Production 對照:rtrb、ringbuf、crossbeam 的 ArrayQueue(MPMC,較慢)。

mod core_impl;

pub use core_impl::{Consumer, Producer, SpscRing, channel, channel_with_start};

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    /// [Dry-Run] 滿/空 trace(cap=2):
    ///   push(1): tail 0→1   push(2): tail 1→2(tail-head=2=cap,滿)
    ///   push(3) → Err(3)(歸還)
    ///   pop→1: head 0→1    pop→2: head 1→2(head==tail,空) pop→None
    #[test]
    fn boundary_full_and_empty() {
        let (mut tx, mut rx) = channel(2);
        tx.push(1).unwrap();
        tx.push(2).unwrap();
        assert_eq!(tx.push(3), Err(3));
        assert_eq!(rx.pop(), Some(1));
        assert_eq!(rx.pop(), Some(2));
        assert_eq!(rx.pop(), None);
    }

    /// boundary:容量上取 2 的冪(3 → 4)。
    #[test]
    fn boundary_capacity_rounds_to_power_of_two() {
        let (tx, _rx) = channel::<u8>(3);
        assert_eq!(tx.capacity(), 4);
        let (tx1, _rx1) = channel::<u8>(1);
        assert_eq!(tx1.capacity(), 1);
    }

    /// boundary:mask wrap——cap=2,推拉 10 輪,計數器一路爬到 10,
    /// 槽位 `counter & 1` 反覆繞圈。任何 off-by-one 都會讀錯槽。
    #[test]
    fn boundary_mask_wrap_many_rounds() {
        let (mut tx, mut rx) = channel(2);
        for i in 0..10 {
            tx.push(i).unwrap();
            assert_eq!(rx.pop(), Some(i));
        }
        assert_eq!(rx.pop(), None);
    }

    /// boundary:**usize 溢位**。head/tail 從 usize::MAX-1 起跑,
    /// 幾次操作後 wrapping_add 越過 0。
    /// trace(cap=2, start=MAX-1):
    ///   push(10): tail MAX-1→MAX      push(20): tail MAX→0(溢位!)
    ///   tail(0) - head(MAX-1) 用 wrapping_sub = 2 = cap → 滿,push(30)→Err ✓
    ///   pop→10: head MAX-1→MAX   pop→20: head MAX→0   相等 → 空 ✓
    /// 這就是「自由跑計數器 + 2 的冪」在溢位點也連續的直接驗證。
    #[test]
    fn boundary_counter_overflow_wraps_correctly() {
        let (mut tx, mut rx) = channel_with_start(2, usize::MAX - 1);
        tx.push(10).unwrap();
        tx.push(20).unwrap(); // tail 溢位到 0
        assert_eq!(tx.push(30), Err(30)); // wrapping_sub 判滿仍正確
        assert_eq!(rx.pop(), Some(10));
        assert_eq!(rx.pop(), Some(20)); // head 溢位到 0
        assert_eq!(rx.pop(), None);
    }

    /// 兩執行緒煙霧測試:100k 個元素順序不亂、一個不少。
    /// (這是「跑很多次沒炸」等級;窮舉版證明在 tests/loom_spsc.rs。)
    #[test]
    fn two_thread_smoke_ordered_delivery() {
        const N: u64 = 100_000;
        let (mut tx, mut rx) = channel(8);
        let producer = thread::spawn(move || {
            for i in 0..N {
                let mut item = i;
                loop {
                    match tx.push(item) {
                        Ok(()) => break,
                        Err(back) => {
                            item = back;
                            thread::yield_now(); // 滿:讓給 consumer
                        }
                    }
                }
            }
        });
        let mut expect = 0;
        while expect < N {
            match rx.pop() {
                Some(v) => {
                    assert_eq!(v, expect); // FIFO:順序必須嚴格遞增
                    expect += 1;
                }
                None => thread::yield_now(),
            }
        }
        producer.join().unwrap();
        assert_eq!(rx.pop(), None);
    }

    /// boundary:帶著未消費元素 drop——Drop 要把 [head, tail) 全清掉,不洩漏。
    #[test]
    fn boundary_drop_with_undelivered_items_no_leak() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        #[derive(Debug)]
        struct CountDrop(Arc<AtomicUsize>);
        impl Drop for CountDrop {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }

        let drops = Arc::new(AtomicUsize::new(0));
        {
            let (mut tx, mut rx) = channel(4);
            tx.push(CountDrop(Arc::clone(&drops))).unwrap();
            tx.push(CountDrop(Arc::clone(&drops))).unwrap();
            tx.push(CountDrop(Arc::clone(&drops))).unwrap();
            let popped = rx.pop().unwrap(); // 1 個正常消費
            drop(popped);
            assert_eq!(drops.load(Ordering::Relaxed), 1);
        } // 兩個把手 drop → ring drop → 剩餘 2 個元素也要被 drop
        assert_eq!(drops.load(Ordering::Relaxed), 3);
    }
}
