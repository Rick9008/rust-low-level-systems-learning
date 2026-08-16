//! 第 3 關：HashMap + index-based 雙向鏈表實作 LRU cache。
//!
//! 先畫出 `head -> ... -> tail`，再實作鏈表操作。不要同時修改多個函式。

use std::collections::HashMap;
use std::hash::Hash;

struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

pub struct LruCache<K, V> {
    positions: HashMap<K, usize>,
    nodes: Vec<Node<K, V>>,
    head: Option<usize>,
    tail: Option<usize>,
    capacity: usize,
}

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be greater than zero");
        Self {
            positions: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            head: None,
            tail: None,
            capacity,
        }
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// 將 index 指向的節點從鏈表中拆下，但不刪除 Vec 或 HashMap 的資料。
    fn detach(&mut self, index: usize) {
        todo!("先複製 prev/next，再分別修正鄰居與 head/tail")
    }

    /// 將已存在的節點放到 head（most recently used）。
    fn push_front(&mut self, index: usize) {
        todo!("設定節點的 prev/next，接回舊 head，並處理空鏈表")
    }

    /// 命中時把節點移到 head，再回傳 value 的引用。
    pub fn get(&mut self, key: &K) -> Option<&V> {
        todo!("先從 positions 複製 index，完成鏈表修改後再建立回傳引用")
    }

    /// 插入或更新資料；滿載時淘汰 tail，並回傳被淘汰的 key/value。
    ///
    /// 建議分成三條路：更新既有 key、未滿時新增、滿載時重用 tail 的 slot。
    pub fn put(&mut self, key: K, value: V) -> Option<(K, V)> {
        todo!("分別完成更新、新增、淘汰三種情況")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lru_promotes_recently_used_item() {
        let mut cache = LruCache::new(2);
        assert_eq!(cache.put("a", 1), None);
        assert_eq!(cache.put("b", 2), None);
        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.put("c", 3), Some(("b", 2)));
        assert_eq!(cache.get(&"b"), None);
        assert_eq!(cache.get(&"a"), Some(&1));
    }

    #[test]
    fn lru_updates_without_growing() {
        let mut cache = LruCache::new(2);
        cache.put("a", 1);
        cache.put("a", 10);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(&"a"), Some(&10));
    }

    #[test]
    fn lru_capacity_one_reuses_the_slot() {
        let mut cache = LruCache::new(1);
        cache.put(1, "one");
        assert_eq!(cache.put(2, "two"), Some((1, "one")));
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"two"));
    }
}
