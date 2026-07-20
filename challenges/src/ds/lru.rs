//! ★ challenge:LRU cache
//!
//! 【題目】固定容量的 key-value cache,滿了淘汰「最久未使用」的條目。
//! `get` 與 `put` 都必須是 O(1)(期望時間)。
//!
//! 【constraints】
//! - std-only、單執行緒;容量 ≥ 1
//! - get 命中要把條目變成「最近使用」;put 已存在的 key 是更新(也算使用)
//! - put 造成淘汰時,回傳被淘汰的 (K, V)(呼叫端可能要寫回)
//! - **禁止 O(n) 掃描**——資料結構組合自己想(這就是本題)
//! - 不准用 Rc/RefCell/unsafe(Rust 面試的鏈式結構有更好的路)
//!
//! 【clarify points——動手前先自答】
//! - O(1) 定位 + O(1) 調整使用順序,各需要什麼結構?怎麼黏起來?
//! - 「淘汰 tail 時要從 node 反查刪 map 條目」——key 需要存幾份?
//! - 你的節點放哪裡?刪除/覆寫時索引還有效嗎?
//!
//! 【要實作】下方簽名。【驗收】tests/lru.rs 轉綠。

use std::hash::Hash;
use std::marker::PhantomData;

pub struct LruCache<K, V> {
    // ↓ 佔位:動手時整個換成你的設計。
    _todo: PhantomData<(K, V)>,
}

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    pub fn new(cap: usize) -> Self {
        todo!("challenge: 從空白開始")
    }

    /// 命中即「使用」(影響淘汰順序)。O(1)。
    pub fn get(&mut self, key: &K) -> Option<&V> {
        todo!("challenge")
    }

    /// 插入/更新;容量滿而需要淘汰時回傳被淘汰的 (K, V)。O(1)。
    pub fn put(&mut self, key: K, value: V) -> Option<(K, V)> {
        todo!("challenge")
    }

    /// 只讀不影響淘汰順序。O(1)。
    pub fn peek(&self, key: &K) -> Option<&V> {
        todo!("challenge")
    }

    pub fn len(&self) -> usize {
        todo!("challenge")
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
