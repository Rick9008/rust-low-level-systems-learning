//! rehearsal g:bounded_channel —— 題目見 rehearsals/README.md。
//!
//! std-only(`Mutex` / `Condvar` / `Arc`)。
//! 只給 API 簽名;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`。

use std::collections::VecDeque;
// use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

// 4:11 start to code
/// receiver 已 drop;把值原封還給你。
#[derive(Debug)]
pub struct SendError<T>(pub T);

struct Shared<T> {
    deque: Mutex<VecDeque<T>>,
    wait_not_full: Condvar,
    wait_not_empty: Condvar,
    sender_cnt: AtomicUsize,
    reciver: AtomicBool,
    cap: usize,
}

impl<T> Shared<T> {
    fn new(cap: usize) -> Self {
        Self {
            deque: Mutex::new(VecDeque::new()),
            wait_not_full: Condvar::new(),
            wait_not_empty: Condvar::new(),
            sender_cnt: AtomicUsize::new(1), // from channel, sender and receiver has 1 instance both
            reciver: AtomicBool::new(true),
            cap,
        }
    }
}

pub struct Sender<T> {
    // ↓ 佔位:動手時整個換成你的設計。
    // _todo: PhantomData<T>,
    shared: Arc<Shared<T>>,
}

pub struct Receiver<T> {
    // _todo: PhantomData<T>,
    shared: Arc<Shared<T>>,
}

/// `capacity >= 1`。多生產者(`Sender: Clone`)、單消費者。
pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    // todo!("rehearsal")
    // SANITY TEST
    assert!(capacity >= 1);
    let shared = Arc::new(Shared::new(capacity));
    (
        Sender {
            shared: shared.clone(),
        },
        Receiver { shared },
    )
}

impl<T> Sender<T> {
    /// 滿 → block 到有空位;receiver 已 drop → `Err(SendError(v))`。
    pub fn send(&self, v: T) -> Result<(), SendError<T>> {
        // todo!("rehearsal")
        if !self.shared.reciver.load(Ordering::Acquire) {
            return Err(SendError(v));
        }

        let mut st = self.shared.deque.lock().unwrap();
        st = self
            .shared
            .wait_not_full
            .wait_while(st, |s| {
                s.len() == self.shared.cap && self.shared.reciver.load(Ordering::Acquire)
            })
            .unwrap();
        if !self.shared.reciver.load(Ordering::Acquire) {
            return Err(SendError(v));
        }
        st.push_back(v);
        drop(st);
        self.shared.wait_not_empty.notify_one();
        Ok(())
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if self.shared.sender_cnt.fetch_sub(1, Ordering::Release) == 1 {
            let _g = self.shared.deque.lock().unwrap();
            self.shared.wait_not_empty.notify_one();
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        // todo!("rehearsal")
        let shared_clone = self.shared.clone();
        self.shared.sender_cnt.fetch_add(1, Ordering::Release);
        Self {
            shared: shared_clone,
        }
    }
}

impl<T> Receiver<T> {
    /// 空 → block;所有 sender 都 drop 且 buffer 已清空 → None。
    pub fn recv(&self) -> Option<T> {
        // todo!("rehearsal")

        let mut st = self.shared.deque.lock().unwrap();
        st = self
            .shared
            .wait_not_empty
            .wait_while(st, |s| {
                s.is_empty() && self.shared.sender_cnt.load(Ordering::Acquire) > 0
            })
            .unwrap();

        let item = st.pop_front();
        drop(st);
        self.shared.wait_not_full.notify_one();
        item
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.shared.reciver.store(false, Ordering::Release);
        {
            let _st = self.shared.deque.lock().unwrap();
            self.shared.wait_not_full.notify_all();
        }
    }
}

// 4:31

// dry run

#[test]
fn dryrun() {
    let (p, c) = channel::<i32>(2);
    assert!(p.send(20).is_ok());
    assert!(p.send(30).is_ok());
    let p2 = p.clone();
    let recv = c.recv();
    match recv {
        Some(v) => assert_eq!(v, 20),
        None => unreachable!("should be value"),
    };
    let recv = c.recv();
    match recv {
        Some(v) => assert_eq!(v, 30),
        None => unreachable!("should be value"),
    };
    let join = std::thread::spawn(move || {
        let recv = c.recv();
        assert!(recv.is_some());
        match recv {
            Some(v) => assert_eq!(v, 60),
            None => panic!("should not be none"),
        };
    });
    assert!(p2.send(60).is_ok());
    assert!(p.send(30).is_ok());
    let _ = join.join();
    let res = p2.send(40);
    match res {
        Ok(_) => unreachable!("should be err"),
        Err(send_error) => assert_eq!(send_error.0, 40),
    };
}

#[test]
fn boundary_test() {
    let (p, c) = channel::<i32>(2);
    assert!(p.send(2).is_ok());
    assert!(p.send(3).is_ok());
    let join = std::thread::spawn(move || {
        assert!(p.send(6).is_err());
    });
    drop(c);
    join.join().unwrap();
}

// 4:49 - 38 mins
