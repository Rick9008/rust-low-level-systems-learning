//! sim j【virtual onsite 準備題】—— sensor interrupt pipeline(題幹:`docs/interviews/sim-problems.md`)。
//!
//! 「題目給的介面」區一律英文(中文對照:`docs/interviews/sim-problems-zh.md`)。
//!
//! 彩排規則同 sim_i:實作+自寫測試在本檔;`tests/sim_j_isr_test.rs` 跑完才開。
//! ⚠ mock 的 Ring 用 Mutex 換單檔簡潔——**協定跟 spsc_ring 相同**(try 語意、固定容量),
//! 真上場把它換成你的 lock-free ring,你寫的 isr/worker 一行都不用動。

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Condvar, Mutex};
use std::time::Duration;

pub type Sample = u32;

// ===================== 題目給的介面(可讀)=====================

/// The hardware FIFO (ISR context only). Tests feed data into it.
#[derive(Default)]
pub struct HwFifo {
    q: VecDeque<Sample>,
}

impl HwFifo {
    pub fn new() -> Self {
        Self::default()
    }

    /// Test harness only: the hardware receives a burst of samples.
    pub fn push_burst(&mut self, samples: impl IntoIterator<Item = Sample>) {
        self.q.extend(samples);
    }

    /// Read one sample (ISR side); `None` = FIFO empty.
    pub fn read_fifo(&mut self) -> Option<Sample> {
        self.q.pop_front()
    }
}

/// Hand-off ring from the ISR to the worker. Fixed capacity, try-only —
/// the ISR must never wait.
pub struct Ring {
    cap: usize,
    q: Mutex<VecDeque<Sample>>,
}

impl Ring {
    pub fn new(cap: usize) -> Self {
        Self {
            cap,
            q: Mutex::new(VecDeque::new()),
        }
    }

    /// On full, the sample is handed back — the drop policy is the caller's call.
    pub fn try_push(&self, s: Sample) -> Result<(), Sample> {
        let mut q = self.q.lock().unwrap();
        if q.len() == self.cap {
            return Err(s);
        }
        q.push_back(s);
        Ok(())
    }

    pub fn try_pop(&self) -> Option<Sample> {
        self.q.lock().unwrap().pop_front()
    }
}

/// Wake-up primitive. `wake()` calls may coalesce (N calls, one wake-up) —
/// that behavior is part of the exam.
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

    /// Sleep until woken. May wake spuriously — waking up does not mean there is work.
    /// (The mock has a 1s safety timeout so a lost-wakeup bug can't hang your
    /// test run; the real thing has no such net.)
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

/// ISR: move samples from the hardware FIFO into the ring and wake the worker.
/// **No allocation, no blocking, no logging in here.** Decide your drop policy
/// first — the `dropped` counter is yours to maintain.
pub fn isr(fifo: &mut HwFifo, ring: &Ring, waker: &Waker, dropped: &AtomicU64) {
    todo!("彩排時實作")
}

/// Worker: sleep → wake → drain the ring → log each sample (here: push into
/// `out`) → sleep again. Once `stop` is set, **drain what's left before
/// exiting** (Phase 2: clean shutdown).
pub fn worker_loop(ring: &Ring, waker: &Waker, stop: &AtomicBool, out: &Mutex<Vec<Sample>>) {
    todo!("彩排時實作")
}
