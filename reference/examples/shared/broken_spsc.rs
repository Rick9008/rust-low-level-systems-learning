//! **故意寫壞的** SPSC ring:把 `spsc_ring/core_impl.rs` 的 acquire/release
//! 全部降級成 `Relaxed`,其餘一字不改。
//!
//! 這個檔案被 `loom_vs_stress.rs` 用 `#[path]` include **兩次**:
//! 一次接 std 型別(真 thread 壓力測試),一次接 loom 型別(model checking)。
//! 同一份原始碼、兩套記憶體模型——正是 `sync_shim.rs` 那個機關,搬到 example 裡。
//!
//! ## 壞在哪
//! producer:先寫槽位、再 `store(tail, Relaxed)`。
//! consumer:`load(tail, Relaxed)` 看到新 tail、就去讀槽位。
//!
//! `Relaxed` **不建立 happens-before**。consumer 看到 `tail == 1`
//! 這件事,不保證 producer 對槽位的那次寫入對它可見——它可能讀到
//! 未初始化的記憶體。這在 C11 記憶體模型裡是貨真價實的 data race(UB)。
//!
//! 而在 x86 上,你幾乎**永遠測不出來**:x86-TSO 不重排 store-store、
//! 也不重排 load-load,`Relaxed` 編出來就是一條普通 `mov`,硬體照樣給你
//! 順序保證。bug 在原始碼裡,不在你的 CPU 上——所以壓力測試是綠的。

use super::sync;
use std::mem::MaybeUninit;
use sync::atomic::{AtomicUsize, Ordering};

pub struct Ring<T> {
    buf: Box<[sync::UnsafeCell<MaybeUninit<T>>]>,
    mask: usize,
    cap: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
}

// SAFETY: 這是「假設演算法正確」才成立的宣告——本檔的演算法**並不正確**,
// 這正是要證明的事。unsafe impl 在這裡是實驗器材,不是背書。
unsafe impl<T: Send> Send for Ring<T> {}
unsafe impl<T: Send> Sync for Ring<T> {}

pub struct Producer<T> {
    ring: sync::Arc<Ring<T>>,
}
pub struct Consumer<T> {
    ring: sync::Arc<Ring<T>>,
}

pub fn channel<T>(cap: usize) -> (Producer<T>, Consumer<T>) {
    let cap = cap.next_power_of_two();
    let ring = sync::Arc::new(Ring {
        buf: (0..cap)
            .map(|_| sync::UnsafeCell::new(MaybeUninit::uninit()))
            .collect(),
        mask: cap - 1,
        cap,
        head: AtomicUsize::new(0),
        tail: AtomicUsize::new(0),
    });
    (
        Producer {
            ring: sync::Arc::clone(&ring),
        },
        Consumer { ring },
    )
}

impl<T> Producer<T> {
    pub fn push(&mut self, item: T) -> Result<(), T> {
        let ring = &*self.ring;
        let tail = ring.tail.load(Ordering::Relaxed);
        // BUG ①:原版是 Acquire。這裡看到 head 前移,不代表 consumer 對槽位的
        // 讀取真的完成了——我們可能覆寫掉一個正在被讀的值。
        let head = ring.head.load(Ordering::Relaxed);
        if tail.wrapping_sub(head) == ring.cap {
            return Err(item);
        }
        ring.buf[tail & ring.mask].with_mut(|p| unsafe {
            (*p).write(item);
        });
        // BUG ②:原版是 Release。少了它,上面那次槽位寫入不會被 publish——
        // consumer 讀到新 tail 時,槽位內容對它可以是「還沒發生」。
        ring.tail.store(tail.wrapping_add(1), Ordering::Relaxed);
        Ok(())
    }
}

impl<T> Consumer<T> {
    pub fn pop(&mut self) -> Option<T> {
        let ring = &*self.ring;
        let head = ring.head.load(Ordering::Relaxed);
        // BUG ③:原版是 Acquire。沒有 acquire,就沒有跟 producer 的 release 配對,
        // 就沒有 happens-before,槽位的內容對本執行緒不保證可見。
        let tail = ring.tail.load(Ordering::Relaxed);
        if head == tail {
            return None;
        }
        let item = ring.buf[head & ring.mask].with(|p| unsafe { (*p).assume_init_read() });
        // BUG ④:原版是 Release。
        ring.head.store(head.wrapping_add(1), Ordering::Relaxed);
        Some(item)
    }
}

impl<T> Drop for Ring<T> {
    fn drop(&mut self) {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        let mut i = head;
        while i != tail {
            self.buf[i & self.mask].with_mut(|p| unsafe {
                (*p).assume_init_drop();
            });
            i = i.wrapping_add(1);
        }
    }
}
