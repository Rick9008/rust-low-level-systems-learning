//! sim k【virtual onsite 準備題】—— per-core telemetry fan-in(題幹:`docs/interviews/sim-problems.md`)。
//!
//! 「題目給的介面」區一律英文(中文對照:`docs/interviews/sim-problems-zh.md`)。
//!
//! 彩排規則同 sim_i:實作+自寫測試在本檔;`tests/sim_k_fanin_test.rs` 跑完才開。
//! ⚠ mock 的 Chan 用 Mutex 換單檔簡潔——協定 = 每核一條 SPSC(try 語意、固定容量);
//! 真上場換 lock-free ring,producer/aggregator 的邏輯不變。

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

// ===================== 題目給的介面(可讀)=====================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    pub core: usize,
    pub seq: u64,
}

/// One SPSC channel per core (mock). Producer side is try-only —
/// **producers must never block**.
pub struct Chan {
    cap: usize,
    q: Mutex<VecDeque<Record>>,
}

impl Chan {
    pub fn new(cap: usize) -> Self {
        Self {
            cap,
            q: Mutex::new(VecDeque::new()),
        }
    }

    pub fn try_push(&self, r: Record) -> Result<(), Record> {
        let mut q = self.q.lock().unwrap();
        if q.len() == self.cap {
            return Err(r);
        }
        q.push_back(r);
        Ok(())
    }

    pub fn try_pop(&self) -> Option<Record> {
        self.q.lock().unwrap().pop_front()
    }
}

/// Create `n` channels (core `i` uses channel `i`).
pub fn make_channels(n: usize, cap: usize) -> Vec<Arc<Chan>> {
    (0..n).map(|_| Arc::new(Chan::new(cap))).collect()
}

/// Wake-up primitive, same as sim_j: wakes may coalesce, sleep may be
/// spurious (mock carries a 1s safety timeout).
#[derive(Default)]
pub struct Waker {
    flag: Mutex<bool>,
    cv: Condvar,
}

impl Waker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn wake(&self) {
        *self.flag.lock().unwrap() = true;
        self.cv.notify_one();
    }

    pub fn sleep(&self) {
        let mut flag = self.flag.lock().unwrap();
        if !*flag {
            let (f, _) = self.cv.wait_timeout(flag, Duration::from_secs(1)).unwrap();
            flag = f;
        }
        *flag = false;
    }
}

// ===================== 作答區 =====================

/// Producer side: called once per record. **Must not block**; what happens
/// on full is your policy (`dropped` is yours to maintain).
pub fn produce(rec: Record, ch: &Chan, waker: &Waker, dropped: &AtomicU64) {
    todo!("彩排時實作")
}

/// Aggregator: sleep → wake → sweep every channel → write out (push into
/// `out`) → sleep. `budget` = max records taken from a **single** channel per
/// round — a hot core must not starve the cold ones (Phase 2). Once `stop`
/// is set, drain every channel before exiting.
pub fn aggregator_loop(
    chs: &[Arc<Chan>],
    waker: &Waker,
    stop: &AtomicBool,
    budget: usize,
    out: &Mutex<Vec<Record>>,
) {
    todo!("彩排時實作")
}
