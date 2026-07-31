//! 驗收:challenges::concurrency::spin_lock。完成後移除 #[ignore]。

use challenges::concurrency::spin_lock::SpinLock;
use std::sync::Arc;
use std::thread;

/// boundary:單執行緒 lock → 改 → 持鎖中 try_lock=None → 放 → try_lock 讀到新值。
#[test]
#[ignore = "challenge:空白題,寫完再拔"]
fn single_thread_roundtrip() {
    let lock = SpinLock::new(0_u64);
    let mut g = lock.lock();
    *g += 41;
    assert!(lock.try_lock().is_none(), "持鎖中必須拿不到");
    drop(g);
    let g2 = lock.try_lock().expect("放鎖後必須拿得到");
    assert_eq!(*g2, 41);
}

/// 互斥本體:2 執行緒 × 100k 遞增,一次不少。
/// (`+= 1` 是 load-modify-store 三步——沒有互斥時必然丟更新。)
#[test]
#[ignore = "challenge:空白題,寫完再拔"]
fn mutual_exclusion_two_threads() {
    const N: u64 = 100_000;
    let lock = Arc::new(SpinLock::new(0_u64));
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let l = Arc::clone(&lock);
            thread::spawn(move || {
                for _ in 0..N {
                    *l.lock() += 1;
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(*lock.lock(), 2 * N);
}

/// 臨界區 panic:unwind 必須放鎖(鎖不可壞死),資料停在 panic 前的狀態。
#[test]
#[ignore = "challenge:空白題,寫完再拔"]
fn panic_does_not_wedge_the_lock() {
    let lock = Arc::new(SpinLock::new(0_u64));
    let l = Arc::clone(&lock);
    let h = thread::spawn(move || {
        let mut g = l.lock();
        *g += 1;
        panic!("boom in critical section");
    });
    assert!(h.join().is_err());
    let g = lock.try_lock().expect("unwind 必須已放鎖");
    assert_eq!(*g, 1);
}
