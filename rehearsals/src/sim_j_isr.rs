//! sim j —— sensor interrupt pipeline(題幹:`docs/interviews/sim-problems.md`)。
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

/// 硬體 FIFO(ISR context 才准碰)。測試會灌資料進來。
#[derive(Default)]
pub struct HwFifo {
    q: VecDeque<Sample>,
}

impl HwFifo {
    pub fn new() -> Self {
        Self::default()
    }

    /// 測試腳本用:模擬硬體收進一批樣本。
    pub fn push_burst(&mut self, samples: impl IntoIterator<Item = Sample>) {
        self.q.extend(samples);
    }

    /// ISR 端讀一筆;`None` = FIFO 空。
    pub fn read_fifo(&mut self) -> Option<Sample> {
        self.q.pop_front()
    }
}

/// ISR → worker 的交棒 ring。固定容量、只有 try 語意——ISR 不准等。
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

    /// 滿了把樣本原樣還你——drop 政策由呼叫端決定,ring 不替你做主。
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

/// 喚醒原語。`wake()` 可以合併(連按多次只醒一次)——這正是考點之一。
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

    /// 睡到被叫醒。可能 spurious 提前醒——醒了不代表有事。
    /// (mock 帶 1s 安全網,防 lost-wakeup 的 bug 把測試吊死;真上場沒有這條保險。)
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

/// ISR:把硬體 FIFO 搬進 ring、叫醒 worker。**不准 alloc/block/log。**
/// ring 滿了怎麼辦——先想好你的 drop 政策(dropped 計數器給你用)。
pub fn isr(fifo: &mut HwFifo, ring: &Ring, waker: &Waker, dropped: &AtomicU64) {
    todo!("彩排時實作")
}

/// worker:睡 → 醒 → drain ring → 逐筆 log(這裡 = push 進 out)→ 再睡。
/// `stop` 立起後要把 ring 裡剩的 **drain 乾淨再退**(Phase 2 的 clean shutdown)。
pub fn worker_loop(ring: &Ring, waker: &Waker, stop: &AtomicBool, out: &Mutex<Vec<Sample>>) {
    todo!("彩排時實作")
}
