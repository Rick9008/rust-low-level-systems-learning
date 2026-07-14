//! 驗收:challenges::spsc_ring。完成後移除 #[ignore]。

use challenges::spsc_ring::channel;
use std::thread;

/// boundary:滿/空/歸還。
#[test]
#[ignore = "完成 challenge 後移除"]
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
#[ignore = "完成 challenge 後移除"]
fn empty_pop_is_none() {
    let (_tx, mut rx) = channel::<u64>(4);
    assert_eq!(rx.pop(), None);
}

/// boundary:多輪 wrap——push/pop 交替次數遠超容量。
#[test]
#[ignore = "完成 challenge 後移除"]
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
#[ignore = "完成 challenge 後移除"]
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
