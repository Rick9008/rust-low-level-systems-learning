//! 參考測試:sim k(per-core telemetry fan-in)。
//!
//! 彩排完才開:
//! `cargo test -p rehearsals --test sim_k_fanin_test -- --include-ignored`

use rehearsals::sim_k_fanin::{Record, Waker, aggregator_loop, make_channels, produce};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// 守恆 + 每核順序:4 核 × 3000 筆,收到的 + 丟掉的 = 總數;
/// 單核的 seq 在輸出裡必須嚴格遞增(跨核不要求)。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn conservation_and_per_core_order() {
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

    // 等收斂:全部樣本要嘛送達要嘛計入 dropped。
    let total = CORES as u64 * PER_CORE;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let seen = out.lock().unwrap().len() as u64 + dropped.load(Ordering::SeqCst);
        if seen == total {
            break;
        }
        assert!(Instant::now() < deadline, "5 秒內沒收斂:守恆帳對不上");
        thread::sleep(Duration::from_millis(10));
    }
    stop.store(true, Ordering::SeqCst);
    waker.wake();
    agg.join().unwrap();

    let out = out.lock().unwrap();
    let mut last = [None::<u64>; CORES];
    for r in out.iter() {
        if let Some(prev) = last[r.core] {
            assert!(
                r.seq > prev,
                "核 {} 的順序破了:{} 之後出現 {}",
                r.core,
                prev,
                r.seq
            );
        }
        last[r.core] = Some(r.seq);
    }
}

/// shutdown drain:stop 立起時 channel 有殘貨,aggregator 必須清完才退。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn shutdown_drains_all_channels() {
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
    assert_eq!(seen, 30, "stop 後殘貨必須 drain 完,不准蒸發");
}
