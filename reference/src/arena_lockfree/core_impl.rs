//! arena + generation-tagged index 的 lock-free stack 核心演算法。
//!
//! 與 spsc 相同的雙重 include 架構:lib 走 std、`tests/loom_arena.rs` 走 loom。
//! 只准用 `crate::sync_shim` 的同步原語。

use crate::sync_shim as sync;
use std::mem::MaybeUninit;
use sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// 空索引哨兵(u32 索引空間的保留值)。
const NIL: u32 = u32::MAX;

/// head 打包格式:`[generation:32 | index:32]`。
/// generation 在**每次成功 CAS** 時 +1——這就是 ABA 的解:
/// 索引值可能因槽位回收而重複出現,但 (gen, idx) 這對組合在
/// 2^32 次操作視窗內不會重複,舊讀者的 CAS 必然失敗。
fn pack(generation: u32, idx: u32) -> u64 {
    (u64::from(generation) << 32) | u64::from(idx)
}

fn unpack(v: u64) -> (u32, u32) {
    ((v >> 32) as u32, v as u32)
}

struct Slot<T> {
    /// 值本體。存取權由「誰持有這個槽位」決定,規則見各 unsafe 的 SAFETY。
    value: sync::UnsafeCell<MaybeUninit<T>>,
    /// 鏈到下一個槽位(stack 鏈或 free 鏈)。
    /// 必須是 atomic:落後的 popper 可能讀到「已被回收再利用」槽位的 next
    /// ——那個讀是 stale 沒關係(它的 CAS 會因 gen 改變而失敗),
    /// 但若 next 是普通欄位,這個並發讀寫就是 data race(UB)。
    next: AtomicU32,
}

/// 固定容量、無鎖(lock-free,非 wait-free:單次操作可能因競爭重試)的 LIFO。
///
/// 兩條鏈共用一個 slot 陣列:
/// - `head` 鏈:目前在 stack 裡的元素
/// - `free` 鏈:可分配的空槽(同樣是 lock-free stack,同樣要 gen 防 ABA)
pub struct ArenaStack<T> {
    slots: Box<[Slot<T>]>,
    /// stack 頂,pack(gen, idx)。
    head: AtomicU64,
    /// free list 頂,pack(gen, idx)。
    free: AtomicU64,
}

// SAFETY: 值的所有權移交全部經由 head/free 兩個 atomic 的
// Release-CAS → Acquire-load 邊建立 happens-before(詳見各操作註解);
// 任一時刻每個槽位至多屬於一方(stack 鏈上 / free 鏈上 / 某執行緒手中),
// 歸屬轉移只發生在成功的 CAS。T: Send 因元素跨執行緒移動。
unsafe impl<T: Send> Send for ArenaStack<T> {}
unsafe impl<T: Send> Sync for ArenaStack<T> {}

impl<T> ArenaStack<T> {
    /// 容量固定 `cap`。O(cap) 初始化:所有槽位串成 free 鏈。
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0, "capacity must be at least 1");
        assert!(
            cap < NIL as usize,
            "index space is u32 with MAX reserved as NIL"
        );
        let slots: Box<[Slot<T>]> = (0..cap)
            .map(|i| Slot {
                value: sync::UnsafeCell::new(MaybeUninit::uninit()),
                // 初始 free 鏈:0 → 1 → ... → cap-1 → NIL
                next: AtomicU32::new(if i + 1 == cap { NIL } else { (i + 1) as u32 }),
            })
            .collect();
        Self {
            slots,
            head: AtomicU64::new(pack(0, NIL)),
            free: AtomicU64::new(pack(0, 0)),
        }
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// 從 free 鏈取一個槽位(lock-free pop)。None = 滿。
    ///
    /// 成功後本執行緒**獨佔**該槽位直到 push 發布它。
    fn alloc_slot(&self) -> Option<u32> {
        let mut cur = self.free.load(Ordering::Acquire);
        loop {
            let (generation, idx) = unpack(cur);
            if idx == NIL {
                return None;
            }
            // stale 讀無害:若這個槽位已被別人搶走再利用,gen 已變,CAS 會失敗。
            let next = self.slots[idx as usize].next.load(Ordering::Relaxed);
            match self.free.compare_exchange_weak(
                cur,
                pack(generation.wrapping_add(1), next),
                // 成功 Acquire:與釋放此槽位者的 Release-CAS 同步——
                // 它對 value 的最後一次讀 happens-before 我們接下來的寫。
                // (嚴格說 loop 頂的 Acquire load 已同步到同一個 store;
                //  這裡再用 Acquire 讓論證不依賴「值相等 ⇒ 同一 store」。)
                Ordering::Acquire,
                Ordering::Acquire, // 失敗:拿最新值重試
            ) {
                Ok(_) => return Some(idx),
                Err(actual) => cur = actual,
            }
        }
    }

    /// 把槽位還給 free 鏈(lock-free push)。
    fn free_slot(&self, idx: u32) {
        let mut cur = self.free.load(Ordering::Relaxed);
        loop {
            let (generation, free_idx) = unpack(cur);
            self.slots[idx as usize]
                .next
                .store(free_idx, Ordering::Relaxed);
            match self.free.compare_exchange_weak(
                cur,
                pack(generation.wrapping_add(1), idx),
                // 成功 Release:我們對這個槽位的所有存取(pop 讀走 value)
                // happens-before 下一個 alloc 到它的執行緒的寫入。
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(actual) => cur = actual,
            }
        }
    }

    /// 無鎖 push。滿時 Err 歸還元素。
    /// 攤銷 O(1);競爭下單次操作可能重試(lock-free 的定義:
    /// 整體必有進展,單一執行緒可能餓)。
    pub fn push(&self, value: T) -> Result<(), T> {
        let Some(idx) = self.alloc_slot() else {
            return Err(value); // 滿:bounded 語意,所有權還給 caller
        };
        // SAFETY(with_mut 獨佔):alloc_slot 成功 ⇒ 此槽位已從 free 鏈摘下,
        // 只有本執行緒持有;前任使用者的讀取由 alloc 的 Acquire 邊定序在前。
        // 槽位是邏輯未初始化(前任 pop 已把值 move 走),write 不會 drop 舊值。
        self.slots[idx as usize].value.with_mut(|p| unsafe {
            (*p).write(value);
        });
        let mut cur = self.head.load(Ordering::Relaxed);
        loop {
            let (generation, head_idx) = unpack(cur);
            self.slots[idx as usize]
                .next
                .store(head_idx, Ordering::Relaxed);
            match self.head.compare_exchange_weak(
                cur,
                pack(generation.wrapping_add(1), idx),
                // 成功 Release:value 與 next 的寫入 happens-before
                // 任何以 Acquire 讀到新 head 的 popper。
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(()),
                Err(actual) => cur = actual,
            }
        }
    }

    /// 無鎖 pop。空時 None。攤銷 O(1)。
    pub fn pop(&self) -> Option<T> {
        let mut cur = self.head.load(Ordering::Acquire);
        loop {
            let (generation, idx) = unpack(cur);
            if idx == NIL {
                return None;
            }
            // 經典 ABA 現場:讀 next 之後、CAS 之前,別的執行緒可能把 idx
            // pop 走、回收、再 push 回來(next 已不同)。
            // 沒有 gen:CAS 只比對 idx,會「成功」並把 head 指向一個
            // 已經不在 stack 上的 next → 結構損毀。
            // 有 gen:那串操作至少 bump 了 gen 一次,CAS 必失敗 → 重讀。
            let next = self.slots[idx as usize].next.load(Ordering::Relaxed);
            match self.head.compare_exchange_weak(
                cur,
                pack(generation.wrapping_add(1), next),
                // 成功 Acquire:與 pusher 的 Release-CAS 同步,
                // 保證下面讀 value 時看到完整寫入。
                Ordering::Acquire,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    // SAFETY(with 唯讀):CAS 勝出 ⇒ 本執行緒獨佔槽位 idx;
                    // pusher 的 value 寫入由 Acquire 邊保證可見且已初始化。
                    // assume_init_read 把值 move 走,槽位回到邏輯未初始化。
                    let value = self.slots[idx as usize]
                        .value
                        .with(|p| unsafe { (*p).assume_init_read() });
                    // 讀完才歸還槽位;free_slot 的 Release 讓下一任看到我們讀完了。
                    self.free_slot(idx);
                    return Some(value);
                }
                Err(actual) => cur = actual,
            }
        }
    }
}

impl<T> Drop for ArenaStack<T> {
    /// &mut self ⇒ 已無並發。沿 stack 鏈把還在裡面的值逐一 drop;
    /// free 鏈上的槽位是邏輯未初始化,不碰。
    fn drop(&mut self) {
        let (_, mut idx) = unpack(self.head.load(Ordering::Relaxed));
        while idx != NIL {
            let slot = &self.slots[idx as usize];
            // SAFETY:stack 鏈上的槽位必為已初始化;獨佔存取。
            slot.value.with_mut(|p| unsafe {
                (*p).assume_init_drop();
            });
            idx = slot.next.load(Ordering::Relaxed);
        }
    }
}
