//! 參考測試:pool_graceful_shutdown。
//!
//! 彩排時先自己寫測試(寫在 src/pool_graceful_shutdown.rs 底部);轉綠後才跑這組:
//! `cargo test -p rehearsals --test pool_graceful_shutdown_test -- --include-ignored`

use rehearsals::pool_graceful_shutdown::{Pool, Rejected};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

/// boundary:shutdown 必須等 in-flight 完成——任務確定已開跑(收到 started 訊號)
/// 才呼叫 shutdown;shutdown 回傳的那一刻,任務的副作用必須已可見。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn shutdown_waits_for_in_flight() {
    let pool = Pool::new(2);
    let done = Arc::new(AtomicUsize::new(0));
    let (started_tx, started_rx) = mpsc::channel();

    let d = Arc::clone(&done);
    pool.submit(move || {
        started_tx.send(()).unwrap();
        thread::sleep(Duration::from_millis(100)); // 模擬慢 health check
        d.fetch_add(1, Ordering::SeqCst);
    })
    .unwrap();

    started_rx.recv().unwrap(); // 任務已在 worker 上跑
    pool.shutdown();
    assert_eq!(
        done.load(Ordering::SeqCst),
        1,
        "shutdown 回傳前任務必須完成"
    );
}

/// boundary:shutdown 之後的 submit 一律拒絕。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn submit_after_shutdown_rejected() {
    let pool = Pool::new(1);
    pool.shutdown();
    assert_eq!(pool.submit(|| {}), Err(Rejected));
}

/// boundary:沒有任何任務時 shutdown 不能 hang——用 recv_timeout 當 watchdog,
/// hang 會變成明確的測試失敗而不是卡死。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn shutdown_with_no_tasks_returns() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let pool = Pool::new(4);
        pool.shutdown();
        tx.send(()).unwrap();
    });
    rx.recv_timeout(Duration::from_secs(5))
        .expect("無任務時 shutdown 不該 hang");
}

/// boundary:重複 shutdown——第二次(以及並發語境下的第 N 次)必須安全,
/// 不 panic、不 deadlock,任務也不會被執行兩次。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn repeated_shutdown_is_safe() {
    let pool = Pool::new(2);
    let done = Arc::new(AtomicUsize::new(0));
    let d = Arc::clone(&done);
    pool.submit(move || {
        d.fetch_add(1, Ordering::SeqCst);
    })
    .unwrap();

    pool.shutdown();
    pool.shutdown(); // 冪等
    assert_eq!(done.load(Ordering::SeqCst), 1);
    assert_eq!(pool.submit(|| {}), Err(Rejected));
}

/// 已接受 = 保證執行:排進 queue 還沒開跑的任務,graceful shutdown 也要跑完
/// (任務數 > worker 數,故 shutdown 當下必有任務還在排隊)。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn accepted_tasks_all_run() {
    let pool = Pool::new(2);
    let done = Arc::new(AtomicUsize::new(0));
    for _ in 0..16 {
        let d = Arc::clone(&done);
        pool.submit(move || {
            thread::sleep(Duration::from_millis(5));
            d.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();
    }
    pool.shutdown();
    assert_eq!(done.load(Ordering::SeqCst), 16, "已接受的任務一個都不能少");
}
