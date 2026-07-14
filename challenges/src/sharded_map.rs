//! challenge:sharded concurrent map
//!
//! 【題目】多執行緒共享的 key-value map。單一 `Mutex<HashMap>` 在高並發下
//! 鎖競爭嚴重——設計一個分片(sharded)版本降低競爭。
//!
//! 【constraints】
//! - std-only;不准用 dashmap 等 crate
//! - 單 key 操作(insert/get/remove/upsert)在均勻負載下的鎖競爭
//!   應隨 shard 數下降
//! - `upsert` 必須是原子的 read-modify-write(並發計數不丟更新)
//! - 不可能有死鎖(想想你的鎖持有紀律)
//!
//! 【clarify points——動手前先自答】
//! - 同一個 key 每次怎麼保證落在同一個 shard?hasher 存在哪?
//! - `get(&K) -> Option<&V>` 這個簽名為什麼做不到?你打算給什麼替代?
//! - `len()` 跨所有 shard——它的結果在並發下是什麼語意?
//!
//! 【要實作】下方簽名。【驗收】tests/sharded_map.rs 轉綠。

use std::hash::Hash;
use std::marker::PhantomData;

pub struct ShardedMap<K, V> {
    // ↓ 佔位:動手時整個換成你的設計。
    _todo: PhantomData<(K, V)>,
}

impl<K: Hash + Eq, V> ShardedMap<K, V> {
    /// shard 數可向上調整到方便的數字。
    pub fn new(num_shards: usize) -> Self {
        todo!("challenge: 從空白開始")
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        todo!("challenge")
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        todo!("challenge")
    }

    pub fn get_cloned(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        todo!("challenge")
    }

    /// 原子 read-modify-write:不存在先放 init,然後 f(&mut 值)。
    pub fn upsert(&self, key: K, init: V, f: impl FnOnce(&mut V)) {
        todo!("challenge")
    }

    /// 跨 shard 總數(快照語意)。
    pub fn len(&self) -> usize {
        todo!("challenge")
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// 提示(不是 helper):你的實作需要讓 &ShardedMap 能跨執行緒共享。
// 如果欄位選型正確(鎖包資料),Send/Sync 會自動成立,不需要 unsafe impl。
