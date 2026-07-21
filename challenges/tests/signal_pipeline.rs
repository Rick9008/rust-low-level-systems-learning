//! challenge 驗收測試:signal_pipeline。
//! `cargo test -p challenges --test signal_pipeline -- --include-ignored`

use challenges::concurrency::signal_pipeline::{Signal, start};
use std::time::Duration;

/// 守恆:狂灌 100k 筆(capacity 8)——accepted + dropped == sent,
/// 聚合 count == accepted(掉的每一筆都被數到,沒有黑洞)。
#[test]
#[ignore = "challenge:實作完成後再開"]
fn conservation_under_burst() {
    let (mut tx, handle) = start(8);
    const N: u64 = 100_000;
    let mut accepted = 0u64;
    for _ in 0..N {
        if tx.send(Signal {
            sensor_id: 1,
            value: 1,
        }) {
            accepted += 1;
        }
    }
    let dropped = tx.dropped();
    drop(tx);
    let stats = handle.shutdown();
    assert_eq!(accepted + dropped, N);
    assert_eq!(stats.count, accepted);
    assert_eq!(stats.sum, accepted as i64);
}

/// park 路徑:consumer 睡著後,下一筆訊號必須叫醒它。
/// lost wakeup(掛牌握手寫錯)會讓 shutdown 的 join 卡死——
/// 測試以 hang 的形式顯性失敗,不會靜默漏資料。
#[test]
#[ignore = "challenge:實作完成後再開"]
fn wakes_parked_consumer() {
    let (mut tx, handle) = start(8);
    assert!(tx.send(Signal {
        sensor_id: 1,
        value: 10,
    }));
    std::thread::sleep(Duration::from_millis(50)); // 讓 consumer 睡著
    assert!(tx.send(Signal {
        sensor_id: 2,
        value: 32,
    }));
    std::thread::sleep(Duration::from_millis(50));
    drop(tx);
    let stats = handle.shutdown();
    assert_eq!(stats.count, 2);
    assert_eq!(stats.sum, 42);
    assert_eq!((stats.min, stats.max), (10, 32));
}

/// shutdown drain:push 完立刻 shutdown,殘料一筆不少。
#[test]
#[ignore = "challenge:實作完成後再開"]
fn shutdown_drains_backlog() {
    let (mut tx, handle) = start(1024);
    let mut accepted = 0u64;
    for i in 0..500 {
        if tx.send(Signal {
            sensor_id: 0,
            value: i,
        }) {
            accepted += 1;
        }
    }
    drop(tx);
    let stats = handle.shutdown();
    assert_eq!(stats.count, accepted);
}
