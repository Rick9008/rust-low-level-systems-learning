//! # lru_locked —— sharded LRU(N 把 Mutex,每 shard 一個獨立 LRU)
//!
//! ## [Clarify]
//! 解決:多執行緒共享的 LRU cache。關鍵事實:**精確 LRU 的 get 是寫操作**
//! (promote-to-MRU 要動鏈表)⇒ RwLock 無效、每次讀都要寫鎖。
//! 本模組給 production 的實際落點:sharding。
//! Constraints:容量 = shards × cap_per_shard **靜態切分**;
//! 逐出品質是 per-shard 近似(熱點 shard 不能借冷 shard 的額度);
//! `get` 回 `V: Clone` 的複製(借用出不了 MutexGuard)。
//!
//! ## [Abstract]
//! 單 shard 的 LRU 邏輯完全復用 `crate::ds::lru::LruCache`——本模組只做
//! 「hash 選 shard + 鎖」這一層。全域精確 LRU、TTL、weigher 都不做
//! (要近似 recency 的無鎖讀路徑見 docs 的 CLOCK / W-TinyLFU 討論)。
//!
//! ## [Iterate]
//! 階梯:`Mutex<LruCache>` 全域一把鎖(level 0,正確先行)→ **本模組**
//! (吞吐 ×N,犧牲全域逐出順序)→ 讀路徑去鎖化 = 放棄精確 LRU
//! (CLOCK 的 atomic flag / W-TinyLFU 的 per-thread buffer——那是另一個模組
//! 量級的工程,見 docs/concurrency/ds_sync.md)。
//!
//! ## [Trade-offs]
//! - shard 數 × 單 shard 容量在建構時定死:實作零 rebalance 成本,
//!   代價是 hot-shard 提早逐出(見 `per_shard_eviction_not_global` 的手 trace)。
//! - hasher 用 `BuildHasherDefault<DefaultHasher>`(固定 key):key→shard
//!   可重現,測試/教學友善;產線要抗 HashDoS 就換 `RandomState`,
//!   代價是每個 process 的 shard 分佈不同、不可離線重演。
//! - `get` 收 `V: Clone`:值大就把 V 包 `Arc`(clone = refcount++)。
//!   另一條路是閉包式 `with(key, f)`——不強迫 Clone,但 f 在鎖內跑,
//!   慢 f 會拖住整個 shard。這裡選 Clone:鎖內只做指標搬運。
//! - `len` 是跨 shard 加總的**行進中快照**(非原子),只當觀測值用。
//! - 時間 O(1) + 單 shard 鎖競爭;空間 O(shards × cap_per_shard)。
//!
//! ## [Dry-Run]
//! 見 `per_shard_eviction_not_global`(教學主場:全域還有空位卻逐出)、
//! 單 shard 退化測試(行為 == 裸 LruCache)、8 執行緒不相交 key 煙霧測試。

use crate::ds::lru::LruCache;
use std::collections::hash_map::DefaultHasher;
use std::hash::{BuildHasher, BuildHasherDefault, Hash};
use std::sync::Mutex;

/// N 把 Mutex 的 sharded LRU。所有操作 `&self`。
pub struct ShardedLru<K, V> {
    shards: Box<[Mutex<LruCache<K, V>>]>,
}

impl<K: Hash + Eq + Clone, V> ShardedLru<K, V> {
    /// `n_shards` 個 shard,每個容量 `cap_per_shard`。O(shards)。
    pub fn new(n_shards: usize, cap_per_shard: usize) -> Self {
        assert!(n_shards > 0, "need at least one shard");
        Self {
            shards: (0..n_shards)
                .map(|_| Mutex::new(LruCache::new(cap_per_shard)))
                .collect(),
        }
    }

    /// key → shard 編號(觀測/測試用;固定 hasher ⇒ 跨執行、跨行程可重現)。
    pub fn shard_of(&self, key: &K) -> usize {
        let h = BuildHasherDefault::<DefaultHasher>::default().hash_one(key);
        (h as usize) % self.shards.len()
    }

    /// 讀 + promote(該 shard 內)。O(1) + 鎖競爭。
    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        self.shards[self.shard_of(key)]
            .lock()
            .unwrap()
            .get(key)
            .cloned()
    }

    /// 寫入;回傳**該 shard** 逐出的 (K, V)(注意:不是全域 LRU 的犧牲者)。
    /// O(1) + 鎖競爭。
    pub fn put(&self, key: K, value: V) -> Option<(K, V)> {
        self.shards[self.shard_of(&key)]
            .lock()
            .unwrap()
            .put(key, value)
    }

    /// 各 shard len 加總。O(shards);跨 shard 非原子——行進中的快照,觀測用。
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

    /// shards=1 退化:行為必須與裸 LruCache 完全一致
    /// (cap=2:put a,b → get a(promote)→ put c ⇒ 逐出的是 b)。
    #[test]
    fn single_shard_degenerates_to_plain_lru() {
        let c = ShardedLru::new(1, 2);
        assert_eq!(c.put("a", 1), None);
        assert_eq!(c.put("b", 2), None);
        assert_eq!(c.get(&"a"), Some(1)); // promote a
        assert_eq!(c.put("c", 3), Some(("b", 2))); // LRU = b
        assert_eq!(c.get(&"b"), None);
        assert_eq!(c.get(&"a"), Some(1));
        assert_eq!(c.len(), 2);
    }

    /// key→shard 映射穩定,且 key 真的會散開(固定 hasher,結果可重現)。
    #[test]
    fn shard_routing_stable_and_spread() {
        let c: ShardedLru<u32, ()> = ShardedLru::new(4, 8);
        let mut seen = [false; 4];
        for k in 0..64u32 {
            let s = c.shard_of(&k);
            assert_eq!(s, c.shard_of(&k)); // 穩定
            seen[s] = true;
        }
        assert!(
            seen.iter().filter(|&&b| b).count() >= 2,
            "64 key 至少散進兩個 shard"
        );
    }

    /// [Dry-Run] 教學主場:**全域容量還有空位,卻發生逐出**。
    /// shards=2、每 shard cap=1(全域容量 2)。取兩個同 shard 的 key k1, k2:
    ///   put(k1):該 shard {k1},另一 shard 空          → 全域 1/2
    ///   put(k2):同 shard 滿 ⇒ 逐出 k1 —— 但另一 shard 還是空的!
    /// 這就是 sharding 犧牲的「全域 LRU 順序」:容量是按 shard 靜態切分的。
    #[test]
    fn per_shard_eviction_not_global() {
        let c: ShardedLru<u32, u32> = ShardedLru::new(2, 1);
        // 用固定 hasher 掃出前兩個落在同一 shard 的 key(可重現)
        let k1 = 0u32;
        let k2 = (1..).find(|k| c.shard_of(k) == c.shard_of(&k1)).unwrap();
        assert_eq!(c.put(k1, 10), None);
        let evicted = c.put(k2, 20);
        assert_eq!(evicted, Some((k1, 10)), "同 shard 撞滿:k1 被逐出");
        assert_eq!(c.len(), 1, "全域容量 2 只住了 1 個——另一 shard 空著");
    }

    /// 並發煙霧測試:8 執行緒寫不相交的 key(池夠大不逐出),
    /// join 後全部讀得到、len 精確。
    #[test]
    fn concurrent_disjoint_puts_all_visible() {
        let c = Arc::new(ShardedLru::new(8, 256));
        let handles: Vec<_> = (0..8u32)
            .map(|t| {
                let c = Arc::clone(&c);
                thread::spawn(move || {
                    for i in 0..100 {
                        assert_eq!(c.put(t * 1000 + i, i), None); // 不逐出
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(c.len(), 800);
        for t in 0..8u32 {
            for i in 0..100 {
                assert_eq!(c.get(&(t * 1000 + i)), Some(i));
            }
        }
    }
}
