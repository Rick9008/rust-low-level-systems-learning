//! drill:spsc_ring —— 填 push/pop 的 memory ordering(本 repo 最核心的填空)。
//!
//! 已給:結構(自由跑計數器 + power-of-2 mask + align(64))、
//! 槽位存取 helper(unsafe 已包好,SAFETY 已寫)、channel 建構。
//! 要填:`push` / `pop`——每一個 load/store 的 Ordering 都要能說出理由。
//!
//! 填之前紙上回答:
//! 1. 為什麼讀「自己的」index 可以 Relaxed?
//! 2. producer 讀 head 為什麼要 Acquire?配對誰的 Release?
//! 3. 為什麼一定要「先寫槽位、再 Release store tail」?反過來會怎樣?
//!
//! 填完後去跑 reference 的 loom 測試感受「窮舉 interleaving」;
//! 這裡的兩執行緒煙霧測試只是「跑很多次沒炸」等級。

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[repr(align(64))] // head/tail 各佔一條 cache line,防 false sharing
struct CachePadded<T>(T);

pub struct SpscRing<T> {
    buf: Box<[UnsafeCell<MaybeUninit<T>>]>,
    mask: usize,
    cap: usize,
    head: CachePadded<AtomicUsize>, // consumer 寫(Release)/ producer 讀(Acquire)
    tail: CachePadded<AtomicUsize>, // producer 寫(Release)/ consumer 讀(Acquire)
}

// SAFETY:單 producer 單 consumer 由型別強制(把手不可 Clone、方法拿 &mut);
// 槽位所有權交接走 head/tail 的 Release→Acquire 邊。
unsafe impl<T: Send> Send for SpscRing<T> {}
unsafe impl<T: Send> Sync for SpscRing<T> {}

impl<T> SpscRing<T> {
    /// helper(已給):把 item 寫進槽位 `idx & mask`。
    /// SAFETY 前提(caller 保證):該槽位此刻不在 consumer 可見區間
    /// [head, tail) 內,且無人同時寫。
    fn slot_write(&self, idx: usize, item: T) {
        unsafe {
            (*self.buf[idx & self.mask].get()).write(item);
        }
    }

    /// helper(已給):把槽位 `idx & mask` 的值 move 出來。
    /// SAFETY 前提(caller 保證):idx ∈ [head, tail),槽位已初始化且
    /// 對應的寫入已由 Acquire 邊同步可見。
    fn slot_take(&self, idx: usize) -> T {
        unsafe { (*self.buf[idx & self.mask].get()).assume_init_read() }
    }
}

pub struct Producer<T> {
    ring: Arc<SpscRing<T>>,
}

pub struct Consumer<T> {
    ring: Arc<SpscRing<T>>,
}

pub fn channel<T>(cap: usize) -> (Producer<T>, Consumer<T>) {
    assert!(cap > 0);
    let cap = cap.next_power_of_two();
    let ring = Arc::new(SpscRing {
        buf: (0..cap)
            .map(|_| UnsafeCell::new(MaybeUninit::uninit()))
            .collect(),
        mask: cap - 1,
        cap,
        head: CachePadded(AtomicUsize::new(0)),
        tail: CachePadded(AtomicUsize::new(0)),
    });
    (
        Producer {
            ring: Arc::clone(&ring),
        },
        Consumer { ring },
    )
}

impl<T> Producer<T> {
    /// spec:無鎖 push,滿時 Err(item) 歸還。
    /// 1. load 自己的 tail(Ordering?)
    /// 2. load 對方的 head(Ordering?)——判滿:tail.wrapping_sub(head) == cap
    /// 3. slot_write(tail, item)
    /// 4. store tail+1(wrapping_add,Ordering?)——發布給 consumer
    pub fn push(&mut self, item: T) -> Result<(), T> {
        todo!("spec: Relaxed 讀自己 / Acquire 讀對方 / 寫槽位 / Release 發布")
    }

    pub fn capacity(&self) -> usize {
        self.ring.cap
    }
}

impl<T> Consumer<T> {
    /// spec:無鎖 pop,空時 None。
    /// 1. load 自己的 head(Ordering?)
    /// 2. load 對方的 tail(Ordering?)——判空:head == tail
    /// 3. slot_take(head)
    /// 4. store head+1(Ordering?)——告訴 producer 槽位可重用
    pub fn pop(&mut self) -> Option<T> {
        todo!("spec: 對稱於 push——想清楚每個 Ordering 配對誰")
    }

    pub fn capacity(&self) -> usize {
        self.ring.cap
    }
}

impl<T> Drop for SpscRing<T> {
    fn drop(&mut self) {
        // 已無並發:把 [head, tail) 之間未消費的元素 drop 掉,不洩漏。
        let head = self.head.0.load(Ordering::Relaxed);
        let tail = self.tail.0.load(Ordering::Relaxed);
        let mut i = head;
        while i != tail {
            // SAFETY:&mut self 獨佔;[head, tail) 必為已初始化。
            unsafe {
                (*self.buf[i & self.mask].get()).assume_init_drop();
            }
            i = i.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    /// boundary:滿/空/歸還。
    #[test]
    #[ignore = "填完 push/pop 後移除"]
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
    #[ignore = "填完 push/pop 後移除"]
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
    #[ignore = "填完 push/pop 後移除"]
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
}
