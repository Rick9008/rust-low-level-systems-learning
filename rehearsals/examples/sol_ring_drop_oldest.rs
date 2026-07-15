//! solution:題 a ring_drop_oldest——**寫完彩排才開**。
//! canonical 設計:head + len 的 index 算術(不是 VecDeque——index 算術才是考點)。
//! 驗證:rehearsals/tests/ring_drop_oldest_test.rs 全綠。

use std::sync::{Arc, Mutex};

pub struct SensorRing {
    buf: Vec<u32>,
    head: usize, // 最舊元素的位置
    len: usize,
    dropped: u64,
}

impl SensorRing {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity >= 1);
        Self {
            buf: vec![0; capacity],
            head: 0,
            len: 0,
            dropped: 0,
        }
    }

    pub fn push(&mut self, value: u32) {
        let cap = self.buf.len();
        if self.len == cap {
            // 滿:tail == head(len == cap ⇒ (head+len)%cap == head)。
            // 直接寫在 head、再推 head——一步完成「丟最舊 + 收最新」。
            self.buf[self.head] = value;
            self.head = (self.head + 1) % cap;
            self.dropped += 1;
        } else {
            let tail = (self.head + self.len) % cap;
            self.buf[tail] = value;
            self.len += 1;
        }
    }

    pub fn pop(&mut self) -> Option<u32> {
        if self.len == 0 {
            return None;
        }
        let v = self.buf[self.head];
        self.head = (self.head + 1) % self.buf.len();
        self.len -= 1;
        Some(v)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }
}

// Part 2:SPSC。注意:drop-oldest 讓 producer 也要動 head(丟最舊),
// head 變成雙寫者——SPSC lock-free 的「每個 index 單寫者」不變量破了,
// 所以這題的正解是 Mutex 包單執行緒版;真要 lock-free 得上 MPMC 級技巧。
// 面試把這句講出來,比硬寫一個錯的 lock-free 值錢。

pub struct Producer {
    inner: Arc<Mutex<SensorRing>>,
}

pub struct Consumer {
    inner: Arc<Mutex<SensorRing>>,
}

pub fn channel(capacity: usize) -> (Producer, Consumer) {
    let inner = Arc::new(Mutex::new(SensorRing::new(capacity)));
    (
        Producer {
            inner: Arc::clone(&inner),
        },
        Consumer { inner },
    )
}

impl Producer {
    pub fn push(&mut self, value: u32) {
        self.inner.lock().unwrap().push(value);
    }
    pub fn dropped(&self) -> u64 {
        self.inner.lock().unwrap().dropped()
    }
}

impl Consumer {
    pub fn pop(&mut self) -> Option<u32> {
        self.inner.lock().unwrap().pop()
    }
}

fn main() {
    // smoke:cap 2,push 1,2,3 → 丟 1;pop 順序 2,3。
    let mut r = SensorRing::new(2);
    r.push(1);
    r.push(2);
    r.push(3);
    assert_eq!(r.dropped(), 1);
    assert_eq!(r.pop(), Some(2));
    assert_eq!(r.pop(), Some(3));
    assert_eq!(r.pop(), None);

    let (mut tx, mut rx) = channel(8);
    let t = std::thread::spawn(move || {
        for i in 0..100 {
            tx.push(i);
        }
        tx.dropped()
    });
    let mut got = 0u64;
    loop {
        match rx.pop() {
            Some(99) => {
                got += 1;
                break;
            }
            Some(_) => got += 1,
            None => std::thread::yield_now(),
        }
    }
    let dropped = t.join().unwrap();
    assert_eq!(got + dropped, 100);
    println!("sol_ring_drop_oldest: ok(dropped={dropped})");
}
