//! sim k —— per-core telemetry fan-in(題幹:`docs/interviews/sim-problems.md`)。
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

/// 每核一條的 SPSC channel(mock)。producer 端只有 try——**producer 永不 block**。
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

/// 開 n 條 channel(核 i 用第 i 條)。
pub fn make_channels(n: usize, cap: usize) -> Vec<Arc<Chan>> {
    (0..n).map(|_| Arc::new(Chan::new(cap))).collect()
}

/// 喚醒原語,同 sim_j:wake 可合併、sleep 可能 spurious(mock 帶 1s 安全網)。
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

/// producer 端:每產一筆呼叫一次。**不准 block**;滿了怎麼辦是你的政策(dropped 給你)。
pub fn produce(rec: Record, ch: &Chan, waker: &Waker, dropped: &AtomicU64) {
    todo!("彩排時實作")
}

/// aggregator:睡 → 醒 → 掃所有 channel → 寫出(push 進 out)→ 再睡。
/// `budget` = 一輪從**單一條** channel 最多拿幾筆——防熱核餓死冷核(Phase 2)。
/// `stop` 立起後把所有 channel drain 乾淨再退。
pub fn aggregator_loop(
    chs: &[Arc<Chan>],
    waker: &Waker,
    stop: &AtomicBool,
    budget: usize,
    out: &Mutex<Vec<Record>>,
) {
    todo!("彩排時實作")
}
