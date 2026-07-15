//! solution:題 g bounded_channel——**寫完彩排才開**。
//! canonical 設計:`Mutex<State>` + **兩顆** Condvar(not_full / not_empty,
//! 一顆 + notify_all 會 thundering herd);wait 一律包在條件迴圈裡
//! (spurious wakeup:喚醒是提示,不是保證);斷線語意靠 sender 計數 + rx_alive,
//! **drop 的那一方要 notify 對向**,否則 block 中的人永遠醒不來。
//! 驗證:rehearsals/tests/bounded_channel_test.rs 全綠。

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
pub struct SendError<T>(pub T);

struct State<T> {
    buf: VecDeque<T>,
    cap: usize,
    senders: usize,
    rx_alive: bool,
}

struct Inner<T> {
    st: Mutex<State<T>>,
    not_full: Condvar,
    not_empty: Condvar,
}

pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    assert!(capacity >= 1);
    let inner = Arc::new(Inner {
        st: Mutex::new(State {
            buf: VecDeque::with_capacity(capacity),
            cap: capacity,
            senders: 1,
            rx_alive: true,
        }),
        not_full: Condvar::new(),
        not_empty: Condvar::new(),
    });
    (
        Sender {
            inner: Arc::clone(&inner),
        },
        Receiver { inner },
    )
}

impl<T> Sender<T> {
    pub fn send(&self, v: T) -> Result<(), SendError<T>> {
        let mut st = self.inner.st.lock().unwrap();
        loop {
            if !st.rx_alive {
                return Err(SendError(v)); // 值原封歸還
            }
            if st.buf.len() < st.cap {
                st.buf.push_back(v);
                self.inner.not_empty.notify_one();
                return Ok(());
            }
            st = self.inner.not_full.wait(st).unwrap();
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.inner.st.lock().unwrap().senders += 1;
        Sender {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut st = self.inner.st.lock().unwrap();
        st.senders -= 1;
        if st.senders == 0 {
            self.inner.not_empty.notify_all(); // 叫醒 block 中的 receiver → None
        }
    }
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Option<T> {
        let mut st = self.inner.st.lock().unwrap();
        loop {
            if let Some(v) = st.buf.pop_front() {
                self.inner.not_full.notify_one();
                return Some(v); // 斷線也先把 buffer 吐完
            }
            if st.senders == 0 {
                return None;
            }
            st = self.inner.not_empty.wait(st).unwrap();
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        let mut st = self.inner.st.lock().unwrap();
        st.rx_alive = false;
        self.inner.not_full.notify_all(); // 叫醒 block 中的 senders → Err
    }
}

fn main() {
    // 注意:producer 必須和 consumer 並行——cap 2 的 channel,
    // 先把兩個 producer 都跑完再開始收,就是教科書死鎖。
    let (tx, rx) = channel(2);
    let tx2 = tx.clone();
    let t1 = std::thread::spawn(move || {
        for i in 0..50 {
            tx.send(i).unwrap();
        }
    });
    let t2 = std::thread::spawn(move || {
        for i in 50..100 {
            tx2.send(i).unwrap();
        }
    });
    let mut got = Vec::new();
    while let Some(v) = rx.recv() {
        got.push(v); // 兩個 sender 都 drop(thread 結束)後回 None
    }
    t1.join().unwrap();
    t2.join().unwrap();
    got.sort_unstable();
    assert_eq!(got, (0..100).collect::<Vec<_>>());
    println!("sol_bounded_channel: ok");
}
