//! drill:arena_lockfree —— 填 CAS 迴圈(push/pop),體會 generation 防 ABA。
//!
//! 已給:packed (gen|idx) 的 pack/unpack、free 鏈的 alloc_slot/free_slot、
//! 槽位存取 helper。要填:`push` / `pop` 的 CAS 重試迴圈。
//!
//! 填之前紙上回答:pop 讀了 (head, next) 之後、CAS 之前,
//! 別人把同一個 idx pop 走又 push 回來(next 已不同)——
//! 沒有 gen 的 CAS 為什麼會「成功」?成功後結構壞在哪?

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

const NIL: u32 = u32::MAX;

fn pack(generation: u32, idx: u32) -> u64 {
    (u64::from(generation) << 32) | u64::from(idx)
}

fn unpack(v: u64) -> (u32, u32) {
    ((v >> 32) as u32, v as u32)
}

struct Slot<T> {
    value: UnsafeCell<MaybeUninit<T>>,
    next: AtomicU32,
}

pub struct ArenaStack<T> {
    slots: Box<[Slot<T>]>,
    head: AtomicU64, // pack(gen, idx):stack 頂
    free: AtomicU64, // pack(gen, idx):free 鏈頂
}

// SAFETY:槽位所有權經 head/free 的 Release-CAS → Acquire 邊移交;
// 任一時刻每個槽位至多屬於一方。
unsafe impl<T: Send> Send for ArenaStack<T> {}
unsafe impl<T: Send> Sync for ArenaStack<T> {}

impl<T> ArenaStack<T> {
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0 && cap < NIL as usize);
        let slots: Box<[Slot<T>]> = (0..cap)
            .map(|i| Slot {
                value: UnsafeCell::new(MaybeUninit::uninit()),
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

    /// 已給:從 free 鏈拿一個槽位(lock-free pop)。None = 滿。
    fn alloc_slot(&self) -> Option<u32> {
        let mut cur = self.free.load(Ordering::Acquire);
        loop {
            let (generation, idx) = unpack(cur);
            if idx == NIL {
                return None;
            }
            let next = self.slots[idx as usize].next.load(Ordering::Relaxed);
            match self.free.compare_exchange_weak(
                cur,
                pack(generation.wrapping_add(1), next),
                Ordering::Acquire,
                Ordering::Acquire,
            ) {
                Ok(_) => return Some(idx),
                Err(actual) => cur = actual,
            }
        }
    }

    /// 已給:把槽位還給 free 鏈(lock-free push,Release 發布「我用完了」)。
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
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(actual) => cur = actual,
            }
        }
    }

    /// helper(已給):寫值進獨佔的槽位。SAFETY:idx 剛從 alloc_slot 拿到。
    fn slot_write(&self, idx: u32, value: T) {
        unsafe {
            (*self.slots[idx as usize].value.get()).write(value);
        }
    }

    /// helper(已給):把值從剛贏得的槽位 move 出來。
    /// SAFETY:pop 的 CAS 勝出後才可呼叫(獨佔且已初始化)。
    fn slot_take(&self, idx: u32) -> T {
        unsafe { (*self.slots[idx as usize].value.get()).assume_init_read() }
    }

    /// spec:無鎖 push。
    /// 1. alloc_slot;None → Err(value) 歸還
    /// 2. slot_write(idx, value)
    /// 3. CAS 迴圈:load head(Relaxed 起步即可)→
    ///    store slots[idx].next = 舊 head 的 idx(Relaxed)→
    ///    compare_exchange_weak(head, pack(gen+1, idx),成功 Release / 失敗 Relaxed)
    ///    失敗拿 actual 重試。
    pub fn push(&self, value: T) -> Result<(), T> {
        todo!("spec: alloc → 寫值 → CAS 迴圈發布(gen 記得 +1)")
    }

    /// spec:無鎖 pop。
    /// CAS 迴圈:load head(Acquire)→ idx == NIL 則 None →
    /// 讀 slots[idx].next(Relaxed;stale 沒關係,想清楚為什麼)→
    /// compare_exchange_weak(head, pack(gen+1, next),成功 Acquire / 失敗 Acquire)
    /// 勝出後 slot_take(idx) → free_slot(idx) → Some(value)。
    pub fn pop(&self) -> Option<T> {
        todo!("spec: CAS 搶 head;勝出才碰 value;讀完才 free_slot")
    }
}

impl<T> Drop for ArenaStack<T> {
    fn drop(&mut self) {
        let (_, mut idx) = unpack(self.head.load(Ordering::Relaxed));
        while idx != NIL {
            let slot = &self.slots[idx as usize];
            // SAFETY:stack 鏈上的槽位必為已初始化;&mut self 獨佔。
            unsafe {
                (*slot.value.get()).assume_init_drop();
            }
            idx = slot.next.load(Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// boundary:LIFO、滿、空、槽位回收重用。
    #[test]
    #[ignore = "填完 push/pop 後移除"]
    fn lifo_full_empty_recycle() {
        let s = ArenaStack::new(2);
        s.push(1).unwrap();
        s.push(2).unwrap();
        assert_eq!(s.push(3), Err(3));
        assert_eq!(s.pop(), Some(2));
        s.push(4).unwrap(); // 重用剛回收的槽位
        assert_eq!(s.pop(), Some(4));
        assert_eq!(s.pop(), Some(1));
        assert_eq!(s.pop(), None);
    }

    /// 並發煙霧:4×1000 push 全收齊無重複。
    /// (窮舉版證明在 reference 的 tests/loom_arena.rs。)
    #[test]
    #[ignore = "填完 push/pop 後移除"]
    fn concurrent_no_loss_no_dup() {
        let s = Arc::new(ArenaStack::new(4096));
        let handles: Vec<_> = (0..4u32)
            .map(|t| {
                let s = Arc::clone(&s);
                thread::spawn(move || {
                    for i in 0..1000 {
                        s.push(t * 1000 + i).unwrap();
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        let mut got = Vec::new();
        while let Some(v) = s.pop() {
            got.push(v);
        }
        got.sort_unstable();
        assert_eq!(got, (0..4000).collect::<Vec<_>>());
    }
}
