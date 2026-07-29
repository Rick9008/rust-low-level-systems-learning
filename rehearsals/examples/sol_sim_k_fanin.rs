//! solution:sim k —— per-core telemetry fan-in。**寫完彩排才開。**
//!
//! 三個決定:
//! 1. producer 永不 block:try_push 失敗 → dropped 計數,一行收工;每筆 wake(flag 合併,不貴)。
//! 2. aggregator 的公平性:每輪對**單一條** channel 最多拿 `budget` 筆,輪完所有 channel
//!    再回頭——熱核塞爆自己的 ring 也搶不走冷核的輪次。
//! 3. 睡前條件:一整輪掃過去「零收穫」才准睡(N 條 ring 全空的那一刻);Waker flag 補住
//!    掃完→睡之間新來的貨。
//!
//! 驗證:`cargo run -p rehearsals --example sol_sim_k_fanin`

use rehearsals::sim_k_fanin::{Chan, Record, Waker, make_channels};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// producer 端:try_push,滿了計數;喚醒交給 flag 合併。
fn produce(rec: Record, ch: &Chan, waker: &Waker, dropped: &AtomicU64) {
    if ch.try_push(rec).is_err() {
        dropped.fetch_add(1, Ordering::Relaxed);
    }
    waker.wake();
}

/// aggregator:round-robin + per-ring budget;全空一輪才睡;stop 前把貨清完。
fn aggregator_loop(
    chs: &[Arc<Chan>],
    waker: &Waker,
    stop: &AtomicBool,
    budget: usize,
    out: &Mutex<Vec<Record>>,
) {
    loop {
        // 掃到一整輪零收穫為止——這一刻所有 ring 同時是空的。
        loop {
            let mut moved = false;
            for ch in chs {
                let mut batch = Vec::new();
                while batch.len() < budget {
                    match ch.try_pop() {
                        Some(r) => batch.push(r),
                        None => break,
                    }
                }
                if !batch.is_empty() {
                    moved = true;
                    out.lock().unwrap().extend(batch);
                }
            }
            if !moved {
                break;
            }
        }
        if stop.load(Ordering::SeqCst) {
            return; // 上面剛 drain 到全空,殘貨已清
        }
        waker.sleep();
    }
}

fn main() {
    // scenario 1:守恆 + 每核順序(4 核 × 3000)。
    const CORES: usize = 4;
    const PER_CORE: u64 = 3000;
    let chs = make_channels(CORES, 256);
    let waker = Arc::new(Waker::new());
    let stop = Arc::new(AtomicBool::new(false));
    let out = Arc::new(Mutex::new(Vec::new()));
    let dropped = Arc::new(AtomicU64::new(0));
    let agg = {
        let (c, w, s, o) = (chs.clone(), waker.clone(), stop.clone(), out.clone());
        thread::spawn(move || aggregator_loop(&c, &w, &s, 64, &o))
    };
    let producers: Vec<_> = (0..CORES)
        .map(|core| {
            let ch = chs[core].clone();
            let (w, d) = (waker.clone(), dropped.clone());
            thread::spawn(move || {
                for seq in 0..PER_CORE {
                    produce(Record { core, seq }, &ch, &w, &d);
                }
            })
        })
        .collect();
    for p in producers {
        p.join().unwrap();
    }
    let total = CORES as u64 * PER_CORE;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let seen = out.lock().unwrap().len() as u64 + dropped.load(Ordering::SeqCst);
        if seen == total {
            break;
        }
        assert!(Instant::now() < deadline, "沒收斂");
        thread::sleep(Duration::from_millis(10));
    }
    stop.store(true, Ordering::SeqCst);
    waker.wake();
    agg.join().unwrap();
    {
        let out = out.lock().unwrap();
        let mut last = [None::<u64>; CORES];
        for r in out.iter() {
            if let Some(prev) = last[r.core] {
                assert!(r.seq > prev, "核 {} 順序破了", r.core);
            }
            last[r.core] = Some(r.seq);
        }
    }

    // scenario 2:shutdown drain。
    let chs = make_channels(3, 64);
    let waker = Arc::new(Waker::new());
    let stop = Arc::new(AtomicBool::new(false));
    let out = Arc::new(Mutex::new(Vec::new()));
    let dropped = Arc::new(AtomicU64::new(0));
    for (core, ch) in chs.iter().enumerate() {
        for seq in 0..10 {
            produce(Record { core, seq }, ch, &waker, &dropped);
        }
    }
    let agg = {
        let (c, w, s, o) = (chs.clone(), waker.clone(), stop.clone(), out.clone());
        thread::spawn(move || aggregator_loop(&c, &w, &s, 4, &o))
    };
    stop.store(true, Ordering::SeqCst);
    waker.wake();
    agg.join().unwrap();
    let seen = out.lock().unwrap().len() as u64 + dropped.load(Ordering::SeqCst);
    assert_eq!(seen, 30);

    println!("sol_sim_k_fanin: all green");
}
