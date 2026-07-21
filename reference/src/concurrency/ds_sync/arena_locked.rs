//! # arena_locked —— Mutex 版 slab arena(alloc 發索引、caller 持索引存取)
//!
//! ## [Clarify]
//! 解決:多執行緒共享的固定容量物件池,alloc → 拿 `u32` 索引,之後憑索引
//! 讀取(`with`)或取回(`take`)。thread-safe 但**非 lock-free**:一把 Mutex。
//! Constraints:容量固定;壞索引 / double-free 回 `None`(不是 UB);
//! 不做 stale-handle 世代檢測——需要的話疊 `crate::io::fd_registry` 的 gen 手法。
//!
//! ## [Abstract]
//! 「值怎麼用」還給 caller(閉包式 `with`);「滿了怎麼辦」還給 caller(`None`)。
//!
//! ## [Iterate]
//! 本模組是 `crate::concurrency::arena_lockfree` 的**對照組**——同一需求,把「無鎖」
//! 這個約束拿掉,看資料結構怎麼塌縮:
//! gen-tagged `AtomicU64` head → 消失(臨界區裡沒有 ABA 窗口);
//! atomic `next` 侵入鏈 → `Vec<u32>` free stack(沒有並發讀者讀 stale next);
//! `MaybeUninit` + unsafe → `Option<T>`(型別系統追蹤初始化);
//! Acquire/Release 論證 → lock/unlock 自帶 happens-before。零 unsafe。
//!
//! ## [Trade-offs]
//! - 無競爭時 `Mutex::lock` 是 futex fast path(一個 CAS 進、一個 store 出),
//!   與 lock-free 版的單發 CAS 幾乎同價;有競爭時等待者去睡(不燒 CPU),
//!   lock-free 買到的是「持鎖者被 preempt 時別人仍有進展」的 progress 保證。
//! - 複合操作(alloc + 寫值)在同一臨界區裡**天生原子**;lock-free 版只有
//!   單字原子性,要靠所有權轉移協議(free 鏈摘下 = 獨佔)拼出同樣效果。
//! - `free` 預先 `Vec::with_capacity(cap)`,push 永不重配置——臨界區內
//!   零配置(不把 malloc 關進鎖裡)。
//! - 讀路徑(`with`)也要整把鎖串行。讀多場景的下一步不是 lock-free,
//!   是 sharding(對照 `crate::concurrency::sharded_map`)或 per-thread cache。
//! - 時間 O(1) + 鎖競爭;空間 O(cap)。
//!
//! ## [Dry-Run]
//! 見 `boundary_cap_one_full_cycle` 的手 trace;double-free、LIFO 槽位重用、
//! 8 執行緒煙霧測試(alloc/take 守恆)。

use std::sync::Mutex;

/// Mutex 版 slab。所有操作 `&self`。
pub struct LockedArena<T> {
    inner: Mutex<Inner<T>>,
}

struct Inner<T> {
    slots: Vec<Option<T>>,
    /// 可分配槽位的 stack(LIFO:剛還的先重用,cache 友善)。
    free: Vec<u32>,
}

impl<T> LockedArena<T> {
    /// 容量固定 `cap`。O(cap) 初始化。索引空間 u32。
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0, "capacity must be at least 1");
        assert!(u32::try_from(cap).is_ok(), "index space is u32");
        Self {
            inner: Mutex::new(Inner {
                slots: (0..cap).map(|_| None).collect(),
                // 倒序放入,讓 alloc 依 0, 1, 2… 的順序發索引(可觀察、好測)
                free: (0..cap as u32).rev().collect(),
            }),
        }
    }

    pub fn capacity(&self) -> usize {
        self.inner.lock().unwrap().slots.len()
    }

    /// 使用中的槽位數。O(1)。
    pub fn len(&self) -> usize {
        let g = self.inner.lock().unwrap();
        g.slots.len() - g.free.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 放入值,發索引。滿了 `Err(value)` 原封退回
    /// (bounded 語意,與 `arena_lockfree` 的 `push` 一致)。O(1)。
    pub fn alloc(&self, value: T) -> Result<u32, T> {
        let mut g = self.inner.lock().unwrap();
        let Some(idx) = g.free.pop() else {
            return Err(value); // 滿:bounded 語意,所有權還給 caller
        };
        g.slots[idx as usize] = Some(value);
        Ok(idx)
    }

    /// 取回值並釋放槽位。壞索引 / 已釋放 → `None`。O(1)。
    pub fn take(&self, idx: u32) -> Option<T> {
        let mut g = self.inner.lock().unwrap();
        let value = g.slots.get_mut(idx as usize)?.take()?;
        g.free.push(idx); // with_capacity 已預留:臨界區內不重配置
        Some(value)
    }

    /// 憑索引讀取:閉包在鎖內執行(借用出不了 guard——這是鎖版容器的 API 稅,
    /// 對照 lock-free 版根本不提供 by-index 讀)。壞索引 → `None`。
    pub fn with<R>(&self, idx: u32, f: impl FnOnce(&T) -> R) -> Option<R> {
        let g = self.inner.lock().unwrap();
        g.slots.get(idx as usize)?.as_ref().map(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// [Dry-Run] 手 trace(cap=1):
    ///   new:slots=[None], free=[0]
    ///   alloc(7):free.pop→0,slots[0]=Some(7) ⇒ Ok(0);len=1
    ///   alloc(8):free 空 ⇒ Err(8) 原值退回
    ///   with(0):Some(&7) → 讀出 7
    ///   take(0):slots[0].take→Some(7),free=[0] ⇒ Some(7);len=0
    ///   take(0):slots[0] 已是 None ⇒ None(double-free 擋下)
    ///   alloc(9):重用槽 0 ⇒ Ok(0)
    #[test]
    fn boundary_cap_one_full_cycle() {
        let a = LockedArena::new(1);
        assert_eq!(a.alloc(7), Ok(0));
        assert_eq!(a.len(), 1);
        assert_eq!(a.alloc(8), Err(8));
        assert_eq!(a.with(0, |v| *v), Some(7));
        assert_eq!(a.take(0), Some(7));
        assert_eq!(a.take(0), None); // double-free
        assert_eq!(a.len(), 0);
        assert_eq!(a.alloc(9), Ok(0)); // 槽位重用
    }

    /// LIFO 重用順序可觀察:發號 0,1;還 0;下一次 alloc 拿回 0。
    #[test]
    fn slot_reuse_is_lifo() {
        let a = LockedArena::new(4);
        assert_eq!(a.alloc("a"), Ok(0));
        assert_eq!(a.alloc("b"), Ok(1));
        assert_eq!(a.take(0), Some("a"));
        assert_eq!(a.alloc("c"), Ok(0));
        assert_eq!(a.with(1, |v| *v), Some("b"));
    }

    /// boundary:壞索引(越界)一律 None,不 panic。
    #[test]
    fn out_of_range_index_is_none() {
        let a = LockedArena::new(2);
        assert_eq!(a.take(99), None);
        assert_eq!(a.with(99, |v: &i32| *v), None);
    }

    /// 並發煙霧測試:8 執行緒各自 alloc→take 500 輪,守恆:結束時全空。
    #[test]
    fn concurrent_alloc_take_conserves() {
        let a = Arc::new(LockedArena::new(8));
        let handles: Vec<_> = (0..8u32)
            .map(|t| {
                let a = Arc::clone(&a);
                thread::spawn(move || {
                    for i in 0..500 {
                        // 池夠大(cap = 執行緒數):alloc 必成功
                        let idx = a.alloc(t * 1000 + i).unwrap();
                        assert_eq!(a.take(idx), Some(t * 1000 + i));
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(a.len(), 0);
    }
}
