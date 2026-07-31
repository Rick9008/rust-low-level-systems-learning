//! sim j【virtual onsite 準備題】—— sensor interrupt pipeline(題幹:`docs/interviews/sim-problems.md`)。
//!
//! 「題目給的介面」區一律英文(中文對照:`docs/interviews/sim-problems-zh.md`)。
//!
//! 彩排規則同 sim_i:實作+自寫測試在本檔;`tests/sim_j_isr_test.rs` 跑完才開。
//! ⚠ mock 的 Ring 用 Mutex 換單檔簡潔——**協定跟 spsc_ring 相同**(try 語意、固定容量),
//! 真上場把它換成你的 lock-free ring,你寫的 isr/worker 一行都不用動。

use std::collections::VecDeque;
#[cfg(test)]
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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
    // todo!("彩排時實作")

    if let Some(data) = fifo.read_fifo()
        && ring.try_push(data).is_err()
    {
        dropped.fetch_add(1, Ordering::Relaxed);
    }
    // ring: 3
    waker.wake();
    // ring: -empty waker wake and take one data so it's empty
    // ring: 4 5 6 7 8 9, assume log is really long
    while let Some(data) = fifo.read_fifo() {
        // 4 5 6 7 8 9 0
        if ring.try_push(data).is_err() {
            // data is 0, and ring is full
            dropped.fetch_add(1, Ordering::Relaxed);
            // dropped is 1
        }
    }
}

/// Worker: sleep → wake → drain the ring → log each sample (here: push into
/// `out`) → sleep again. Once `stop` is set, **drain what's left before
/// exiting** (Phase 2: clean shutdown).
pub fn worker_loop(ring: &Ring, waker: &Waker, stop: &AtomicBool, out: &Mutex<Vec<Sample>>) {
    // todo!("彩排時實作")
    let mut opt_log = ring.try_pop();
    while !stop.load(Ordering::Acquire) || opt_log.is_some() {
        if let Some(log) = opt_log.take() {
            out.lock().unwrap().push(log);
        }

        // Invariant: take will make opt_log into None
        opt_log = ring.try_pop();
        if opt_log.is_none() && !stop.load(Ordering::Acquire) {
            waker.sleep();
        }
        // after wake up we should take a opt_log, or the while loop condition might be thought as
        // empty
        if opt_log.is_none() {
            opt_log = ring.try_pop()
        }
    }
}

pub fn stop_fn(stop: &AtomicBool, waker: &Waker) {
    stop.store(true, Ordering::Release);
    waker.wake();
}

pub fn dropped_count(dropped: &AtomicU64) -> u64 {
    dropped.load(Ordering::Acquire)
}

// dry run status
// cap: 6
// isr fifo: 3 4 5 6 7 8 9 0
// worker_loop will consume the isr fifo, assume that log is really really long.
//
// after wake, isr will take out
// so in the final results 3 4 5 6 7 8 9 will log, and 0 is dropped.
// cnt is 1

#[test]
fn dry_run() {
    let drop_cnt = AtomicU64::new(0);
    let arc_ring = Arc::new(Ring::new(6));
    let arc_waker = Arc::new(Waker::new());
    let arc_ring_clone = arc_ring.clone();
    let arc_waker_clone = arc_waker.clone();
    let out = Arc::new(Mutex::new(Vec::new()));
    let out_clone = out.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let stop_cln = stop.clone();
    let worker = std::thread::spawn(move || {
        worker_loop(
            arc_ring_clone.as_ref(),
            arc_waker_clone.as_ref(),
            stop_cln.as_ref(),
            &out_clone,
        );
    });
    let mut hw_fifo = HwFifo::new();
    hw_fifo.push_burst([3, 4, 5, 6, 7, 8, 9, 0]);
    isr(&mut hw_fifo, arc_ring.as_ref(), &arc_waker, &drop_cnt);
    std::thread::sleep(Duration::from_millis(20));
    hw_fifo.push_burst([1, 2, 25]);
    isr(&mut hw_fifo, arc_ring.as_ref(), &arc_waker, &drop_cnt);
    stop_fn(&stop, &arc_waker);
    let _ = worker.join();
    assert!(arc_ring.q.lock().unwrap().is_empty());
    assert_eq!(
        dropped_count(&drop_cnt) + out.lock().unwrap().len() as u64,
        11
    );
}

#[test]
fn sleep_without_wake() {
    let drop_cnt = AtomicU64::new(0);
    let arc_ring = Arc::new(Ring::new(6));
    let arc_waker = Arc::new(Waker::new());
    let arc_ring_clone = arc_ring.clone();
    let arc_waker_clone = arc_waker.clone();
    let out = Arc::new(Mutex::new(Vec::new()));
    let out_clone = out.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let stop_cln = stop.clone();
    let worker = std::thread::spawn(move || {
        worker_loop(
            arc_ring_clone.as_ref(),
            arc_waker_clone.as_ref(),
            stop_cln.as_ref(),
            &out_clone,
        );
    });
    let mut hw_fifo = HwFifo::new();
    hw_fifo.push_burst([3, 4, 5, 6, 7, 8, 9, 0]);
    isr(&mut hw_fifo, arc_ring.as_ref(), &arc_waker, &drop_cnt);
    std::thread::sleep(Duration::from_millis(30));
    stop_fn(&stop, &arc_waker);
    let _ = worker.join();
}
