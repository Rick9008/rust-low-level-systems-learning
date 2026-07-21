//! # ring_buffer —— 單執行緒 bounded ring(資料結構面)
//!
//! ## [Clarify]
//! 解決:固定容量的 FIFO,push/pop O(1)、零 realloc、記憶體上限可預測——
//! telemetry、log buffer、固定深度 pipeline 的典型底層。
//! Constraints:單執行緒(並發版見 [`crate::concurrency::spsc_ring`],那邊講 memory ordering,
//! 這邊講 index 算術)。容量不必是 2 的冪。
//!
//! ## [Abstract]
//! 元素型別泛型;滿時的策略(拒絕 vs 覆蓋最舊)兩種都給,由 caller 選。
//!
//! ## [Iterate]
//! 索引表示法是本模組的核心決策,三種常見方案:
//! 1. head + tail,留一格空(滿 = `next(tail)==head`):浪費 1 slot,滿/空可區分
//! 2. head + tail 自由跑不 wrap(spsc_ring 用這招):需要 2 的冪 + wrapping 算術
//! 3. **head + len(本實作)**:不浪費 slot、不需 2 的冪、滿/空直接看 len——
//!    單執行緒下最簡單正確的形狀(len 需要兩個角色同時寫時不適用,那是 spsc 的事)
//!
//! ## [Trade-offs]
//! - `Vec<Option<T>>` 而非 `Vec<MaybeUninit<T>>`:每格多一個 discriminant
//!   (T 有 niche 時免費),換來零 unsafe。面試先寫這版,追問再談 MaybeUninit。
//! - wrap 用條件減法而非 `%`:`%` 是整數除法(x86 ~20-40 cycle),
//!   條件減法一個 cmp+sub;且不要求容量是 2 的冪(mask 版要求)。
//! - 全部操作 O(1) 時間;空間 O(cap) 一次配足。
//!
//! ## [Dry-Run]
//! 見測試:空 pop、滿 push、**wrap 臨界點逐格 trace**、cap=1、覆蓋模式、proptest 對照 VecDeque。

pub struct RingBuffer<T> {
    buf: Vec<Option<T>>, // 長度固定 == cap,槽位重複使用
    head: usize,         // 最舊元素所在索引;僅在 pop 時前進
    len: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0, "zero-capacity ring is degenerate");
        Self {
            buf: (0..cap).map(|_| None).collect(),
            head: 0,
            len: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == self.buf.len()
    }

    /// 把邏輯位移 wrap 回實體索引。前置條件:`i < 2*cap`
    /// (head < cap 且位移 ≤ cap,和最大 2cap-1,一次條件減法必然夠)。
    fn wrap(&self, i: usize) -> usize {
        let cap = self.buf.len();
        if i >= cap { i - cap } else { i }
    }

    /// O(1)。滿時拒絕並歸還元素(backpressure 語意)。
    pub fn push_back(&mut self, item: T) -> Result<(), T> {
        if self.is_full() {
            return Err(item);
        }
        let idx = self.wrap(self.head + self.len);
        self.buf[idx] = Some(item);
        self.len += 1;
        Ok(())
    }

    /// O(1)。滿時覆蓋最舊元素並回傳它(telemetry 語意:新資料比舊資料值錢)。
    pub fn push_overwrite(&mut self, item: T) -> Option<T> {
        if self.is_full() {
            let evicted = self.pop_front();
            let _ = self.push_back(item); // 剛騰出一格,必成功
            evicted
        } else {
            let _ = self.push_back(item);
            None
        }
    }

    /// O(1)。
    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let item = self.buf[self.head].take();
        self.head = self.wrap(self.head + 1);
        self.len -= 1;
        item
    }

    pub fn front(&self) -> Option<&T> {
        self.get(0)
    }

    pub fn back(&self) -> Option<&T> {
        self.len.checked_sub(1).and_then(|i| self.get(i))
    }

    /// 邏輯索引存取:get(0)=最舊、get(len-1)=最新。O(1)。
    pub fn get(&self, i: usize) -> Option<&T> {
        if i >= self.len {
            return None;
        }
        self.buf[self.wrap(self.head + i)].as_ref()
    }

    /// 由舊到新迭代。整體 O(len)。
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len).filter_map(|i| self.get(i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::VecDeque;

    /// [Dry-Run] wrap 臨界點逐格 trace(cap=3):
    ///   push(1): head=0 len=1 buf=[1,_,_]     push(2): len=2 buf=[1,2,_]
    ///   push(3): len=3 buf=[1,2,3](滿)
    ///   pop→1 : head=1 len=2 buf=[_,2,3]
    ///   push(4): 寫入位置 wrap(1+2)=wrap(3)=0 ← **繞回開頭**,buf=[4,2,3]
    ///   pop→2 : head=2                       pop→3 : head=wrap(3)=0 ← head 也繞回
    ///   pop→4 : head=1 len=0
    /// boundary:寫入索引 wrap、head wrap、滿→部分排空→再填。
    #[test]
    fn boundary_wraparound_write_and_head() {
        let mut rb = RingBuffer::new(3);
        rb.push_back(1).unwrap();
        rb.push_back(2).unwrap();
        rb.push_back(3).unwrap();
        assert_eq!(rb.pop_front(), Some(1));
        rb.push_back(4).unwrap(); // 實體索引 0:wrap 發生
        assert_eq!(rb.pop_front(), Some(2));
        assert_eq!(rb.pop_front(), Some(3));
        assert_eq!(rb.pop_front(), Some(4));
        assert_eq!(rb.pop_front(), None);
    }

    /// boundary:空 pop 回 None、不 panic。
    #[test]
    fn boundary_pop_empty_returns_none() {
        let mut rb: RingBuffer<i32> = RingBuffer::new(2);
        assert_eq!(rb.pop_front(), None);
    }

    /// boundary:滿 push 拒絕並歸還元素(所有權還給 caller)。
    #[test]
    fn boundary_push_full_returns_item_back() {
        let mut rb = RingBuffer::new(2);
        rb.push_back(1).unwrap();
        rb.push_back(2).unwrap();
        assert_eq!(rb.push_back(3), Err(3));
        assert_eq!(rb.len(), 2); // 內容未被破壞
    }

    /// boundary:cap=1 退化——每次 push 都緊貼滿/空兩個邊界。
    #[test]
    fn boundary_cap_one_alternating() {
        let mut rb = RingBuffer::new(1);
        for i in 0..5 {
            rb.push_back(i).unwrap();
            assert!(rb.is_full());
            assert_eq!(rb.pop_front(), Some(i));
            assert!(rb.is_empty());
        }
    }

    /// 覆蓋模式:滿時吐出最舊。trace(cap=2):
    ///   push_overwrite(1)→None  push_overwrite(2)→None
    ///   push_overwrite(3)→Some(1)(1 最舊被擠掉)  內容=[2,3]
    #[test]
    fn push_overwrite_evicts_oldest() {
        let mut rb = RingBuffer::new(2);
        assert_eq!(rb.push_overwrite(1), None);
        assert_eq!(rb.push_overwrite(2), None);
        assert_eq!(rb.push_overwrite(3), Some(1));
        assert_eq!(rb.iter().copied().collect::<Vec<_>>(), vec![2, 3]);
    }

    /// 邏輯索引與迭代順序(含 wrap 後):get(0) 永遠是最舊。
    #[test]
    fn get_and_iter_are_oldest_first_across_wrap() {
        let mut rb = RingBuffer::new(3);
        rb.push_back(1).unwrap();
        rb.push_back(2).unwrap();
        rb.pop_front();
        rb.push_back(3).unwrap();
        rb.push_back(4).unwrap(); // 實體上已 wrap
        assert_eq!(rb.get(0), Some(&2));
        assert_eq!(rb.back(), Some(&4));
        assert_eq!(rb.get(3), None); // 越界回 None
        assert_eq!(rb.iter().copied().collect::<Vec<_>>(), vec![2, 3, 4]);
    }

    proptest! {
        /// property:任意操作序列下與 VecDeque(容量語意由測試維護)行為一致。
        /// 這是「模型測試」:拿 std 的正確實作當 oracle,窮不盡的 boundary 交給隨機。
        #[test]
        fn prop_matches_vecdeque_model(ops in proptest::collection::vec((0u8..3, 0i32..100), 1..200)) {
            let mut rb = RingBuffer::new(4);
            let mut model: VecDeque<i32> = VecDeque::new();
            for (op, v) in ops {
                match op {
                    0 => {
                        let expect_ok = model.len() < 4;
                        let got = rb.push_back(v);
                        prop_assert_eq!(got.is_ok(), expect_ok);
                        if expect_ok { model.push_back(v); }
                    }
                    1 => {
                        prop_assert_eq!(rb.pop_front(), model.pop_front());
                    }
                    _ => {
                        let expect_evict = if model.len() == 4 { model.pop_front() } else { None };
                        model.push_back(v);
                        prop_assert_eq!(rb.push_overwrite(v), expect_evict);
                    }
                }
                prop_assert_eq!(rb.len(), model.len());
                prop_assert_eq!(rb.iter().copied().collect::<Vec<_>>(), model.iter().copied().collect::<Vec<_>>());
            }
        }
    }
}
