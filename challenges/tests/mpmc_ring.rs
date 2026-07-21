//! 驗收:challenges::concurrency::mpmc_ring。完成後移除 #[ignore]。

use challenges::concurrency::mpmc_ring::MpmcRing;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

/// boundary:滿/空/歸還(容量可能被上調,塞到 Err 為止)。
#[test]
#[ignore = "完成 challenge 後移除"]
fn full_empty_and_return() {
    let q = MpmcRing::new(2);
    let mut pushed = 0u64;
    let overflow = loop {
        match q.try_push(pushed) {
            Ok(()) => pushed += 1,
            Err(back) => break back,
        }
    };
    assert_eq!(overflow, pushed, "滿時應歸還同一個元素");
    assert!(pushed >= 2, "容量至少 2:實際 {pushed}");
    for i in 0..pushed {
        assert_eq!(q.try_pop(), Some(i), "FIFO(單執行緒下取號順序=呼叫順序)");
    }
    assert_eq!(q.try_pop(), None);
}

/// boundary:排空後環狀重用(第二圈)。
#[test]
#[ignore = "完成 challenge 後移除"]
fn reuse_after_drain() {
    let q = MpmcRing::new(2);
    let cap = q.capacity() as u64;
    for round in 0..3 {
        for i in 0..cap {
            q.try_push(round * 100 + i).unwrap();
        }
        for i in 0..cap {
            assert_eq!(q.try_pop(), Some(round * 100 + i));
        }
        assert_eq!(q.try_pop(), None);
    }
}

/// boundary:帶著未消費元素 drop——全數回收、剛好一次。
#[test]
#[ignore = "完成 challenge 後移除"]
fn drop_reclaims_unconsumed() {
    #[derive(Debug)]
    struct DropSpy(Arc<AtomicUsize>);
    impl Drop for DropSpy {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
    let n = Arc::new(AtomicUsize::new(0));
    {
        let q = MpmcRing::new(4);
        for _ in 0..3 {
            q.try_push(DropSpy(n.clone())).unwrap();
        }
        drop(q.try_pop()); // 消費 1
    }
    assert_eq!(n.load(Ordering::Relaxed), 3, "1 消費 + 2 由 Drop 回收");
}

/// 2P2C 壓力測試:不丟不重(multiset 相等)+ 每個 producer 各自保序。
#[test]
#[ignore = "完成 challenge 後移除"]
fn two_producers_two_consumers_stress() {
    const PER: u64 = 30_000;
    let q = Arc::new(MpmcRing::new(8));
    let done = Arc::new(AtomicUsize::new(0));
    let mut producers = Vec::new();
    for pid in 0..2u64 {
        let q = Arc::clone(&q);
        producers.push(thread::spawn(move || {
            for i in 0..PER {
                let mut item = (pid << 32) | i;
                while let Err(back) = q.try_push(item) {
                    item = back;
                    thread::yield_now();
                }
            }
        }));
    }
    let mut consumers = Vec::new();
    for _ in 0..2 {
        let q = Arc::clone(&q);
        let done = Arc::clone(&done);
        consumers.push(thread::spawn(move || {
            let mut got = Vec::new();
            loop {
                match q.try_pop() {
                    Some(v) => {
                        got.push(v);
                        done.fetch_add(1, Ordering::Relaxed);
                    }
                    None => {
                        if done.load(Ordering::Relaxed) as u64 == 2 * PER {
                            break;
                        }
                        thread::yield_now();
                    }
                }
            }
            got
        }));
    }
    for p in producers {
        p.join().unwrap();
    }
    let mut all: Vec<u64> = Vec::new();
    for c in consumers {
        let got = c.join().unwrap();
        let mut last = [None::<u64>; 2];
        for &v in &got {
            let (pid, i) = ((v >> 32) as usize, v & 0xffff_ffff);
            if let Some(prev) = last[pid] {
                assert!(i > prev, "producer {pid} 的元素在單一 consumer 內亂序");
            }
            last[pid] = Some(i);
        }
        all.extend(got);
    }
    all.sort_unstable();
    let expect: Vec<u64> = (0..2u64)
        .flat_map(|pid| (0..PER).map(move |i| (pid << 32) | i))
        .collect();
    assert_eq!(all, expect);
}
