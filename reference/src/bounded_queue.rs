//! # bounded_queue —— Mutex + Condvar 的阻塞式有界佇列
//!
//! ## [Clarify]
//! 解決:多執行緒間傳遞工作項,佇列滿時 producer 阻塞(backpressure)、
//! 空時 consumer 阻塞,並支援 close(shutdown 時喚醒所有等待者)。
//! Constraints:std-only、MPMC(多 producer 多 consumer)、FIFO、容量固定。
//! 預期規模:容量 10²–10⁴,執行緒數 ~10¹;每次操作持鎖時間必須 O(1)。
//!
//! ## [Abstract]
//! 元素型別 `T` 完全泛型;不做序列化、不做優先級——面試時這些先 stub 掉往前走,
//! 核心是「predicate wait + close 語意」。
//!
//! ## [Iterate]
//! `mod naive`:單一 Condvar + 一律 `notify_all`——正確但 thundering herd。
//! 本模組主體:兩個 Condvar(`not_empty` / `not_full`)+ `notify_one`——
//! 每次操作只喚醒一個「等對邊條件」的執行緒。
//!
//! ## [Trade-offs]
//! - close 後 pop 仍可 drain 剩餘元素、push 立即失敗並**歸還元素**。
//!   不對稱是刻意的:佇列裡的資料是有效的,丟掉才是 bug;新資料進不去要讓
//!   caller 知道並拿回所有權。
//! - 對照 production:crossbeam-channel(lock-free MPMC)、std::sync::mpsc。
//!
//! ## [Dry-Run]
//! 見下方測試:每個核心操作至少一個逐行 trace,boundary 涵蓋
//! 空 pop、滿 push、cap=1、close 喚醒、drain、close 歸還元素。

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

/// push 進已 close 的佇列時歸還元素,caller 保有所有權。
#[derive(Debug, PartialEq, Eq)]
pub struct PushError<T>(pub T);

struct State<T> {
    buf: VecDeque<T>,
    cap: usize,
    closed: bool,
}

pub struct BoundedQueue<T> {
    state: Mutex<State<T>>,
    // 兩個 condvar 把「等空位」與「等資料」的執行緒分開:
    // push 只需 notify_one(not_empty),不會誤醒其他 producer。
    // 單 condvar 版見 mod naive——需 notify_all,喚醒 O(waiters) 個執行緒再各自重查。
    not_empty: Condvar,
    not_full: Condvar,
}

impl<T> BoundedQueue<T> {
    /// 建立容量為 `cap` 的佇列。`cap == 0` 是 rendezvous channel 的語意
    /// (push 必須等到有 pop 在場),本實作不支援,直接 panic 講清楚。
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0, "cap=0 (rendezvous) not supported");
        Self {
            state: Mutex::new(State {
                // 預配固定容量:佇列生命週期內零 realloc,持鎖區間延遲可預測。
                // 空間代價 O(cap) 一次付清。
                buf: VecDeque::with_capacity(cap),
                cap,
                closed: false,
            }),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
        }
    }

    /// 阻塞直到有空位或被 close。O(1) 持鎖時間。
    ///
    /// 核心 idiom:**predicate wait**。`wait_while` 在「條件仍成立」時持續等,
    /// 每次被喚醒(包含 spurious wakeup)都重查條件——用 `if` 代替迴圈是經典 bug。
    pub fn push(&self, item: T) -> Result<(), PushError<T>> {
        let mut st = self.state.lock().unwrap();
        // 等待條件:滿 且 未關。close 會把等待者從這裡放行。
        st = self
            .not_full
            .wait_while(st, |s| s.buf.len() == s.cap && !s.closed)
            .unwrap();
        if st.closed {
            // 歸還所有權:caller 可以記 log、寫 fallback、或 drop——由它決定。
            return Err(PushError(item));
        }
        st.buf.push_back(item);
        drop(st); // 先解鎖再 notify:被喚醒者不會立刻撞上還被持有的鎖(避免 hurry-up-and-wait)
        self.not_empty.notify_one();
        Ok(())
    }

    /// 阻塞直到有資料;close 且 drain 完畢後回傳 `None`。O(1) 持鎖時間。
    ///
    /// close 語意:closed 只擋「新資料進來」,不擋「舊資料出去」——
    /// pop 先 drain,空了才回 None。
    pub fn pop(&self) -> Option<T> {
        let mut st = self.state.lock().unwrap();
        st = self
            .not_empty
            .wait_while(st, |s| s.buf.is_empty() && !s.closed)
            .unwrap();
        // 走到這裡:非空(拿資料)或 closed(且空 → None)。
        let item = st.buf.pop_front();
        if item.is_some() {
            drop(st);
            self.not_full.notify_one();
        }
        item
    }

    /// 不阻塞:滿或已 close 回 Err 歸還元素。
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

    /// 不阻塞:空回 None(不論是否 closed)。
    pub fn try_pop(&self) -> Option<T> {
        let mut st = self.state.lock().unwrap();
        let item = st.buf.pop_front();
        if item.is_some() {
            drop(st);
            self.not_full.notify_one();
        }
        item
    }

    /// 關閉佇列:此後 push 失敗;pop drain 完剩餘元素後回 None。
    ///
    /// 必須 `notify_all` 兩邊:所有阻塞中的 producer/consumer 都要觀察到
    /// closed 並離開等待——`notify_one` 只會放行一個,其餘永久卡死。
    pub fn close(&self) {
        let mut st = self.state.lock().unwrap();
        st.closed = true;
        drop(st);
        self.not_empty.notify_all();
        self.not_full.notify_all();
    }

    pub fn is_closed(&self) -> bool {
        self.state.lock().unwrap().closed
    }

    /// 快照值:回傳瞬間可能已過期,僅供監控/測試,不可用來做同步決策。
    pub fn len(&self) -> usize {
        self.state.lock().unwrap().buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// [Iterate] naive 版:單一 Condvar。
///
/// 正確性不變,但 push 完成時無法只喚醒 consumer——同一個 condvar 上睡著
/// 等空位的 producer 也會被 `notify_all` 掃到,醒來重查條件又睡回去。
/// 每次操作喚醒 O(waiters) 個執行緒(thundering herd),優化版是 O(1)。
/// 這裡只保留 push/pop/close 展示差異。
pub mod naive {
    use super::PushError;
    use std::collections::VecDeque;
    use std::sync::{Condvar, Mutex};

    struct State<T> {
        buf: VecDeque<T>,
        cap: usize,
        closed: bool,
    }

    pub struct NaiveBoundedQueue<T> {
        state: Mutex<State<T>>,
        cv: Condvar,
    }

    impl<T> NaiveBoundedQueue<T> {
        pub fn new(cap: usize) -> Self {
            assert!(cap > 0);
            Self {
                state: Mutex::new(State {
                    buf: VecDeque::with_capacity(cap),
                    cap,
                    closed: false,
                }),
                cv: Condvar::new(),
            }
        }

        pub fn push(&self, item: T) -> Result<(), PushError<T>> {
            let mut st = self.state.lock().unwrap();
            st = self
                .cv
                .wait_while(st, |s| s.buf.len() == s.cap && !s.closed)
                .unwrap();
            if st.closed {
                return Err(PushError(item));
            }
            st.buf.push_back(item);
            drop(st);
            // 只能 notify_all:notify_one 可能喚到另一個 producer(等空位的),
            // 它重查條件後睡回去,真正該醒的 consumer 沒被叫到 → lost wakeup。
            self.cv.notify_all();
            Ok(())
        }

        pub fn pop(&self) -> Option<T> {
            let mut st = self.state.lock().unwrap();
            st = self
                .cv
                .wait_while(st, |s| s.buf.is_empty() && !s.closed)
                .unwrap();
            let item = st.buf.pop_front();
            if item.is_some() {
                drop(st);
                self.cv.notify_all();
            }
            item
        }

        pub fn close(&self) {
            self.state.lock().unwrap().closed = true;
            self.cv.notify_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    /// [Dry-Run] FIFO 基本流。手 trace(cap=4):
    ///   push(1): buf=[1]        push(2): buf=[1,2]      push(3): buf=[1,2,3]
    ///   pop() → 1: buf=[2,3]    pop() → 2: buf=[3]      pop() → 3: buf=[]
    /// boundary:空→非空→空 的完整往返。
    #[test]
    fn fifo_order_roundtrip() {
        let q = BoundedQueue::new(4);
        q.push(1).unwrap();
        q.push(2).unwrap();
        q.push(3).unwrap();
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.pop(), Some(2));
        assert_eq!(q.pop(), Some(3));
        assert!(q.is_empty());
    }

    /// boundary:空佇列 pop 必須阻塞,直到另一線 push。
    /// trace:consumer 先起跑 → wait_while(空 && 未關) 睡下 →
    ///        main sleep 50ms 後 push(42) → notify_one(not_empty) →
    ///        consumer 醒來重查條件(非空)→ pop_front → 42。
    #[test]
    fn boundary_empty_pop_blocks_until_push() {
        let q = Arc::new(BoundedQueue::new(1));
        let q2 = Arc::clone(&q);
        let consumer = thread::spawn(move || q2.pop());
        // 讓 consumer 大概率已進入 wait(即使沒進入,語意也正確:pop 直接拿到)
        thread::sleep(Duration::from_millis(50));
        q.push(42).unwrap();
        assert_eq!(consumer.join().unwrap(), Some(42));
    }

    /// boundary:滿佇列 push 必須阻塞,直到另一線 pop 讓出空位。
    /// trace(cap=1):buf=[1](滿)→ producer push(2) 睡下 →
    ///   main pop → 1,notify_one(not_full) → producer 醒,buf=[2] → main pop → 2。
    #[test]
    fn boundary_full_push_blocks_until_pop() {
        let q = Arc::new(BoundedQueue::new(1));
        q.push(1).unwrap();
        let q2 = Arc::clone(&q);
        let producer = thread::spawn(move || q2.push(2));
        thread::sleep(Duration::from_millis(50));
        assert_eq!(q.pop(), Some(1));
        producer.join().unwrap().unwrap();
        assert_eq!(q.pop(), Some(2));
    }

    /// boundary:close 喚醒阻塞中的 pop → None(空且已關)。
    /// 沒有 close 語意時這裡會永久卡死——這就是 shutdown 路徑的價值。
    #[test]
    fn boundary_close_wakes_blocked_pop_to_none() {
        let q: Arc<BoundedQueue<i32>> = Arc::new(BoundedQueue::new(1));
        let q2 = Arc::clone(&q);
        let consumer = thread::spawn(move || q2.pop());
        thread::sleep(Duration::from_millis(50));
        q.close();
        assert_eq!(consumer.join().unwrap(), None);
    }

    /// boundary:close 喚醒阻塞中的 push → Err 且**歸還元素**(所有權還給 caller)。
    #[test]
    fn boundary_close_wakes_blocked_push_and_returns_item() {
        let q = Arc::new(BoundedQueue::new(1));
        q.push(1).unwrap(); // 佔滿
        let q2 = Arc::clone(&q);
        let producer = thread::spawn(move || q2.push(2));
        thread::sleep(Duration::from_millis(50));
        q.close();
        assert_eq!(producer.join().unwrap(), Err(PushError(2)));
    }

    /// boundary:close 後 drain——剩餘元素仍可依序 pop,之後才是 None。
    /// trace:buf=[1,2] → close → pop→1 → pop→2 → pop→None(非阻塞,因 closed)。
    #[test]
    fn boundary_drain_after_close_then_none() {
        let q = BoundedQueue::new(4);
        q.push(1).unwrap();
        q.push(2).unwrap();
        q.close();
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.pop(), Some(2));
        assert_eq!(q.pop(), None);
    }

    /// boundary:close 後 push 立即失敗(不阻塞)。
    #[test]
    fn boundary_push_after_close_fails_immediately() {
        let q = BoundedQueue::new(4);
        q.close();
        assert_eq!(q.push(7), Err(PushError(7)));
    }

    /// boundary:try_* 的滿/空立即返回,不阻塞。
    #[test]
    fn boundary_try_push_full_and_try_pop_empty() {
        let q = BoundedQueue::new(1);
        assert_eq!(q.try_pop(), None); // 空
        q.try_push(1).unwrap();
        assert_eq!(q.try_push(2), Err(PushError(2))); // 滿
        assert_eq!(q.try_pop(), Some(1));
    }

    /// MPMC 煙霧測試:4 producer × 100 項、4 consumer,總和不多不少。
    /// 驗證:無遺失、無重複(每個值恰好被一個 consumer 拿到)。
    #[test]
    fn mpmc_no_loss_no_dup() {
        let q = Arc::new(BoundedQueue::new(8));
        let producers: Vec<_> = (0..4)
            .map(|p| {
                let q = Arc::clone(&q);
                thread::spawn(move || {
                    for i in 0..100 {
                        q.push(p * 100 + i).unwrap();
                    }
                })
            })
            .collect();
        let consumers: Vec<_> = (0..4)
            .map(|_| {
                let q = Arc::clone(&q);
                thread::spawn(move || {
                    let mut sum: i64 = 0;
                    while let Some(v) = q.pop() {
                        sum += i64::from(v);
                    }
                    sum
                })
            })
            .collect();
        for p in producers {
            p.join().unwrap();
        }
        q.close(); // producer 全部完成後才 close;consumer drain 完拿 None 退出
        let total: i64 = consumers.into_iter().map(|c| c.join().unwrap()).sum();
        assert_eq!(total, (0..400).sum::<i64>());
    }

    /// naive 版行為等價(單 condvar + notify_all,效率差但正確)。
    #[test]
    fn naive_version_same_semantics() {
        let q = naive::NaiveBoundedQueue::new(2);
        q.push(1).unwrap();
        q.push(2).unwrap();
        q.close();
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.pop(), Some(2));
        assert_eq!(q.pop(), None);
        assert_eq!(q.push(3), Err(PushError(3)));
    }
}
