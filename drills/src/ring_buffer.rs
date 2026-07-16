//! drill:ring_buffer —— 填 index 算術(wrap 是一切)。
//!
//! 已給:結構、get/front/back/iter、len 系列。
//! 要填:`wrap` / `push_back` / `pop_front` / `push_overwrite`。
//! 表示法:head + len(不浪費 slot、不需 2 的冪)。
//! 紙上先 trace cap=3:push×3、pop、push——第 4 次 push 寫進哪個實體索引?

// take a example
// push 3, 2, 1
// buf: [3, 2, 1]
//       ^
// pop
// buf: [x, 2, 1]
// len is 2
// push 0
// buf: [0, 2, 1]
//          ^
// len is 3
// so when we try to pop or push when len == buf.size()
// we need to add 1 to head
// for push when len == buf.size(): head + len is the new element exists
// for pop: head += 1, len -= 1

pub struct RingBuffer<T> {
    buf: Vec<Option<T>>, // 長度固定 == cap
    head: usize,         // 最舊元素的實體索引
    len: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0);
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

    /// spec:把邏輯位移 wrap 回實體索引。前置條件 i < 2*cap。
    /// 用條件減法,不用 `%`(想想為什麼可以:head < cap,位移 ≤ cap)。
    /// input: you should put the index you want and add the idx you want to access
    ///   ->  self.head + i
    /// output: the actual index you need to access in vec
    fn wrap(&self, i: usize) -> usize {
        // todo!("spec: i >= cap ? i - cap : i")
        // invariant: i should less than size

        let cap = self.buf.len();
        if i >= 2 * cap {
            panic!(
                "you using wrap wrong, it should not larger or equal than maximum size, undefined behavior, cap: {}, i: {}",
                cap, i
            )
        }

        if i >= cap { i - cap } else { i }
    }

    /// spec:滿 → Err(item) 歸還;否則寫入「head + len 的 wrap 位置」,len+1。
    // 3 2 1
    // [3, 2, 1]
    //  ^
    // [x, 2, 1]
    //     ^
    //     1 + 2 = 3
    //     wrap(3) = 0
    pub fn push_back(&mut self, item: T) -> Result<(), T> {
        // todo!("spec: 檢查滿; buf[wrap(head+len)] = Some(item); len += 1")
        let cap = self.buf.len();
        if self.len == cap {
            return Err(item);
        }
        let actual_idx = self.wrap(self.head + self.len);
        self.buf[actual_idx] = Some(item);
        self.len += 1;
        Ok(())
    }

    /// spec:空 → None;否則取走 head 位置的值(take),head 前進一格(wrap),len-1。
    pub fn pop_front(&mut self) -> Option<T> {
        // todo!("spec: take buf[head]; head = wrap(head+1); len -= 1")

        if self.len == 0 {
            return None;
        }

        let item = std::mem::take(&mut self.buf[self.head]);
        self.head = self.wrap(self.head + 1);
        self.len -= 1;
        item
    }

    /// spec:滿時擠掉最舊元素並回傳它(Some(evicted)),否則普通 push 回 None。
    /// 提示:pop_front + push_back 組合即可。
    pub fn push_overwrite(&mut self, item: T) -> Option<T> {
        // todo!("spec: 滿 → pop_front 後 push_back,回 evicted;未滿 → push_back,回 None")
        let evicted = if self.len == self.buf.len() {
            self.pop_front()
        } else {
            None
        };
        // invariant: if it's full, we will pop
        // (expect 需要 E: Debug,這裡 E = T 無界——用 assert!(is_ok) 不碰 E)
        //
        assert!(self.push_back(item).is_ok(), "剛騰出空間,必有空位");
        evicted
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        if i >= self.len {
            return None;
        }
        self.buf[self.wrap(self.head + i)].as_ref()
    }

    pub fn front(&self) -> Option<&T> {
        self.get(0)
    }

    pub fn back(&self) -> Option<&T> {
        self.len.checked_sub(1).and_then(|i| self.get(i))
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len).filter_map(|i| self.get(i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// boundary:wrap 臨界——滿、pop 一個、再 push 必須繞回實體索引 0。
    #[test]
    #[ignore = "填完 wrap/push_back/pop_front 後移除"]
    fn wraparound_write_and_head() {
        let mut rb = RingBuffer::new(3);
        rb.push_back(1).unwrap();
        rb.push_back(2).unwrap();
        rb.push_back(3).unwrap();
        assert_eq!(rb.pop_front(), Some(1));
        rb.push_back(4).unwrap(); // 實體索引 0
        assert_eq!(rb.iter().copied().collect::<Vec<_>>(), vec![2, 3, 4]);
        assert_eq!(rb.pop_front(), Some(2));
        assert_eq!(rb.pop_front(), Some(3));
        assert_eq!(rb.pop_front(), Some(4));
        assert_eq!(rb.pop_front(), None);
    }

    /// boundary:空 pop、滿 push(歸還)、cap=1 退化。
    #[test]
    #[ignore = "填完 wrap/push_back/pop_front 後移除"]
    fn empty_full_and_cap_one() {
        let mut rb = RingBuffer::new(1);
        assert_eq!(rb.pop_front(), None);
        rb.push_back(7).unwrap();
        assert_eq!(rb.push_back(8), Err(8));
        assert_eq!(rb.pop_front(), Some(7));
    }

    /// boundary:覆蓋模式擠掉最舊。
    #[test]
    #[ignore = "填完 push_overwrite 後移除"]
    fn overwrite_evicts_oldest() {
        let mut rb = RingBuffer::new(2);
        assert_eq!(rb.push_overwrite(1), None);
        assert_eq!(rb.push_overwrite(2), None);
        assert_eq!(rb.push_overwrite(3), Some(1));
        assert_eq!(rb.iter().copied().collect::<Vec<_>>(), vec![2, 3]);
    }

    /// boundary:繞環之後的讀取視角——get(0) 永遠最舊、back 最新、
    /// iter 由舊到新、get(len) 越界回 None。
    #[test]
    #[ignore = "填完 wrap/push_back/pop_front 後移除"]
    fn reads_are_oldest_first_across_wrap() {
        let mut rb = RingBuffer::new(3);
        for i in 1..=3 {
            rb.push_back(i).unwrap();
        }
        rb.pop_front();
        rb.push_back(4).unwrap(); // 實體索引 0,邏輯最新
        assert_eq!(rb.front(), Some(&2));
        assert_eq!(rb.back(), Some(&4));
        assert_eq!(rb.get(1), Some(&3));
        assert_eq!(rb.get(3), None); // 越界
        assert_eq!(rb.iter().copied().collect::<Vec<_>>(), vec![2, 3, 4]);
    }

    /// boundary:cap=1 連續多圈——head 每一步都繞回 0,
    /// wrap 的條件減法每次都被走到。
    #[test]
    #[ignore = "填完 wrap/push_back/pop_front 後移除"]
    fn cap_one_many_cycles() {
        let mut rb = RingBuffer::new(1);
        for i in 0..100 {
            rb.push_back(i).unwrap();
            assert!(rb.is_full());
            assert_eq!(rb.pop_front(), Some(i));
            assert!(rb.is_empty());
        }
    }

    /// oracle 對照:偽隨機 400 步 push_back / pop_front / push_overwrite,
    /// 每步與 VecDeque 模型全比對——手寫單例測試漏掉的交錯,oracle 一次掃過。
    /// (為什麼單例測試「常常抓不完」的解藥就是這種模型比對。)
    #[test]
    #[ignore = "填完 wrap/push_back/pop_front/push_overwrite 後移除"]
    fn matches_vecdeque_oracle() {
        use std::collections::VecDeque;
        const CAP: usize = 4;
        let mut rb = RingBuffer::new(CAP);
        let mut model: VecDeque<u32> = VecDeque::new();
        let mut seed: u32 = 0x2026_0716;
        for step in 0..400u32 {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223); // LCG
            match seed % 3 {
                0 => {
                    let r = rb.push_back(step);
                    if model.len() < CAP {
                        assert!(r.is_ok());
                        model.push_back(step);
                    } else {
                        assert_eq!(r, Err(step));
                    }
                }
                1 => assert_eq!(rb.pop_front(), model.pop_front()),
                _ => {
                    let evicted = rb.push_overwrite(step);
                    let model_evicted = if model.len() == CAP {
                        model.pop_front()
                    } else {
                        None
                    };
                    assert_eq!(evicted, model_evicted);
                    model.push_back(step);
                }
            }
            assert_eq!(rb.len(), model.len());
            assert!(rb.iter().eq(model.iter()), "step {step}: 內容或順序分歧");
        }
    }
}
