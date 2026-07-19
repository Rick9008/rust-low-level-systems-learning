//! rehearsal a:ring_drop_oldest —— 題目見 rehearsals/README.md。
//!
//! 只給 API 簽名。結構自己設計(佔位欄位整個換掉);你自己的測試寫在本檔底部
//! `#[cfg(test)] mod tests`(模擬 CoderPad 單檔)。

// —— Part 1:單執行緒版 ——
use std::sync::Mutex;
// Bounded buf for the ring
pub struct SensorRing {
    // ↓ 佔位:動手時整個換成你的設計。
    // _todo: (),
    buf: Vec<u32>,
    head: usize,
    tail: usize,
    cap: usize, // for the buf len
    len: usize,
    drop_cnt: u64,
}

// dry run with capacity: 3
impl SensorRing {
    /// 恰好容納 `capacity` 筆;`capacity >= 1`。
    pub fn new(capacity: usize) -> Self {
        // todo!("rehearsal")
        assert!(capacity >= 1);
        SensorRing {
            buf: (0u32..capacity as u32).collect(),
            head: 0,
            tail: 0,
            cap: capacity,
            len: 0,
            drop_cnt: 0,
        }
    }

    // 0 .. cap .. 2 *cap
    // helper: wrap the index, we will use head and tail between 0 ~ 2*cap and we can use
    // wrap(tail + cap) != head to check if it is full
    // head is push side, tail is pop side.
    // if idx > cap, idx - cap
    // ex.
    //  3 is the tail, cap is 4
    //  wrap(3 + 4) = wrap(7) = return 7 - 4 = 3
    fn wrap(&self, idx: usize) -> usize {
        if idx >= self.cap { idx - self.cap } else { idx }
    }

    // tail: 0   0  0  0  1
    // head: 0   1  2  0  1
    // len: 0    1  2  3  3
    //  push     1, 2, 3, 4  -> [4, 2, 3]
    //  pop -> get Some(2)
    /// 永遠成功;滿時丟最舊的一筆並計數。
    pub fn push(&mut self, value: u32) {
        // todo!("rehearsal")
        // 0 != 0 + 3
        // 0 != 1 + 3
        if self.len == self.cap {
            self.pop();
            self.drop_cnt += 1;
        }
        self.buf[self.head] = value;
        self.head = self.wrap(self.head + 1);
        self.len += 1;
    }

    /// FIFO。
    pub fn pop(&mut self) -> Option<u32> {
        // todo!("rehearsal")
        if self.len == 0 {
            return None;
        }
        let val = self.buf[self.tail];
        self.len -= 1;
        self.tail = self.wrap(self.tail + 1);
        Some(val)
    }

    pub fn len(&self) -> usize {
        // todo!("rehearsal")
        self.len
    }

    pub fn is_empty(&self) -> bool {
        // todo!("rehearsal")
        self.len == 0
    }

    /// 累計被丟棄的筆數。
    pub fn dropped(&self) -> u64 {
        // todo!("rehearsal")
        self.drop_cnt
    }
}

// —— Part 2:SPSC 版(恰好一個 producer 執行緒、一個 consumer 執行緒)——

use std::sync::Arc;

struct Spsc {
    ring: Mutex<SensorRing>,
    // not_empty: Condvar,
}

pub struct Producer {
    spsc: Arc<Spsc>,
}

pub struct Consumer {
    spsc: Arc<Spsc>,
}

pub fn channel(capacity: usize) -> (Producer, Consumer) {
    // todo!("rehearsal")
    let spsc = Arc::new(Spsc {
        ring: Mutex::new(SensorRing::new(capacity)),
        // not_empty: Condvar::new(),
    });
    (Producer { spsc: spsc.clone() }, Consumer { spsc })
}

impl Producer {
    /// 永遠成功;滿時丟最舊的一筆並計數。
    pub fn push(&mut self, value: u32) {
        // todo!("rehearsal")
        // SAFETY unwrap:
        // only case cause Err is poisoned, and if it's really poisoned, we just panic
        let mut st = self.spsc.ring.lock().unwrap();
        st.push(value);
        drop(st);
        // self.spsc.not_empty.notify_one();
    }

    /// 累計被丟棄的筆數。
    pub fn dropped(&self) -> u64 {
        // todo!("rehearsal")
        self.spsc.ring.lock().unwrap().drop_cnt
    }
}

impl Consumer {
    /// FIFO;空 → None(不 block)。
    pub fn pop(&mut self) -> Option<u32> {
        // todo!("rehearsal")
        let mut st = self.spsc.ring.lock().unwrap();
        if st.len == 0 {
            return None;
        }
        // st = self.spsc.not_empty.wait_while(st, |s| {
        //     s.len == 0
        // }).unwrap();
        st.pop()
    }
}

fn dryrun_part_ii() {
    let (mut pro, mut con) = channel(2);
    std::thread::spawn(move || {
        con.pop(); // wait on not_empty
    });

    pro.push(2); // push and notify_one
}
