//! drill:ring_buffer —— 填 index 算術(wrap 是一切)。
//!
//! 已給:結構、get/front/back/iter、len 系列。
//! 要填:`wrap` / `push_back` / `pop_front` / `push_overwrite`。
//! 表示法:head + len(不浪費 slot、不需 2 的冪)。
//! 紙上先 trace cap=3:push×3、pop、push——第 4 次 push 寫進哪個實體索引?

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
    fn wrap(&self, i: usize) -> usize {
        todo!("spec: i >= cap ? i - cap : i")
    }

    /// spec:滿 → Err(item) 歸還;否則寫入「head + len 的 wrap 位置」,len+1。
    pub fn push_back(&mut self, item: T) -> Result<(), T> {
        todo!("spec: 檢查滿; buf[wrap(head+len)] = Some(item); len += 1")
    }

    /// spec:空 → None;否則取走 head 位置的值(take),head 前進一格(wrap),len-1。
    pub fn pop_front(&mut self) -> Option<T> {
        todo!("spec: take buf[head]; head = wrap(head+1); len -= 1")
    }

    /// spec:滿時擠掉最舊元素並回傳它(Some(evicted)),否則普通 push 回 None。
    /// 提示:pop_front + push_back 組合即可。
    pub fn push_overwrite(&mut self, item: T) -> Option<T> {
        todo!("spec: 滿 → pop_front 後 push_back,回 evicted;未滿 → push_back,回 None")
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
}
