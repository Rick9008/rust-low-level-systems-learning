//! 參考測試:sim j(sensor interrupt pipeline)。
//!
//! 彩排完才開:
//! `cargo test -p rehearsals --test sim_j_isr_test -- --include-ignored`

use rehearsals::sim_j_isr::{HwFifo, Ring, Waker, isr, worker_loop};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn setup() -> (
    Arc<Ring>,
    Arc<Waker>,
    Arc<AtomicBool>,
    Arc<Mutex<Vec<u32>>>,
    Arc<AtomicU64>,
) {
    (
        Arc::new(Ring::new(64)),
        Arc::new(Waker::new()),
        Arc::new(AtomicBool::new(false)),
        Arc::new(Mutex::new(Vec::new())),
        Arc::new(AtomicU64::new(0)),
    )
}

/// 守恆 + 順序:100 波 × 100 筆,收到的 + 丟掉的 = 總數;收到的嚴格遞增(不重複、不亂序)。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn conservation_and_order() {
    let (ring, waker, stop, out, dropped) = setup();
    let worker = {
        let (r, w, s, o) = (ring.clone(), waker.clone(), stop.clone(), out.clone());
        thread::spawn(move || worker_loop(&r, &w, &s, &o))
    };

    let total: u64 = 10_000;
    let mut fifo = HwFifo::new();
    for burst in 0..100u32 {
        fifo.push_burst(burst * 100..(burst + 1) * 100);
        isr(&mut fifo, &ring, &waker, &dropped);
    }

    // 等 pipeline 靜下來(全部樣本要嘛送達要嘛計入 dropped)。
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let seen = out.lock().unwrap().len() as u64 + dropped.load(Ordering::SeqCst);
        if seen == total {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "5 秒內沒收斂:守恆帳對不上(漏樣本或 lost wakeup)"
        );
        thread::sleep(Duration::from_millis(10));
    }
    stop.store(true, Ordering::SeqCst);
    waker.wake();
    worker.join().unwrap();

    let out = out.lock().unwrap();
    assert!(
        out.windows(2).all(|w| w[0] < w[1]),
        "順序破了:收到的樣本必須嚴格遞增"
    );
}

/// Phase 2 clean shutdown:stop 立起時 ring 裡還有貨,worker 必須 drain 乾淨才退。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn shutdown_drains_ring() {
    let (ring, waker, stop, out, dropped) = setup();

    // 先塞貨、再開 worker、立刻 stop——考「stop 不等於丟資料」。
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
    assert_eq!(
        seen, 50,
        "stop 後 ring 裡的殘貨必須 drain 完(或計入 dropped),不准蒸發"
    );
}
