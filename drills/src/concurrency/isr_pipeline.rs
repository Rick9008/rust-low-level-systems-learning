// ⚠⚠ 防雷:本檔是 sim j(sensor interrupt pipeline)的填空版,spec 註解含解法方向。
// 計時場排在 7/31——在那之前不要讀。自定規則:跑題前不開 oracle/sol。

//! drill:isr_pipeline —— 填 isr 與 worker_loop(sim j 的填空版)。
//!
//! 已給:`HwFifo` / `Ring` / `Waker`(借 reference 的,題目給定件)。
//! 要填:`isr` / `worker_loop` 兩個函式。
//!
//! 核心不變量:
//! - ISR 三禁(不 alloc、不 block、不 log)——裡面只准「搬 + 計數 + 叫醒」;
//! - `wake()` 會合併、`sleep()` 會 spurious——worker 醒了不代表有事,回圈頂重新 drain;
//! - stop 是「別再睡」不是「丟貨」——看到 stop 後要把殘貨 drain 完才退。
//!
//! 設計取捨見 reference 同名模組檔頭(**跑過 sim j 之後**再讀)。

use reference::concurrency::isr_pipeline::{HwFifo, Ring, Sample, Waker};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// spec:ISR——把硬體 FIFO 搬空進 ring;ring 滿了 **drop-newest + `dropped` 計數**
/// (Relaxed 就夠,只是統計);整波搬完**叫醒一次**(合併友好)。
/// 三禁之下不准出現:配置、等待、逐筆 wake。
pub fn isr(fifo: &mut HwFifo, ring: &Ring, waker: &Waker, dropped: &AtomicU64) {
    todo!("spec: 搬空 FIFO;滿了 drop-newest+計數;最後 wake 一次")
}

/// spec:worker——迴圈:drain ring(批次收,一次鎖寫進 `out`)→ 查 `stop`
/// (是 → **最後一輪 drain 殘貨**再 return)→ `waker.sleep()`。
/// 醒了不代表有事;drain 放迴圈頂,睡醒自然重掃。
pub fn worker_loop(ring: &Ring, waker: &Waker, stop: &AtomicBool, out: &Mutex<Vec<Sample>>) {
    todo!("spec: drain → 查 stop(退前清殘貨)→ sleep;批次寫出")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    /// cap=4 塞 10 筆:前 4 筆保序進 ring,後 6 筆 drop-newest、dropped == 6。
    #[test]
    #[ignore = "drill:填完 isr 後拔掉"]
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

    /// 守恆 + 順序:50 波 × 100 筆,out + dropped == 5000 且 out 嚴格遞增。
    #[test]
    #[ignore = "drill:填完 isr/worker_loop 後拔掉"]
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

    /// shutdown drain:worker 起來前先塞 50 筆 → stop+wake → 殘貨不准蒸發。
    #[test]
    #[ignore = "drill:填完 worker_loop 後拔掉"]
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
