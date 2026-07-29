//! solution:sim j —— sensor interrupt pipeline。**寫完彩排才開。**
//!
//! 兩條鐵律:
//! 1. ISR 只做「搬 + 計數 + 叫醒」——不 alloc、不 block、不 log;滿了 drop-newest + 計數,
//!    讓 ISR 永遠 O(1) 收尾(ring 的 try_push 拒新,要 drop-oldest 得 ring 自己支援——講出這個限制)。
//! 2. lost-wakeup 免疫靠 Waker 的 flag:wake() 貼「有事發生過」的貼紙,sleep() 先看貼紙再睡。
//!    worker 在「drain 完 → 睡」的縫隙被 ISR 插隊也不漏:貼紙還在,sleep 立刻返回。
//!
//! shutdown 協定:立 stop 的人要 **先 flag 後 wake**;worker 看到 stop 後再 drain 最後一輪才退
//! ——stop 是「別再睡」,不是「丟掉手上的貨」。
//!
//! 驗證:`cargo run -p rehearsals --example sol_sim_j_isr`

use rehearsals::sim_j_isr::{HwFifo, Ring, Waker};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// ISR:drain 硬體 FIFO → ring;滿了 drop-newest + 計數;最後叫醒一次(合併友好)。
fn isr(fifo: &mut HwFifo, ring: &Ring, waker: &Waker, dropped: &AtomicU64) {
    while let Some(s) = fifo.read_fifo() {
        if ring.try_push(s).is_err() {
            dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
    waker.wake();
}

/// worker:drain → 看 stop(要退先清殘貨)→ 睡;醒了不代表有事,回圈頂再 drain。
fn worker_loop(ring: &Ring, waker: &Waker, stop: &AtomicBool, out: &Mutex<Vec<u32>>) {
    loop {
        let mut batch = Vec::new();
        while let Some(s) = ring.try_pop() {
            batch.push(s);
        }
        if !batch.is_empty() {
            out.lock().unwrap().extend(batch); // 鎖一次收一批,不逐筆搶鎖
        }
        if stop.load(Ordering::SeqCst) {
            // stop 後最後一輪 drain:先 flag 後 wake 的協定下,這裡保證殘貨不蒸發。
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

fn main() {
    // scenario 1:守恆 + 順序(100 波 × 100 筆)。
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
    for burst in 0..100u32 {
        fifo.push_burst(burst * 100..(burst + 1) * 100);
        isr(&mut fifo, &ring, &waker, &dropped);
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let seen = out.lock().unwrap().len() as u64 + dropped.load(Ordering::SeqCst);
        if seen == 10_000 {
            break;
        }
        assert!(Instant::now() < deadline, "沒收斂");
        thread::sleep(Duration::from_millis(10));
    }
    stop.store(true, Ordering::SeqCst);
    waker.wake();
    worker.join().unwrap();
    assert!(out.lock().unwrap().windows(2).all(|w| w[0] < w[1]));

    // scenario 2:shutdown drain——先塞貨、開 worker、立刻 stop,殘貨不准蒸發。
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

    println!("sol_sim_j_isr: all green");
}
