//! 驗收:challenges::sharded_map。完成後移除 #[ignore]。

use challenges::sharded_map::ShardedMap;
use std::sync::Arc;
use std::thread;

/// 核心驗收:同 key 永遠找得回來(hasher/分片一致性)。
#[test]
#[ignore = "完成 challenge 後移除"]
fn keys_never_vanish() {
    let m = ShardedMap::new(16);
    for i in 0..1000u32 {
        m.insert(i, i * 2);
    }
    for i in 0..1000u32 {
        assert_eq!(m.get_cloned(&i), Some(i * 2), "key {i} 消失");
    }
    assert_eq!(m.len(), 1000);
}

/// insert 回舊值;remove 回被移除的值。
#[test]
#[ignore = "完成 challenge 後移除"]
fn insert_and_remove_semantics() {
    let m = ShardedMap::new(4);
    assert_eq!(m.insert("k", 1), None);
    assert_eq!(m.insert("k", 2), Some(1));
    assert_eq!(m.remove(&"k"), Some(2));
    assert_eq!(m.remove(&"k"), None);
}

/// 並發不相交插入:全部可見。
#[test]
#[ignore = "完成 challenge 後移除"]
fn concurrent_disjoint_inserts() {
    let m = Arc::new(ShardedMap::new(16));
    let handles: Vec<_> = (0..8u32)
        .map(|t| {
            let m = Arc::clone(&m);
            thread::spawn(move || {
                for i in 0..1000 {
                    m.insert(t * 1000 + i, t);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(m.len(), 8000);
}

/// 核心驗收:同 key 高競爭 upsert,8×1000 次遞增一次不丟。
#[test]
#[ignore = "完成 challenge 後移除"]
fn contended_upsert_atomic() {
    let m = Arc::new(ShardedMap::new(16));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let m = Arc::clone(&m);
            thread::spawn(move || {
                for _ in 0..1000 {
                    m.upsert("counter", 0u64, |v| *v += 1);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(m.get_cloned(&"counter"), Some(8000));
}
