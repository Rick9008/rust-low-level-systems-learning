// ⚠⚠ 防雷:本檔是 sim j(sensor interrupt pipeline)的解法。計時場排在 7/31——
// 在那之前不要讀本檔(含註解與測試)。自定規則:跑題前不開 oracle/sol。

//! # isr_pipeline —— ISR → bottom-half pipeline(sim j 教學版)
//!
//! ## [Clarify]
//! 題幹:`docs/interviews/sim-problems.md` sim j(彩排 harness:`rehearsals/src/sim_j_isr.rs`)。
//! 感測器中斷高速湧入,ISR 收、worker 執行緒處理。隱藏 spec:
//! - **ISR 三禁**:不 alloc、不 block、不 log——ISR 裡只准「搬 + 計數 + 叫醒」;
//! - ring 滿了怎麼辦是你的 drop policy,但**丟多少要可觀測**(dropped 計數);
//! - `wake()` 會合併(N 次呼叫一次喚醒)——worker 不能假設「一醒一事」;
//! - shutdown(Phase 2):stop 是「別再睡」,不是「丟掉手上的貨」——殘貨要 drain 完才退。
//!
//! ## [Abstract]
//! 與 [`crate::concurrency::signal_pipeline`] 同一張 JD 本尊圖,差別在包裝:
//! 這裡是 spec-heavy 版(題目給定 `HwFifo`/`Ring`/`Waker`,考你 isr/worker 的紀律),
//! 那裡是手搓版(考你 ring 與握手本身)。三個正交決策:
//! 1. 交棒結構:try-only 固定容量 ring(mock 用 Mutex 換單檔簡潔,協定同 spsc_ring);
//! 2. full policy:**drop-newest + 計數**——try_push 拒新;要 drop-oldest 得 ring
//!    自己支援(consumer 單寫的 head,producer 動不了),說得出這個限制是加分點;
//! 3. 喚醒:sticky flag(先貼「有事發生過」的貼紙再 notify),吃掉合併與 spurious。
//!
//! ## [Iterate]
//! naive:worker 忙輪詢(100% CPU)→ 睡 condvar:ISR 在「drain 完 → 睡」的縫隙
//! 插隊就 lost wakeup → **sticky flag**:wake() 貼貼紙、sleep() 先看貼紙再睡,
//! 縫隙裡來的事貼紙還在,sleep 立刻返回。
//!
//! ## [Trade-offs]
//! - ISR 端 O(1) 收尾:drop-newest 讓最壞情況也只是「試一下、失敗、+1」;
//!   任何「等一下就有位子」的想法都是在 ISR 裡 block,直接紅牌。
//! - worker 批次 drain 再一次鎖寫出(不逐筆搶 out 的鎖)——鎖次數 O(波數) 而非 O(樣本數)。
//! - shutdown 協定:立 stop 的人**先 flag 後 wake**;worker 看到 stop 後再 drain
//!   最後一輪才退。順序反了(先 wake 後 flag)worker 可能醒來沒看到 stop 又睡回去。
//!
//! ## [Dry-Run]
//! 測試 `shutdown_drain_no_residue` 的手 trace:先塞 50 筆(worker 還沒起)→
//! 起 worker → 立刻 stop+wake → worker 迴圈頂 drain 到殘貨 → 看到 stop →
//! 最後一輪 drain(此時已空)→ 退。out+dropped == 50,一筆不蒸發。
//!
//! 對照:彩排解答 `rehearsals/examples/sol_sim_j_isr.rs`(同設計,單檔面試版)。

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::Duration;

pub type Sample = u32;

// ===================== 題目給的介面(與 sim j 相同,英文保留)=====================

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
/// the ISR must never wait.(mock 用 Mutex 換簡潔;協定同 `spsc_ring`,
/// 換成 lock-free 版時 isr/worker 一行不用動。)
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

/// Wake-up primitive. `wake()` calls may coalesce (N calls, one wake-up)。
/// sticky flag:貼紙語意——wake 先貼、sleep 先看再睡,lost-wakeup 免疫。
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

    /// Sleep until woken. May wake spuriously — waking up does not mean there
    /// is work.(mock 帶 1s 安全網,lost-wakeup bug 不會吊死測試;真硬體沒有。)
    pub fn sleep(&self) {
        let mut flag = self.flag.lock().unwrap();
        if !*flag {
            let (f, _) = self.cv.wait_timeout(flag, Duration::from_secs(1)).unwrap();
            flag = f;
        }
        *flag = false;
    }
}

// ===================== 實作 =====================

/// ISR:drain 硬體 FIFO → ring;滿了 **drop-newest + 計數**;最後叫醒一次。
/// 三禁之下的最小形狀:搬(O(1) per sample)、計數(Relaxed 就夠——只是統計,
/// 不同步任何資料)、叫醒(一波一次,合併友好)。
pub fn isr(fifo: &mut HwFifo, ring: &Ring, waker: &Waker, dropped: &AtomicU64) {
    while let Some(s) = fifo.read_fifo() {
        if ring.try_push(s).is_err() {
            dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
    waker.wake();
}

/// worker:drain → 看 stop(要退先清殘貨)→ 睡;醒了不代表有事,回圈頂再 drain。
/// 批次收、一次鎖寫出——鎖次數 O(波數) 而非 O(樣本數)。
pub fn worker_loop(ring: &Ring, waker: &Waker, stop: &AtomicBool, out: &Mutex<Vec<Sample>>) {
    loop {
        let mut batch = Vec::new();
        while let Some(s) = ring.try_pop() {
            batch.push(s);
        }
        if !batch.is_empty() {
            out.lock().unwrap().extend(batch);
        }
        if stop.load(Ordering::SeqCst) {
            // stop 後最後一輪 drain:「先 flag 後 wake」的協定下,殘貨保證不蒸發。
            let mut rest = Vec::new();
            while let Some(s) = ring.try_pop() {
                rest.push(s);
            }
            if !rest.is_empty() {
                out.lock().unwrap().extend(rest);
            }
            return;
        }
        waker.sleep();
    }
}

// ===================== 測試 =====================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Instant;

    /// 單執行緒、全確定:cap=4 的 ring 塞 10 筆 → 前 4 筆進 ring(FIFO 序),
    /// 後 6 筆 drop-newest、dropped == 6。手 trace:try_push(0..=3) Ok、
    /// try_push(4..=9) Err → fetch_add ×6。驗 ISR 的 full policy 與可觀測性。
    #[test]
    fn drop_newest_when_full_and_counted() {
        let ring = Ring::new(4);
        let waker = Waker::new();
        let dropped = AtomicU64::new(0);
        let mut fifo = HwFifo::new();
        fifo.push_burst(0..10);
        isr(&mut fifo, &ring, &waker, &dropped);
        assert_eq!(dropped.load(Ordering::Relaxed), 6);
        let mut kept = Vec::new();
        while let Some(s) = ring.try_pop() {
            kept.push(s);
        }
        assert_eq!(kept, vec![0, 1, 2, 3]);
    }

    /// sticky flag 的貼紙語意(lost-wakeup 免疫的根基):
    /// wake 先到、sleep 後到 → sleep 立刻返回(不等 1s 安全網)。
    #[test]
    fn wake_before_sleep_is_not_lost() {
        let waker = Waker::new();
        waker.wake();
        let t0 = Instant::now();
        waker.sleep();
        assert!(
            t0.elapsed() < Duration::from_millis(500),
            "貼紙丟了,睡到安全網才醒"
        );
    }

    /// 守恆 + 順序(threaded):50 波 × 100 筆,out + dropped == 5000,
    /// 且 out 嚴格遞增(SPSC 一產一消 → per-source FIFO 保序)。
    #[test]
    fn conservation_and_order_across_bursts() {
        let ring = Arc::new(Ring::new(64));
        let waker = Arc::new(Waker::new());
        let stop = Arc::new(AtomicBool::new(false));
        let out = Arc::new(Mutex::new(Vec::new()));
        let dropped = Arc::new(AtomicU64::new(0));
        let worker = {
            let (r, w, s, o) = (ring.clone(), waker.clone(), stop.clone(), out.clone());
            thread::spawn(move || worker_loop(&r, &w, &s, &o))
        };
        let mut fifo = HwFifo::new();
        for burst in 0..50u32 {
            fifo.push_burst(burst * 100..(burst + 1) * 100);
            isr(&mut fifo, &ring, &waker, &dropped);
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let seen = out.lock().unwrap().len() as u64 + dropped.load(Ordering::SeqCst);
            if seen == 5_000 {
                break;
            }
            assert!(Instant::now() < deadline, "沒收斂:掉樣本或 worker 睡死");
            thread::sleep(Duration::from_millis(10));
        }
        stop.store(true, Ordering::SeqCst);
        waker.wake();
        worker.join().unwrap();
        assert!(out.lock().unwrap().windows(2).all(|w| w[0] < w[1]), "亂序");
    }

    /// 檔頭 [Dry-Run] 的劇本:殘貨 shutdown。先塞 50 筆(worker 還沒起)→
    /// 起 worker → 立刻 stop(先 flag)+ wake(後叫)→ worker drain 完才退,
    /// out + dropped == 50——stop 不是「丟掉手上的貨」。
    #[test]
    fn shutdown_drain_no_residue() {
        let ring = Arc::new(Ring::new(64));
        let waker = Arc::new(Waker::new());
        let stop = Arc::new(AtomicBool::new(false));
        let out = Arc::new(Mutex::new(Vec::new()));
        let dropped = Arc::new(AtomicU64::new(0));
        let mut fifo = HwFifo::new();
        fifo.push_burst(0..50);
        isr(&mut fifo, &ring, &waker, &dropped);
        let worker = {
            let (r, w, s, o) = (ring.clone(), waker.clone(), stop.clone(), out.clone());
            thread::spawn(move || worker_loop(&r, &w, &s, &o))
        };
        stop.store(true, Ordering::SeqCst);
        waker.wake();
        worker.join().unwrap();
        let seen = out.lock().unwrap().len() as u64 + dropped.load(Ordering::SeqCst);
        assert_eq!(seen, 50);
    }
}
