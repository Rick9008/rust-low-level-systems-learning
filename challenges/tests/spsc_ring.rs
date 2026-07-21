//! 驗收:challenges::concurrency::spsc_ring。完成後移除 #[ignore]。

use challenges::concurrency::spsc_ring::channel;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

/// boundary:滿/空/歸還。
#[test]
fn full_empty_and_return() {
    let (mut tx, mut rx) = channel(2);
    tx.push(1).unwrap();
    tx.push(2).unwrap();
    // 容量可能被你上調(例如到 2 的冪),所以塞到 Err 為止
    let mut extra = 3;
    let overflow = loop {
        match tx.push(extra) {
            Ok(()) => extra += 1,
            Err(back) => break back,
        }
    };
    assert_eq!(overflow, extra); // 滿時歸還的是同一個元素
    assert_eq!(rx.pop(), Some(1));
    assert_eq!(rx.pop(), Some(2));
}

/// boundary:空 pop 回 None(不阻塞、不 panic)。
#[test]
fn empty_pop_is_none() {
    let (_tx, mut rx) = channel::<u64>(4);
    assert_eq!(rx.pop(), None);
}

/// boundary:多輪 wrap——push/pop 交替次數遠超容量。
#[test]
fn wrap_many_rounds() {
    let (mut tx, mut rx) = channel(2);
    for i in 0..1000 {
        tx.push(i).unwrap();
        assert_eq!(rx.pop(), Some(i));
    }
    assert_eq!(rx.pop(), None);
}

/// 核心驗收:兩執行緒 10 萬元素,順序不亂、一個不少。
#[test]
fn two_thread_ordered_delivery() {
    const N: u64 = 100_000;
    let (mut tx, mut rx) = channel(8);
    let producer = thread::spawn(move || {
        for i in 0..N {
            let mut item = i;
            while let Err(back) = tx.push(item) {
                item = back;
                thread::yield_now();
            }
        }
    });
    let mut expect = 0;
    while expect < N {
        match rx.pop() {
            Some(v) => {
                assert_eq!(v, expect, "FIFO 順序錯亂");
                expect += 1;
            }
            None => thread::yield_now(),
        }
    }
    producer.join().unwrap();
    assert_eq!(rx.pop(), None);
}

#[derive(Debug)]
struct DropSpy(Arc<AtomicUsize>);
impl Drop for DropSpy {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

/// 容量邊界 + 環狀重用:塞到滿 → FIFO 全取出 → 排空後還能再塞。
/// cap 用 2 的冪,避開 #1,單驗 push/pop 的重用邏輯。現在應該綠。
#[test]
fn fill_to_capacity_then_reuse() {
    let (mut tx, mut rx) = channel(4);
    let mut pushed = 0u64;
    while tx.push(pushed).is_ok() {
        pushed += 1;
    }
    assert!(pushed >= 4, "容量至少 4:實際塞進 {pushed}");
    for i in 0..pushed {
        assert_eq!(rx.pop(), Some(i), "FIFO 應保序");
    }
    assert_eq!(rx.pop(), None);
    tx.push(999).unwrap(); // 排空後環狀重用
    assert_eq!(rx.pop(), Some(999));
}

/// 🔴 抓 #2:帶著未消費元素被 drop,Drop 要把它們全數回收、剛好一次。
/// 推 3、消費 1、剩 2 在環裡;channel 離開 scope 後總 drop 次數該 = 3
/// (1 消費 + 2 被 SpscRing::drop 回收)。
/// 目前你的 Drop 方向錯 → 會走進未初始化槽 → **UB,很可能直接崩(SIGSEGV)**。
/// 那個崩就是紅燈。把 Drop 改成「從 head 走到 tail」後應該剛好 = 3。
#[test]
fn drop_reclaims_unconsumed() {
    let n = Arc::new(AtomicUsize::new(0));
    {
        let (mut tx, mut rx) = channel(4);
        for _ in 0..3 {
            tx.push(DropSpy(n.clone())).unwrap();
        }
        drop(rx.pop()); // 消費 1 → drop +1
    } // 剩 2 個該由 SpscRing::drop 回收
    assert_eq!(
        n.load(Ordering::Relaxed),
        3,
        "應剛好 3 次;<3 = Drop 漏收,崩/亂 = Drop 走錯範圍碰到未初始化槽"
    );
}

/// 🔴 抓 #2 另一角:完全不 pop 就 drop,3 個都該回收。
#[test]
fn drop_reclaims_when_none_popped() {
    let n = Arc::new(AtomicUsize::new(0));
    {
        let (mut tx, _rx) = channel(4);
        for _ in 0..3 {
            tx.push(DropSpy(n.clone())).unwrap();
        }
    }
    assert_eq!(
        n.load(Ordering::Relaxed),
        3,
        "未消費的 3 個都該被 Drop 回收"
    );
}

/// 🔴 抓 #1:cap 非 2 的冪時,mask 沒處理好會讓多個邏輯位置撞同一實體槽
/// → 值被覆蓋、FIFO 崩。目前的 `mask = cap - 1` 會讓這條紅。
/// 註:本測試假設你選「向上取 2 的冪」契約;若你選 assert!(is_power_of_two),
/// channel(5) 會 panic,那就把它改成 #[should_panic]。
#[test]
fn odd_capacity_preserves_fifo() {
    let (mut tx, mut rx) = channel(5);
    let mut pushed = 0u64;
    while tx.push(pushed).is_ok() {
        pushed += 1;
        if pushed > 1024 {
            break; // 保險:契約沒實作時別無限塞
        }
    }
    for i in 0..pushed {
        assert_eq!(rx.pop(), Some(i), "非 2 的冪 cap:槽位撞車,FIFO 崩");
    }
    assert_eq!(rx.pop(), None);
}
