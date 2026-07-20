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
    // if idx >= cap, idx - cap
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
}

pub struct Producer {
    spsc: Arc<Spsc>,
}

pub struct Consumer {
    spsc: Arc<Spsc>,
}

pub fn channel(capacity: usize) -> (Producer, Consumer) {
    let spsc = Arc::new(Spsc {
        ring: Mutex::new(SensorRing::new(capacity)),
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
        st.pop()
    }
}

#[cfg(test)]
mod test {

    use crate::ring_drop_oldest::channel;
    #[test]
    fn full_pop() {
        let (mut p, mut c) = channel(2);
        p.push(2);
        p.push(1);
        assert_eq!(c.pop(), Some(2));
    }

    #[test]
    fn drop_cnt_k() {
        let (mut p, _c) = channel(2);
        p.push(2);
        p.push(1);
        p.push(0);
        p.push(0);
        assert_eq!(p.dropped(), 2);
    }

    #[test]
    fn pop_on_empty_is_nonblocking_none() {
        let (_pro, mut con) = channel(2);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(con.pop());
        });
        match rx.recv_timeout(std::time::Duration::from_millis(50)) {
            Ok(res) => assert!(res.is_none()),
            Err(_) => panic!("pop() blocked on empty - contract says non-blocking here"),
        }
    }
}
