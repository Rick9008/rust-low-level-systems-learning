//! # sharded_map —— per-shard Mutex 的並發 HashMap
//!
//! ## [Clarify]
//! 解決:多執行緒共享的 key-value map,單一 `Mutex<HashMap>` 在高並發下
//! 所有操作序列化,鎖競爭成為瓶頸。分片(shard)讓不同 key 大概率落在
//! 不同鎖上,均勻負載下競爭降 ~N 倍(N = shard 數)。
//! Constraints:std-only、K: Hash + Eq、operations 只碰單一 key(無跨 key 交易)。
//! 預期規模:10⁵–10⁷ entries、10¹–10² 執行緒;shard 數取 2 的冪(典型 16–64)。
//!
//! ## [Abstract]
//! hash 函數用 std 的 RandomState stub 掉(面試時聲明「先用預設 hasher 往前走」);
//! 不做 resize 跨 shard rebalance——每個 shard 是獨立 HashMap,自己 resize。
//!
//! ## [Trade-offs]
//! - **鎖不變量:任何操作最多持有一把 shard 鎖**。跨 shard 的操作(len、iter)
//!   只能是非線性化的快照,或要按固定順序鎖全部 shard(本實作選前者)。
//!   永不同時持兩把鎖 ⇒ 無 lock-ordering 死鎖問題。
//! - `get(&K) -> Option<&V>` 做不到:`&V` 指向 shard 內部,鎖 guard 一離開
//!   函式就釋放,`&V` 會懸空——Rust 直接不讓編譯。三條出路:
//!   (1) `V: Clone` 複製出來(本實作 `get_cloned`),
//!   (2) closure API 在持鎖區間內借用(本實作 `with`),
//!   (3) 值存 `Arc<V>`(caller 自己包)。
//! - 對照 production:dashmap(分片 + RwLock)、papaya / evmap(epoch/雙 buffer)。
//!
//! ## [Dry-Run]
//! 見測試:roundtrip、shard 數上取 2 的冪、單 shard 退化、並發插入無遺失、
//! 同 key 高競爭 upsert 計數正確。

use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, RandomState};
use std::sync::Mutex;

pub struct ShardedMap<K, V> {
    // 同一個 BuildHasher 供所有操作使用:同一 key 永遠落同一 shard。
    // 每次操作各自 new 一個 RandomState 是經典 bug——seed 不同,key 會「消失」。
    build_hasher: RandomState,
    shards: Box<[Mutex<HashMap<K, V>>]>,
    // shards.len() 為 2 的冪,mask = len-1:
    // 取 shard 用位遮罩 O(1),比 `%`(整數除法,x86 上 ~20-40 cycle)快且無分支。
    mask: usize,
}

impl<K: Hash + Eq, V> ShardedMap<K, V> {
    /// `num_shards` 上取到 2 的冪(至少 1)。
    /// 空間:N 個空 HashMap + N 個 Mutex,O(N);時間換空間買併發度。
    pub fn new(num_shards: usize) -> Self {
        let n = num_shards.max(1).next_power_of_two();
        let shards = (0..n).map(|_| Mutex::new(HashMap::new())).collect();
        Self {
            build_hasher: RandomState::new(),
            shards,
            mask: n - 1,
        }
    }

    pub fn num_shards(&self) -> usize {
        self.shards.len()
    }

    fn shard_for(&self, key: &K) -> &Mutex<HashMap<K, V>> {
        let h = self.build_hasher.hash_one(key);
        // 取 hash 高低位皆可;mask 取低位。u64 → usize 截斷在 64-bit 平台無損。
        &self.shards[(h as usize) & self.mask]
    }

    /// O(1) 期望時間(hash + 單 shard HashMap insert)。回傳被覆蓋的舊值。
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.shard_for(&key).lock().unwrap().insert(key, value)
    }

    /// O(1) 期望時間。
    pub fn remove(&self, key: &K) -> Option<V> {
        self.shard_for(key).lock().unwrap().remove(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.shard_for(key).lock().unwrap().contains_key(key)
    }

    /// 出路 (1):複製值。適合 V 小(數十 byte);V 大時改用 `with` 或存 Arc<V>。
    pub fn get_cloned(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        self.shard_for(key).lock().unwrap().get(key).cloned()
    }

    /// 出路 (2):closure 在持鎖區間內借用 `&V`,零複製。
    /// 代價:f 執行期間佔著 shard 鎖——f 必須短,禁止在 f 裡再碰同一個 map(死鎖)。
    pub fn with<R>(&self, key: &K, f: impl FnOnce(Option<&V>) -> R) -> R {
        f(self.shard_for(key).lock().unwrap().get(key))
    }

    /// 原子的 read-modify-write:key 不存在先放 `init`,再對值執行 `f`。
    /// 整段持同一把 shard 鎖 ⇒ 並發 upsert 不會遺失更新。
    pub fn upsert(&self, key: K, init: V, f: impl FnOnce(&mut V)) {
        let mut shard = self.shard_for(&key).lock().unwrap();
        f(shard.entry(key).or_insert(init));
    }

    /// 跨 shard 快照總數:O(N) 把鎖依序取放。
    /// **非線性化**:讀 shard 3 時 shard 0 可能已變;只當監控值用。
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

    /// [Dry-Run] roundtrip 手 trace:
    ///   insert("a",1) → None(新 key)   insert("a",2) → Some(1)(覆蓋回舊值)
    ///   get_cloned("a") → Some(2)        remove("a") → Some(2)
    ///   get_cloned("a") → None(已刪)
    /// boundary:同 key 覆蓋、刪除後查詢。
    #[test]
    fn insert_get_remove_roundtrip() {
        let m: ShardedMap<&str, i32> = ShardedMap::new(4);
        assert_eq!(m.insert("a", 1), None);
        assert_eq!(m.insert("a", 2), Some(1));
        assert_eq!(m.get_cloned(&"a"), Some(2));
        assert_eq!(m.remove(&"a"), Some(2));
        assert_eq!(m.get_cloned(&"a"), None);
    }

    /// boundary:shard 數上取 2 的冪;0 與 1 都退化為 1(仍須正確)。
    #[test]
    fn boundary_shard_count_rounds_up_to_power_of_two() {
        assert_eq!(ShardedMap::<u32, u32>::new(0).num_shards(), 1);
        assert_eq!(ShardedMap::<u32, u32>::new(1).num_shards(), 1);
        assert_eq!(ShardedMap::<u32, u32>::new(3).num_shards(), 4);
        assert_eq!(ShardedMap::<u32, u32>::new(16).num_shards(), 16);
        assert_eq!(ShardedMap::<u32, u32>::new(17).num_shards(), 32);
    }

    /// boundary:單 shard 退化 = 全域一把鎖,行為仍正確(只是沒併發度)。
    #[test]
    fn boundary_single_shard_degenerate_still_correct() {
        let m = ShardedMap::new(1);
        for i in 0..100u32 {
            m.insert(i, i * 2);
        }
        assert_eq!(m.len(), 100);
        assert_eq!(m.get_cloned(&7), Some(14));
    }

    /// with:持鎖區間內零複製借用。
    #[test]
    fn with_borrows_without_clone() {
        let m = ShardedMap::new(4);
        m.insert("k".to_string(), vec![1, 2, 3]);
        let sum: i32 = m.with(&"k".to_string(), |v| v.map_or(0, |v| v.iter().sum()));
        assert_eq!(sum, 6);
    }

    /// 並發插入無遺失:8 執行緒 × 1000 個不相交 key。
    /// 驗證 len 總和與抽查值——任何 shard 選擇不一致都會讓 key「消失」。
    #[test]
    fn concurrent_disjoint_inserts_all_visible() {
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
        assert_eq!(m.get_cloned(&(3 * 1000 + 999)), Some(3));
    }

    /// boundary:同一 key 高競爭 read-modify-write。
    /// upsert 整段持鎖 ⇒ 8×1000 次遞增一次不丟。
    /// (若用 get_cloned + insert 兩段式,更新會互相覆蓋,計數 < 8000。)
    #[test]
    fn boundary_same_key_contended_upsert_loses_nothing() {
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
