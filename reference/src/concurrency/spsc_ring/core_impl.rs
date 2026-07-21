//! SPSC ring 核心演算法。
//!
//! 這個檔案被兩個地方 include:
//! 1. `spsc_ring/mod.rs`(lib 本體,sync = std)
//! 2. `tests/loom_spsc.rs`(loom 驗證,sync = loom)
//!
//! 所以這裡只准用 `crate::sync_shim` 提供的同步原語,不直接碰 std::sync。

use crate::sync_shim as sync;
use std::mem::MaybeUninit;
use sync::atomic::{AtomicUsize, Ordering};

/// 讓 head / tail 各佔一條 cache line(x86_64 為 64B)。
/// 沒有它:兩個 index 落同一條 line,producer 每次 store tail 都把
/// consumer 核心的 line 打成 Invalid(false sharing),吞吐掉一個數量級。
/// 代價:每個欄位多 ~56B——空間換 cache coherence 流量。
#[repr(align(64))]
struct CachePadded<T>(T);

/// 環本體。head/tail 是**自由跑計數器**(只增不 wrap 回 0):
/// - `tail - head` 直接是元素數,滿/空判定不需要浪費一格
/// - 每個計數器只有一方會寫(tail←producer、head←consumer),SPSC 的核心前提
/// - 實體槽位 = `counter & mask`,要求 cap 為 2 的冪:2^64 是 cap 的倍數,
///   usize 溢位 wrap 時 `& mask` 的結果仍連續——cap 非 2 的冪在溢位點會跳格(bug)。
pub struct SpscRing<T> {
    buf: Box<[sync::UnsafeCell<MaybeUninit<T>>]>,
    mask: usize,
    cap: usize,
    /// consumer 的讀位置。consumer store(Release) / producer load(Acquire)。
    head: CachePadded<AtomicUsize>,
    /// producer 的寫位置。producer store(Release) / consumer load(Acquire)。
    tail: CachePadded<AtomicUsize>,
}

// SAFETY: SpscRing 跨執行緒共享的存取規則由型別系統強制——
// Producer/Consumer 不是 Clone,push/pop 又拿 &mut self,
// 所以任一時刻至多一條執行緒寫 tail+槽位、一條執行緒寫 head+讀槽位;
// 槽位所有權的交接由 tail(Release→Acquire)與 head(Release→Acquire)建立
// happens-before。T: Send 因為元素會跨執行緒移動。
unsafe impl<T: Send> Send for SpscRing<T> {}
unsafe impl<T: Send> Sync for SpscRing<T> {}

/// 生產端把手。**不是 Clone**:單一 producer 由型別系統保證,不是文件約定。
pub struct Producer<T> {
    ring: sync::Arc<SpscRing<T>>,
}

/// 消費端把手。同上,單一 consumer。
pub struct Consumer<T> {
    ring: sync::Arc<SpscRing<T>>,
}

/// 建立容量 `cap`(上取 2 的冪)的 SPSC channel。
pub fn channel<T>(cap: usize) -> (Producer<T>, Consumer<T>) {
    channel_with_start(cap, 0)
}

/// 測試後門:讓 head/tail 從指定值起跑,用來把「計數器溢位」這種
/// 2^64 次操作才會到的邊界拉到測試裡。語意與 `channel` 完全相同。
#[doc(hidden)]
pub fn channel_with_start<T>(cap: usize, start: usize) -> (Producer<T>, Consumer<T>) {
    assert!(cap > 0, "capacity must be at least 1");
    let cap = cap.next_power_of_two();
    let ring = sync::Arc::new(SpscRing {
        buf: (0..cap)
            .map(|_| sync::UnsafeCell::new(MaybeUninit::uninit()))
            .collect(),
        mask: cap - 1,
        cap,
        head: CachePadded(AtomicUsize::new(start)),
        tail: CachePadded(AtomicUsize::new(start)),
    });
    (
        Producer {
            ring: sync::Arc::clone(&ring),
        },
        Consumer { ring },
    )
}

impl<T> Producer<T> {
    /// 無鎖 push。滿時 Err 歸還元素。O(1),無 CAS——SPSC 只需要 load/store。
    pub fn push(&mut self, item: T) -> Result<(), T> {
        let ring = &*self.ring;
        // 自己的 index 用 Relaxed:tail 只有本執行緒寫,讀自己寫的值不需同步。
        let tail = ring.tail.0.load(Ordering::Relaxed);
        // Acquire 配對 consumer 的 head store(Release):看到 head=h,
        // 代表 consumer 對槽位 h-1 的讀取已完成 → 我們覆寫 tail&mask 槽位
        // (它在更早的圈次屬於 < head 的位置)不會撕掉一個正在被讀的值。
        let head = ring.head.0.load(Ordering::Acquire);
        if tail.wrapping_sub(head) == ring.cap {
            return Err(item); // 滿:backpressure 交給 caller(spin / park / 丟棄)
        }
        // SAFETY(with_mut 獨佔):槽位 tail&mask 此刻不屬於 consumer 可見區間
        // [head, tail)。上面的 Acquire 保證 consumer 已讀完舊值;
        // 又只有單一 producer(&mut self),無人與我們同時寫。
        // 槽位此刻是未初始化/已搬空的 MaybeUninit,write 不會 drop 舊值(正確:
        // 舊值早在 pop 時被 move 走)。
        ring.buf[tail & ring.mask].with_mut(|p| unsafe {
            (*p).write(item);
        });
        // Release 發布:槽位寫入 happens-before 這個 store;
        // consumer 以 Acquire 讀到新 tail 後,保證看得到完整的元素。
        // 順序不能反:先 bump tail 再寫槽位 = consumer 可能讀到垃圾。
        ring.tail.0.store(tail.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    /// 容量(已上取 2 的冪)。
    pub fn capacity(&self) -> usize {
        self.ring.cap
    }
}

impl<T> Consumer<T> {
    /// 無鎖 pop。空時 None。O(1)。
    pub fn pop(&mut self) -> Option<T> {
        let ring = &*self.ring;
        // 自己的 index:Relaxed(同 push 的理由)。
        let head = ring.head.0.load(Ordering::Relaxed);
        // Acquire 配對 producer 的 tail store(Release):
        // 看到 tail=t ⇒ 槽位 [head, t) 的元素已完整寫入。
        let tail = ring.tail.0.load(Ordering::Acquire);
        if head == tail {
            return None; // 空
        }
        // SAFETY(with 唯讀):槽位 head&mask 在 [head, tail) 內 ⇒ 已初始化
        // (由上面 Acquire 保證可見);producer 不會碰它(它只寫 [tail, head+cap))。
        // assume_init_read 把值 move 出來,槽位回到「邏輯未初始化」,
        // 之後只會被 producer 覆寫,不會 double-drop。
        let item = ring.buf[head & ring.mask].with(|p| unsafe { (*p).assume_init_read() });
        // Release:上面的讀取 happens-before 這個 store;
        // producer 以 Acquire 看到新 head 後才會覆寫該槽位。
        ring.head.0.store(head.wrapping_add(1), Ordering::Release);
        Some(item)
    }

    /// 容量(已上取 2 的冪)。
    pub fn capacity(&self) -> usize {
        self.ring.cap
    }
}

impl<T> Drop for SpscRing<T> {
    /// 兩個把手都 drop 後(Arc 歸零)才會走到這:已無並發,
    /// 把 [head, tail) 之間還沒被 pop 的元素逐一 drop,避免洩漏。
    fn drop(&mut self) {
        let head = self.head.0.load(Ordering::Relaxed);
        let tail = self.tail.0.load(Ordering::Relaxed);
        let mut i = head;
        while i != tail {
            // SAFETY:&mut self ⇒ 獨佔;[head, tail) 內的槽位必為已初始化。
            self.buf[i & self.mask].with_mut(|p| unsafe {
                (*p).assume_init_drop();
            });
            i = i.wrapping_add(1);
        }
    }
}
