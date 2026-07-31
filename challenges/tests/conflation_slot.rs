//! 驗收:challenges::concurrency::conflation_slot。完成後移除 #[ignore]。

use challenges::concurrency::conflation_slot::Conflator;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

/// boundary:三筆同 key publish 摺成一筆(最新值 + 摺疊數);取走後可再排隊。
#[test]
#[ignore = "challenge:空白題,寫完再拔"]
fn fold_semantics() {
    let c = Conflator::new();
    c.publish(7_u32, 1, 100_u64);
    c.publish(7, 2, 200);
    c.publish(7, 3, 300);
    assert_eq!(c.recv(), Some((7, 300, 3)));
    c.publish(7, 4, 400);
    assert_eq!(c.recv(), Some((7, 400, 1)));
}

/// 公平性:吵鬧 key(1000 筆)摺成一格,安靜 key 的通知不被擠掉。
#[test]
#[ignore = "challenge:空白題,寫完再拔"]
fn noisy_key_cannot_evict_quiet_key() {
    let c = Conflator::new();
    c.publish(1_u32, 1, 11_u64);
    for s in 1..=1000 {
        c.publish(2, s, s);
    }
    assert_eq!(c.recv(), Some((1, 11, 1)), "安靜 key 必須先到且存活");
    assert_eq!(c.recv(), Some((2, 1000, 1000)), "吵鬧 key 摺成 1 筆");
}

/// 亂序保護:遲到的舊 seq 不許覆蓋,且可觀測。
#[test]
#[ignore = "challenge:空白題,寫完再拔"]
fn stale_seq_rejected() {
    let c = Conflator::new();
    c.publish(5_u32, 10, 999_u64);
    c.publish(5, 3, 111);
    assert_eq!(c.recv(), Some((5, 999, 1)));
    assert_eq!(c.stale_count(), 1);
}

/// close 語意:drain 完剩貨 → None 不掛死;關店後 publish 忽略。
#[test]
#[ignore = "challenge:空白題,寫完再拔"]
fn close_drains_then_none() {
    let c = Conflator::new();
    c.publish(1_u32, 1, 10_u64);
    c.publish(2, 1, 20);
    c.close();
    c.publish(3, 1, 30);
    let mut got = vec![c.recv(), c.recv()];
    got.sort();
    assert_eq!(got, vec![Some((1, 10, 1)), Some((2, 20, 1))]);
    assert_eq!(c.recv(), None);
}

/// 最終狀態保證:亂流之後每個 key 最後收到的值 = 它最後一次 publish;不回退。
#[test]
#[ignore = "challenge:空白題,寫完再拔"]
fn final_state_delivered_under_contention() {
    let c = Arc::new(Conflator::new());
    let p = {
        let c = Arc::clone(&c);
        thread::spawn(move || {
            for s in 1..=5000_u64 {
                c.publish((s % 4) as u32, s, s);
            }
            c.close();
        })
    };
    let cc = Arc::clone(&c);
    let consumer = thread::spawn(move || {
        let mut last: HashMap<u32, u64> = HashMap::new();
        while let Some((k, v, _n)) = cc.recv() {
            if let Some(&prev) = last.get(&k) {
                assert!(v > prev, "key {k} 回退 {prev} -> {v}");
            }
            last.insert(k, v);
        }
        last
    });
    p.join().unwrap();
    let last = consumer.join().unwrap();
    for k in 0..4_u32 {
        let expect = (1..=5000_u64)
            .filter(|s| (s % 4) as u32 == k)
            .last()
            .unwrap();
        assert_eq!(last[&k], expect, "key {k} 最終狀態未送達");
    }
}
