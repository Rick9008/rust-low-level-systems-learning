//! drill:sharded_map —— 填 shard 選擇與原子 upsert。
//!
//! 已給:結構、insert/remove/get_cloned/with/len。
//! 要填:`shard_for`(hasher 一致性 + power-of-2 mask)與 `upsert`
//! (一段式 read-modify-write——兩段式會丟更新)。

use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, RandomState};
use std::sync::Mutex;

pub struct ShardedMap<K, V> {
    build_hasher: RandomState, // 必須共用:每次 new 一個 seed 不同,key 會「消失」
    shards: Box<[Mutex<HashMap<K, V>>]>,
    mask: usize, // shards.len() - 1(len 為 2 的冪)
}

impl<K: Hash + Eq, V> ShardedMap<K, V> {
    pub fn new(num_shards: usize) -> Self {
        let n = num_shards.max(1).next_power_of_two();
        Self {
            build_hasher: RandomState::new(),
            shards: (0..n).map(|_| Mutex::new(HashMap::new())).collect(),
            mask: n - 1,
        }
    }

    pub fn num_shards(&self) -> usize {
        self.shards.len()
    }

    /// spec:用 `self.build_hasher.hash_one(key)` 取 hash,
    /// 以 mask 取低位選 shard,回傳該 shard 的參照。
    /// 為什麼要存 build_hasher 而不是每次 RandomState::new()?想清楚再填。
    fn shard_for(&self, key: &K) -> &Mutex<HashMap<K, V>> {
        todo!("spec: hash_one + (h as usize) & self.mask")
    }

    /// spec:原子的 read-modify-write。key 不存在先插入 `init`,
    /// 然後對值執行 `f`。**整段持同一把 shard 鎖**。
    /// 提示:`shard.entry(key).or_insert(init)`。
    pub fn upsert(&self, key: K, init: V, f: impl FnOnce(&mut V)) {
        todo!("spec: lock 一次,entry().or_insert(init),再 f(值)——不可 get 完放鎖再 insert")
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.shard_for(&key).lock().unwrap().insert(key, value)
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.shard_for(key).lock().unwrap().remove(key)
    }

    pub fn get_cloned(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        self.shard_for(key).lock().unwrap().get(key).cloned()
    }

    pub fn with<R>(&self, key: &K, f: impl FnOnce(Option<&V>) -> R) -> R {
        f(self.shard_for(key).lock().unwrap().get(key))
    }

    pub fn len(&self) -> usize {
        self.shards.iter().map(|s| s.lock().unwrap().len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// boundary:同一 key 每次都要落同一 shard(hasher 一致性)。
    #[test]
    #[ignore = "填完 shard_for 後移除"]
    fn insert_then_get_same_key() {
        let m = ShardedMap::new(16);
        for i in 0..100u32 {
            m.insert(i, i * 2);
        }
        for i in 0..100u32 {
            assert_eq!(
                m.get_cloned(&i),
                Some(i * 2),
                "key {i} 消失:shard 選擇不一致?"
            );
        }
    }

    /// boundary:同 key 高競爭 upsert,8×1000 次遞增一次不丟。
    #[test]
    #[ignore = "填完 shard_for/upsert 後移除"]
    fn contended_upsert_loses_nothing() {
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
}
