//! 驗收:challenges::lru。完成後移除 #[ignore]。

use challenges::lru::LruCache;

/// 核心驗收:promotion 改變淘汰受害者。
#[test]
#[ignore = "完成 challenge 後移除"]
fn promotion_changes_victim() {
    let mut c = LruCache::new(2);
    assert_eq!(c.put("a", 1), None);
    assert_eq!(c.put("b", 2), None);
    assert_eq!(c.get(&"a"), Some(&1)); // a 變最新
    assert_eq!(c.put("c", 3), Some(("b", 2))); // 淘汰 b
    assert_eq!(c.get(&"b"), None);
    assert_eq!(c.get(&"a"), Some(&1));
    assert_eq!(c.get(&"c"), Some(&3));
}

/// boundary:cap=1——每個新 key 都淘汰唯一住戶。
#[test]
#[ignore = "完成 challenge 後移除"]
fn cap_one() {
    let mut c = LruCache::new(1);
    c.put(1, "one");
    assert_eq!(c.put(2, "two"), Some((1, "one")));
    assert_eq!(c.get(&1), None);
    assert_eq!(c.get(&2), Some(&"two"));
}

/// boundary:同 key put 是更新不是插入(len 不變、無淘汰)。
#[test]
#[ignore = "完成 challenge 後移除"]
fn same_key_update() {
    let mut c = LruCache::new(2);
    c.put("k", 1);
    assert_eq!(c.put("k", 2), None);
    assert_eq!(c.len(), 1);
    assert_eq!(c.get(&"k"), Some(&2));
}

/// boundary:peek 不 promote。
#[test]
#[ignore = "完成 challenge 後移除"]
fn peek_does_not_promote() {
    let mut c = LruCache::new(2);
    c.put("a", 1);
    c.put("b", 2);
    assert_eq!(c.peek(&"a"), Some(&1));
    assert_eq!(c.put("c", 3), Some(("a", 1))); // a 仍是 LRU
}

/// 壓力:大量交錯操作後行為與 O(n) 天真模型一致。
#[test]
#[ignore = "完成 challenge 後移除"]
fn matches_naive_model() {
    struct Naive {
        entries: Vec<(u32, u32)>,
        cap: usize,
    }
    impl Naive {
        fn get(&mut self, k: u32) -> Option<u32> {
            let pos = self.entries.iter().position(|e| e.0 == k)?;
            let e = self.entries.remove(pos);
            self.entries.insert(0, e);
            Some(self.entries[0].1)
        }
        fn put(&mut self, k: u32, v: u32) -> Option<(u32, u32)> {
            if let Some(pos) = self.entries.iter().position(|e| e.0 == k) {
                self.entries.remove(pos);
                self.entries.insert(0, (k, v));
                return None;
            }
            let evicted = if self.entries.len() == self.cap {
                self.entries.pop()
            } else {
                None
            };
            self.entries.insert(0, (k, v));
            evicted
        }
    }
    let mut lru = LruCache::new(3);
    let mut model = Naive {
        entries: Vec::new(),
        cap: 3,
    };
    // 決定性偽隨機序列(LCG),key 空間小逼出高碰撞
    let mut x: u64 = 42;
    for _ in 0..5000 {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let k = (x >> 33) as u32 % 6;
        let v = (x >> 40) as u32;
        if x.is_multiple_of(2) {
            assert_eq!(lru.get(&k).copied(), model.get(k));
        } else {
            assert_eq!(lru.put(k, v), model.put(k, v));
        }
    }
}
