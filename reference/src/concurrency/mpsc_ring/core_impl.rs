//! MPSC ring(Vyukov MPMC 的單消費退化)核心演算法。
//!
//! 這個檔案被兩個地方 include:
//! 1. `mpsc_ring/mod.rs`(lib 本體,sync = std)
//! 2. `tests/loom_mpsc_ring.rs`(loom 驗證,sync = loom)
//!
//! 所以這裡只准用 `crate::sync_shim` 提供的同步原語,不直接碰 std::sync。
//!
//! 與 `mpmc_ring/core_impl.rs` 逐行對照著讀:producer 側**一字不差**;
//! 差異全在 consumer 側——這就是退化表(docs/concurrency/mpmc_ring.md)
//! 「哪端是單,那端 index 保持單寫者」的實體。

// NOTE:
// Vyukov MPSC ring depends on Seq Slot, and it can make the pair ordering only use Acquire/Release,
// and the head can only use Relaxed, because the Acquire / Release pair on Seq slot already provide
// the snychronize-with property

use crate::sync_shim as sync;
use std::mem::MaybeUninit;
use sync::atomic::{AtomicUsize, Ordering};

/// tail 獨佔一條 cache line(producer 之間本來就會 ping-pong)。
/// head 不用陪葬:它已經不是共享變數(見下),塞在冷資料區即可。
#[repr(align(64))]
struct CachePadded<T>(T);

/// 槽位 = 資料 + 發布訊號。三態同 mpmc_ring:
/// seq==pos(輪空可搶)/ pos+1(已發布可讀)/ pos+cap(已釋放,等下一圈)。
//
// 修正 by witherslin:
// dif = seq − pos,seq 永遠單獨站等號左邊(名牌 − 票)
// For push side:
// seq - pos == 0 -> no value now, 可塞不可拿
// seq - pos > 0 -> 重拿一次, 他有兩種可能 1. seq - pos == 1 2. seq - pos == cap, 對於 push
// 端來說兩種狀況都要重拿 pos, 因為他是被人搶走了塞值的機會
// seq - pos < 0 -> 滿的(-cap 還在塞值的縫隙, -cap + 1 滿的) -> Err(v) 還東西
// For pop side:
// seq - pos == 1 可拿, 其他都不行
struct Slot<T> {
    seq: AtomicUsize,
    val: sync::UnsafeCell<MaybeUninit<T>>,
}

/// Vyukov bounded MPSC queue:任意多執行緒 push、單一執行緒 pop。
///
/// 單消費買到的東西,一格一格看:
/// - pop 免 CAS(head 單寫者);
/// - **head 連 atomic 都不是**——Vyukov 協定裡 producer 從不讀 head
///   (滿的判定走槽位 seq),單 consumer 下 head 退化成消費端私有狀態,
///   只是剛好住在共享結構裡;
/// - producer 側原封不動:縫(佔位→發布)在生產側,與 consumer 數量無關,
///   所以 per-slot seq 一個都省不掉。
pub struct MpscRing<T> {
    buf: Box<[Slot<T>]>,
    mask: usize,
    cap: usize,
    /// producer 取號機。多寫者:只能 CAS——與 mpmc_ring 一字不差。
    tail: CachePadded<AtomicUsize>,
    /// 單 consumer 的讀位置。**非原子**:唯一會讀寫它的是 consumer 本人
    /// (`Consumer` 不可 Clone、`try_pop` 拿 `&mut self`)。
    head: sync::UnsafeCell<usize>,
}

// SAFETY:
// - 槽位互斥由 seq 協定保證(producer CAS 搶到號且 seq==pos 才寫;
//   consumer 讀到 seq==pos+1 才讀),資料可見性走 seq 的 Release→Acquire。
// - head 單寫者由型別系統保證(Consumer 非 Clone、&mut self),
//   producer 不碰它——非原子因此合法。
// - T: Send 因為元素跨執行緒移動。
unsafe impl<T: Send> Send for MpscRing<T> {}
unsafe impl<T: Send> Sync for MpscRing<T> {}

/// 生產把手:可 Clone(多生產者)。
pub struct Producer<T> {
    ring: sync::Arc<MpscRing<T>>,
}

impl<T> Clone for Producer<T> {
    fn clone(&self) -> Self {
        Self {
            ring: sync::Arc::clone(&self.ring),
        }
    }
}

/// 消費把手:不可 Clone——head 的「非原子」合法性整個掛在這上面。
pub struct Consumer<T> {
    ring: sync::Arc<MpscRing<T>>,
}

/// 建立容量 `cap`(上取 2 的冪,至少 2)的 MPSC channel。
pub fn channel<T>(cap: usize) -> (Producer<T>, Consumer<T>) {
    channel_with_start(cap, 0)
}

/// 測試後門:讓計數器從指定值起跑,把「溢位 wrap」拉進測試。
#[doc(hidden)]
pub fn channel_with_start<T>(cap: usize, start: usize) -> (Producer<T>, Consumer<T>) {
    assert!(cap > 0, "capacity must be at least 1");
    // 下限 2 的理由同 mpmc_ring:cap=1 時「已發布」(pos+1)與
    // 「下一圈輪空」(pos+cap)同值,三態塌縮會覆寫未消費資料。
    let cap = cap.next_power_of_two().max(2);
    let mask = cap - 1;
    let buf: Box<[Slot<T>]> = (0..cap)
        .map(|_| Slot {
            seq: AtomicUsize::new(0),
            val: sync::UnsafeCell::new(MaybeUninit::uninit()),
        })
        .collect();
    for k in 0..cap {
        let pos = start.wrapping_add(k);
        buf[pos & mask].seq.store(pos, Ordering::Relaxed);
    }
    let ring = sync::Arc::new(MpscRing {
        buf,
        mask,
        cap,
        tail: CachePadded(AtomicUsize::new(start)),
        head: sync::UnsafeCell::new(start),
    });
    (
        Producer {
            ring: sync::Arc::clone(&ring),
        },
        Consumer { ring },
    )
}

impl<T> Producer<T> {
    /// 無鎖 push。滿時 Err 歸還。**與 mpmc_ring::try_push 逐行相同**——
    /// 縫在生產側,單 consumer 幫不了 producer 任何忙。
    pub fn try_push(&self, item: T) -> Result<(), T> {
        let ring = &*self.ring;
        let mut tail = ring.tail.0.load(Ordering::Relaxed);
        loop {
            let slot = &ring.buf[tail & ring.mask];
            // Acquire 配對 consumer 釋放槽位的 store(Release)。
            let seq = slot.seq.load(Ordering::Acquire);
            let dif = seq.wrapping_sub(tail) as isize;
            if dif == 0 {
                // 輪空可搶:CAS 取號(Relaxed 就夠,同步全在 seq 上)。
                match ring.tail.0.compare_exchange_weak(
                    tail,
                    tail.wrapping_add(1),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // SAFETY(with_mut 獨佔):CAS 搶到 pos=tail 且
                        // seq==tail ⇒ consumer 已離場、槽位是空的 MaybeUninit。
                        slot.val.with_mut(|p| unsafe {
                            (*p).write(item);
                        });
                        // 發布:寫入 happens-before consumer 的 Acquire 讀。
                        slot.seq.store(tail.wrapping_add(1), Ordering::Release);
                        return Ok(());
                    }
                    Err(cur) => tail = cur,
                }
            } else if dif < 0 {
                return Err(item); // 滿:上一圈還沒被消化
            } else {
                tail = ring.tail.0.load(Ordering::Relaxed); // 被搶走,追上
            }
        }
    }

    /// 容量(已上取 2 的冪)。
    pub fn capacity(&self) -> usize {
        self.ring.cap
    }
}

impl<T> Consumer<T> {
    /// 單 consumer pop:**無 CAS、head 無原子操作**。空時 None
    /// (誠實邊界同 mpmc_ring:None = 沒有「已發布」的元素,
    /// 可能有 producer 卡在縫裡)。
    pub fn try_pop(&mut self) -> Option<T> {
        let ring = &*self.ring;
        // SAFETY(with 唯讀):head 是 consumer 私有(非 Clone + &mut self)。
        let head = ring.head.with(|p| unsafe { *p });
        let slot = &ring.buf[head & ring.mask];
        // Acquire 配對 producer 發布的 store(Release)。
        let seq = slot.seq.load(Ordering::Acquire);
        let dif = seq.wrapping_sub(head.wrapping_add(1)) as isize;
        if dif != 0 {
            // dif < 0:沒有已發布元素(空,或 producer 佔了號還沒發布)。
            // dif > 0 不可能:seq 超前 head+1 代表槽位被更晚的圈發布過,
            // 但推進圈數(seq += cap)的只有本 consumer,而我們還站在 head。
            debug_assert!(dif < 0, "single consumer 下 seq 不可能超前 head+1");
            return None;
        }
        // SAFETY(with 唯讀):seq==head+1 ⇒ 已發布且寫入可見;
        // 單 consumer ⇒ 無人與我們搶這個槽。assume_init_read move 出值。
        let item = slot.val.with(|p| unsafe { (*p).assume_init_read() });
        // 釋放:seq 跳到下一圈 producer 期望的 pos。
        slot.seq
            .store(head.wrapping_add(ring.cap), Ordering::Release);
        // SAFETY(with_mut 獨佔):head 單寫者,前進一格。
        ring.head.with_mut(|p| unsafe { *p = head.wrapping_add(1) });
        Some(item)
    }

    /// 容量(已上取 2 的冪)。
    pub fn capacity(&self) -> usize {
        self.ring.cap
    }
}

impl<T> Drop for MpscRing<T> {
    /// 兩個把手都 drop 後才會走到這(Arc 歸零):已無並發,
    /// [head, tail) 之間必然全是已發布元素,逐槽 drop 不洩漏。
    fn drop(&mut self) {
        let mut h = self.head.with(|p| unsafe { *p });
        let t = self.tail.0.load(Ordering::Relaxed);
        while h != t {
            // SAFETY:&mut self 獨佔;[head, tail) 的槽位必為已初始化。
            self.buf[h & self.mask].val.with_mut(|p| unsafe {
                (*p).assume_init_drop();
            });
            h = h.wrapping_add(1);
        }
    }
}
