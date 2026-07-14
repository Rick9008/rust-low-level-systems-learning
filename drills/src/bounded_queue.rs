//! drill:bounded_queue —— 填 predicate wait 與 close 語意。
//!
//! 已給:結構定義、try_push/try_pop、len/is_closed。
//! 要填:`push` / `pop` / `close`(mutex/condvar 面試的全部考點)。
//! 填之前先紙上 dry-run:滿時 push 誰喚醒它?close 時誰必須醒?

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

#[derive(Debug, PartialEq, Eq)]
pub struct PushError<T>(pub T);

struct State<T> {
    buf: VecDeque<T>,
    cap: usize,
    closed: bool,
}

pub struct BoundedQueue<T> {
    state: Mutex<State<T>>,
    not_empty: Condvar, // pop 在這裡等資料
    not_full: Condvar,  // push 在這裡等空位
}

impl<T> BoundedQueue<T> {
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0);
        Self {
            state: Mutex::new(State {
                buf: VecDeque::with_capacity(cap),
                cap,
                closed: false,
            }),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
        }
    }

    /// spec:阻塞直到「有空位」或「已 close」。
    /// - 未滿且未關 → push_back,喚醒一個等資料的人,回 Ok(())
    /// - 滿 → 在 not_full 上等(注意 spurious wakeup:條件要用迴圈/wait_while 重查)
    /// - 醒來發現 closed → 回 Err(PushError(item)) **歸還元素**
    ///
    /// 提示:`Condvar::wait_while(guard, |s| 條件)`;notify 前先 drop guard。
    pub fn push(&self, item: T) -> Result<(), PushError<T>> {
        todo!("spec: predicate wait on not_full; closed 歸還元素; 成功後 notify not_empty")
    }

    /// spec:阻塞直到「有資料」或「closed 且已排空」。
    /// - 有資料 → pop_front,喚醒一個等空位的人,回 Some
    /// - 空且未關 → 在 not_empty 上等
    /// - closed:**先 drain**——還有資料照樣回 Some;空了才回 None
    pub fn pop(&self) -> Option<T> {
        todo!("spec: predicate wait on not_empty; close 後先 drain 再 None; 成功後 notify not_full")
    }

    /// spec:標記 closed 並喚醒**所有**等待者(兩個 condvar 都要)。
    /// 用 notify_one 的話只有一個人醒,其餘永久卡死——想清楚為什麼。
    pub fn close(&self) {
        todo!("spec: set closed; notify_all 兩邊")
    }

    pub fn try_push(&self, item: T) -> Result<(), PushError<T>> {
        let mut st = self.state.lock().unwrap();
        if st.closed || st.buf.len() == st.cap {
            return Err(PushError(item));
        }
        st.buf.push_back(item);
        drop(st);
        self.not_empty.notify_one();
        Ok(())
    }

    pub fn try_pop(&self) -> Option<T> {
        let mut st = self.state.lock().unwrap();
        let item = st.buf.pop_front();
        if item.is_some() {
            drop(st);
            self.not_full.notify_one();
        }
        item
    }

    pub fn len(&self) -> usize {
        self.state.lock().unwrap().buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_closed(&self) -> bool {
        self.state.lock().unwrap().closed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    /// boundary:FIFO + 空/滿往返。紙上 trace cap=2 的 push/push/pop/pop。
    #[test]
    #[ignore = "填完 push/pop 後移除"]
    fn fifo_roundtrip() {
        let q = BoundedQueue::new(2);
        q.push(1).unwrap();
        q.push(2).unwrap();
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.pop(), Some(2));
    }

    /// boundary:滿 push 阻塞,pop 讓位後完成。
    #[test]
    #[ignore = "填完 push/pop 後移除"]
    fn full_push_blocks_until_pop() {
        let q = Arc::new(BoundedQueue::new(1));
        q.push(1).unwrap();
        let q2 = Arc::clone(&q);
        let producer = thread::spawn(move || q2.push(2));
        thread::sleep(Duration::from_millis(50));
        assert_eq!(q.pop(), Some(1));
        producer.join().unwrap().unwrap();
        assert_eq!(q.pop(), Some(2));
    }

    /// boundary:close 喚醒阻塞中的 pop → None;阻塞中的 push → 歸還元素。
    #[test]
    #[ignore = "填完 push/pop/close 後移除"]
    fn close_wakes_everyone() {
        let q: Arc<BoundedQueue<i32>> = Arc::new(BoundedQueue::new(1));
        let q2 = Arc::clone(&q);
        let consumer = thread::spawn(move || q2.pop());
        thread::sleep(Duration::from_millis(50));
        q.close();
        assert_eq!(consumer.join().unwrap(), None);

        let q = Arc::new(BoundedQueue::new(1));
        q.push(1).unwrap();
        let q2 = Arc::clone(&q);
        let producer = thread::spawn(move || q2.push(2));
        thread::sleep(Duration::from_millis(50));
        q.close();
        assert_eq!(producer.join().unwrap(), Err(PushError(2)));
    }

    /// boundary:close 後 drain——剩餘元素照拿,拿完才 None;push 立即失敗。
    #[test]
    #[ignore = "填完 push/pop/close 後移除"]
    fn drain_after_close() {
        let q = BoundedQueue::new(4);
        q.push(1).unwrap();
        q.push(2).unwrap();
        q.close();
        assert_eq!(q.push(3), Err(PushError(3)));
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.pop(), Some(2));
        assert_eq!(q.pop(), None);
    }
}
