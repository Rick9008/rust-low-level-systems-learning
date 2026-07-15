//! 參考測試:bounded_channel。
//!
//! 彩排時先自己寫測試;轉綠後才跑這組:
//! `cargo test -p rehearsals --test bounded_channel_test -- --include-ignored`

use rehearsals::bounded_channel::channel;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// FIFO 基本盤:不滿不空的順路徑。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn fifo_happy_path() {
    let (tx, rx) = channel(4);
    tx.send(1).unwrap();
    tx.send(2).unwrap();
    tx.send(3).unwrap();
    assert_eq!(rx.recv(), Some(1));
    assert_eq!(rx.recv(), Some(2));
    assert_eq!(rx.recv(), Some(3));
}

/// boundary:滿了要 block——recv 騰出空位後才放行(capacity 1 最嚴格)。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn send_blocks_when_full_until_recv() {
    let (tx, rx) = channel(1);
    tx.send(1).unwrap(); // 滿

    let (probe_tx, probe_rx) = mpsc::channel();
    let h = thread::spawn(move || {
        tx.send(2).unwrap(); // 必須 block 在這
        probe_tx.send(()).unwrap();
    });

    assert!(
        probe_rx.recv_timeout(Duration::from_millis(200)).is_err(),
        "channel 滿,send 不該立刻完成"
    );
    assert_eq!(rx.recv(), Some(1)); // 騰出空位
    probe_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("recv 之後,block 中的 send 要被放行");
    assert_eq!(rx.recv(), Some(2));
    h.join().unwrap();
}

/// boundary:空了要 block——sender 送來才醒。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn recv_blocks_when_empty_until_send() {
    let (tx, rx) = channel(2);
    let h = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        tx.send(9).unwrap();
    });
    assert_eq!(rx.recv(), Some(9)); // 這行必須 block 到 send 發生
    h.join().unwrap();
}

/// boundary:全部 sender drop——buffer 先清空,然後 recv 回 None(不能 hang)。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn senders_dropped_drain_then_none() {
    let (tx, rx) = channel(4);
    let tx2 = tx.clone();
    tx.send(1).unwrap();
    tx2.send(2).unwrap();
    drop(tx);
    drop(tx2);
    assert_eq!(rx.recv(), Some(1), "斷線前送進去的要先吐完");
    assert_eq!(rx.recv(), Some(2));
    assert_eq!(rx.recv(), None, "空了且無 sender → None,不是 hang");
}

/// boundary:receiver drop 後 send → Err,值原封歸還;
/// block 中的 sender 也要被 receiver 的 drop 叫醒。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn receiver_dropped_send_errors_and_unblocks() {
    let (tx, rx) = channel(1);
    drop(rx);
    assert_eq!(tx.send(7).unwrap_err().0, 7, "值要原封還你");

    // block 中的 sender 被 drop(rx) 叫醒
    let (tx, rx) = channel(1);
    tx.send(1).unwrap(); // 滿
    let h = thread::spawn(move || tx.send(2)); // block
    thread::sleep(Duration::from_millis(100));
    drop(rx);
    let res = h.join().unwrap();
    assert_eq!(
        res.unwrap_err().0,
        2,
        "receiver 消失要叫醒 block 中的 sender"
    );
}

/// 多生產者:兩個 sender 並發灌,總量與內容都不丟不重。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn multi_producer_no_loss() {
    let (tx, rx) = channel(8);
    let tx2 = tx.clone();
    let h1 = thread::spawn(move || {
        for i in 0..100u32 {
            tx.send(i).unwrap();
        }
    });
    let h2 = thread::spawn(move || {
        for i in 100..200u32 {
            tx2.send(i).unwrap();
        }
    });
    let mut got = Vec::new();
    while let Some(v) = rx.recv() {
        got.push(v);
    }
    h1.join().unwrap();
    h2.join().unwrap();
    got.sort_unstable();
    assert_eq!(got, (0..200).collect::<Vec<_>>());
}
