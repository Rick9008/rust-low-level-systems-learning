//! ★ challenge:SPSC lock-free ring buffer
//!
//! 【題目】實作單生產者單消費者的無鎖環形佇列:一條執行緒 push、
//! 另一條 pop,不使用任何鎖(Mutex/Condvar/channel 都不行),
//! 只用 atomic 操作。
//!
//! 【constraints】
//! - std-only;push/pop 均攤 O(1)、無 syscall
//! - 容量固定(建構時決定,可向上調整到方便的數字)
//! - 滿時 push 回 Err 歸還元素;空時 pop 回 None(不阻塞)
//! - 「單生產者單消費者」必須由**型別系統**強制,不是靠註解
//!
//! 【clarify points——動手前先自答】
//! - 兩個 index 各由誰寫?讀對方的 index 時需要什麼 memory ordering?為什麼?
//! - 滿與空怎麼區分?你的 index 表示法在 usize 溢位時還對嗎?
//! - 元素跨執行緒移交,槽位用什麼型別存?`Vec<T>` 直接存為什麼不行?
//! - 帶著未消費元素被 drop 時,誰負責 drop 它們?
//!
//! 【要實作】下方三個簽名。struct 內部完全自己設計。
//! 【驗收】tests/spsc_ring.rs 轉綠(含 10 萬元素的雙執行緒順序測試),
//! 然後跑 reference 的 loom 測試對照你的 ordering 選擇。

// We can use a ring buffer to handle this problem, however, because it need 2 thread use one
// producer and consumer in the same time, we need to maintain head and tail with atomic variable

// I want to use bounded structure with fixed heap allocate datas
// and need UnsafeCell for internal mutablity because we need to use reference to change the value
// inside across 2 threads
// and use MaybeUninit to reduce runtime cost, with the head and tail, we can tell wheater a
// postion has value

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicUsize;

#[repr(align(64))]
struct CachePadding<T>(T);

struct SpscRing<T> {
    buf: Box<[UnsafeCell<MaybeUninit<T>>]>,
    cap: usize,
    // when we try to increase the head and tail, we use unlimited increase and use wrapping_add /
    // wrapping_sub to check the boundary and get the actual index from the mask
    mask: usize,
    tail: CachePadding<AtomicUsize>,
    head: CachePadding<AtomicUsize>,
}

// I want to keep the write / read operation in SpscRing itself, and keep the unsafe here

impl<T> SpscRing<T> {
    fn slot_write(&self, idx: usize, item: T) {
        // SAFETY: you should always use the head and tail to call this func
        // otherwise you must check this index already init
        unsafe {
            (*self.buf[idx & self.mask].get()).write(item);
        }
    }

    fn slot_read(&self, idx: usize) -> T {
        // SAFETY: you should always use the head and tail to call this func
        // otherwise you must check this index already init
        unsafe { (*self.buf[idx & self.mask].get()).assume_init_read() }
    }
}

impl<T> Drop for SpscRing<T> {
    fn drop(&mut self) {
        let mut head = self.head.0.load(Ordering::Relaxed);
        let tail = self.tail.0.load(Ordering::Relaxed);
        while head != tail {
            unsafe {
                // MaybeUninit need to drop by our self
                (*self.buf[head & self.mask].get()).assume_init_drop();
            }
            head = head.wrapping_add(1);
        }
    }
}

// SAFETY:
// the SpscRing we will ensure 2 threads to write or read so only one will push or pop
// and it cannot clone the producer or consumer, so SpscRing is safe with UnsafeCell
// furthermore, T should be Send because the T will send across threads
// but we don't give &T so T doesn't need sync
unsafe impl<T: Send> Send for SpscRing<T> {}
unsafe impl<T: Send> Sync for SpscRing<T> {}

// use std::marker::PhantomData;
use std::sync::Arc;

pub struct Producer<T> {
    // ↓ 佔位:讓空殼能編譯。動手時整個換成你的設計。
    // _todo: PhantomData<T>,
    ring: Arc<SpscRing<T>>,
}

pub struct Consumer<T> {
    // _todo: PhantomData<T>,
    ring: Arc<SpscRing<T>>,
}

/// 建立容量至少為 `cap` 的 SPSC channel。
/// and we need to use the power of 2, because cap - 1 only works when it's power of 2(bitmask)
pub fn channel<T: Send>(cap: usize) -> (Producer<T>, Consumer<T>) {
    // todo!("challenge: 從空白開始")
    assert!(cap > 0);
    let cap = cap.next_power_of_two();
    let ring = Arc::new(SpscRing {
        buf: (0..cap)
            .map(|_| UnsafeCell::new(MaybeUninit::uninit()))
            .collect(),
        cap,
        mask: cap - 1,
        tail: CachePadding(AtomicUsize::new(0)),
        head: CachePadding(AtomicUsize::new(0)),
    });
    (Producer { ring: ring.clone() }, Consumer { ring })
}

use std::sync::atomic::Ordering;

impl<T: Send> Producer<T> {
    /// 無鎖 push;滿時 Err(item) 歸還。
    pub fn push(&mut self, item: T) -> Result<(), T> {
        // todo!("challenge")

        let tail = self.ring.tail.0.load(Ordering::Relaxed);
        let head = self.ring.head.0.load(Ordering::Acquire);
        if tail.wrapping_sub(head) == self.ring.cap {
            return Err(item);
        }

        self.ring.slot_write(tail, item);
        self.ring
            .tail
            .0
            .store(tail.wrapping_add(1), Ordering::Release);

        Ok(())
    }
}

impl<T: Send> Consumer<T> {
    /// 無鎖 pop;空時 None。
    pub fn pop(&mut self) -> Option<T> {
        // todo!("challenge")

        let tail = self.ring.tail.0.load(Ordering::Acquire);
        let head = self.ring.head.0.load(Ordering::Relaxed);
        if tail.wrapping_sub(head) == 0 {
            return None;
        }

        let item = self.ring.slot_read(head);
        self.ring
            .head
            .0
            .store(head.wrapping_add(1), Ordering::Release);

        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    /// boundary:滿/空/歸還。
    #[test]
    fn full_empty_roundtrip() {
        let (mut tx, mut rx) = channel(2);
        tx.push(1).unwrap();
        tx.push(2).unwrap();
        assert_eq!(tx.push(3), Err(3));
        assert_eq!(rx.pop(), Some(1));
        assert_eq!(rx.pop(), Some(2));
        assert_eq!(rx.pop(), None);
    }

    /// boundary:mask wrap 多輪。
    #[test]
    fn wrap_many_rounds() {
        let (mut tx, mut rx) = channel(2);
        for i in 0..10 {
            tx.push(i).unwrap();
            assert_eq!(rx.pop(), Some(i));
        }
    }

    /// 兩執行緒煙霧測試:100k 元素順序不亂一個不少。
    /// (真正的證明是 reference 的 loom 測試——這裡只是 sanity。)
    #[test]
    fn two_thread_smoke() {
        const N: u64 = 100_000;
        let (mut tx, mut rx) = channel(8);
        let producer = thread::spawn(move || {
            for i in 0..N {
                let mut item = i;
                while let Err(back) = tx.push(item) {
                    item = back;
                    thread::yield_now();
                }
            }
        });
        let mut expect = 0;
        while expect < N {
            match rx.pop() {
                Some(v) => {
                    assert_eq!(v, expect);
                    expect += 1;
                }
                None => thread::yield_now(),
            }
        }
        producer.join().unwrap();
    }

    // ───────── 2026-07-18 review 補的測試(只驗你的 impl,沒改它)─────────

    /// 幫手:每次被 drop 就把共享計數器 +1。
    /// u64 沒有解構子,漏 drop 也看不出來——這個型別才驗得到 Drop。
    /// (derive Debug 只是為了 `push(...).unwrap()` 在滿時能印出被歸還的值。)
    #[derive(Debug)]
    struct DropSpy(Arc<AtomicUsize>);
    impl Drop for DropSpy {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// 非 Copy 型別(String)進出:驗 assume_init_read 的 move 語意
    /// 對「擁有堆資源」的 T 也對(不是只有 Copy 能過)。現在應該綠。
    #[test]
    fn non_copy_string_roundtrip() {
        let (mut tx, mut rx) = channel(2);
        tx.push(String::from("hello")).unwrap();
        tx.push(String::from("world")).unwrap();
        assert_eq!(rx.pop().as_deref(), Some("hello"));
        assert_eq!(rx.pop().as_deref(), Some("world"));
        assert_eq!(rx.pop(), None);
    }

    /// 容量邊界 + 環狀重用:塞到滿 → FIFO 全取出 → 排空後還能再塞。
    /// cap 用 2 的冪,避開 #1,單驗 push/pop 的重用邏輯。現在應該綠。
    #[test]
    fn fill_to_capacity_then_reuse() {
        let (mut tx, mut rx) = channel(4);
        let mut pushed = 0u64;
        while tx.push(pushed).is_ok() {
            pushed += 1;
        }
        assert!(pushed >= 4, "容量至少 4:實際塞進 {pushed}");
        for i in 0..pushed {
            assert_eq!(rx.pop(), Some(i), "FIFO 應保序");
        }
        assert_eq!(rx.pop(), None);
        tx.push(999).unwrap(); // 排空後環狀重用
        assert_eq!(rx.pop(), Some(999));
    }

    /// 🔴 抓 #2:帶著未消費元素被 drop,Drop 要把它們全數回收、剛好一次。
    /// 推 3、消費 1、剩 2 在環裡;channel 離開 scope 後總 drop 次數該 = 3
    /// (1 消費 + 2 被 SpscRing::drop 回收)。
    /// 目前你的 Drop 方向錯 → 會走進未初始化槽 → **UB,很可能直接崩(SIGSEGV)**。
    /// 那個崩就是紅燈。把 Drop 改成「從 head 走到 tail」後應該剛好 = 3。
    #[test]
    fn drop_reclaims_unconsumed() {
        let n = Arc::new(AtomicUsize::new(0));
        {
            let (mut tx, mut rx) = channel(4);
            for _ in 0..3 {
                tx.push(DropSpy(n.clone())).unwrap();
            }
            drop(rx.pop()); // 消費 1 → drop +1
        } // 剩 2 個該由 SpscRing::drop 回收
        assert_eq!(
            n.load(Ordering::Relaxed),
            3,
            "應剛好 3 次;<3 = Drop 漏收,崩/亂 = Drop 走錯範圍碰到未初始化槽"
        );
    }

    /// 🔴 抓 #2 另一角:完全不 pop 就 drop,3 個都該回收。
    #[test]
    fn drop_reclaims_when_none_popped() {
        let n = Arc::new(AtomicUsize::new(0));
        {
            let (mut tx, _rx) = channel(4);
            for _ in 0..3 {
                tx.push(DropSpy(n.clone())).unwrap();
            }
        }
        assert_eq!(
            n.load(Ordering::Relaxed),
            3,
            "未消費的 3 個都該被 Drop 回收"
        );
    }

    /// 🔴 抓 #1:cap 非 2 的冪時,mask 沒處理好會讓多個邏輯位置撞同一實體槽
    /// → 值被覆蓋、FIFO 崩。目前的 `mask = cap - 1` 會讓這條紅。
    /// 註:本測試假設你選「向上取 2 的冪」契約;若你選 assert!(is_power_of_two),
    /// channel(5) 會 panic,那就把它改成 #[should_panic]。
    #[test]
    fn odd_capacity_preserves_fifo() {
        let (mut tx, mut rx) = channel(5);
        let mut pushed = 0u64;
        while tx.push(pushed).is_ok() {
            pushed += 1;
            if pushed > 1024 {
                break; // 保險:契約沒實作時別無限塞
            }
        }
        for i in 0..pushed {
            assert_eq!(rx.pop(), Some(i), "非 2 的冪 cap:槽位撞車,FIFO 崩");
        }
        assert_eq!(rx.pop(), None);
    }
}
