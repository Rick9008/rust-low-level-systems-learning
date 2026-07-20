//! drill:lru —— 填 index-based 雙向鏈表的手術刀。
//!
//! 已給:結構、new、peek、len。
//! 要填:`unlink` / `push_front`(鏈表手術)、`get` / `put`(組合邏輯)。
//! 紙上先畫 cap=2 的鏈:put a、put b、get a、put c——誰被淘汰?
//! 哨兵:NIL = usize::MAX 表示「無」。

use std::collections::HashMap;
use std::hash::Hash;

const NIL: usize = usize::MAX;

struct Node<K, V> {
    key: K,
    value: V,
    prev: usize,
    next: usize,
}

pub struct LruCache<K, V> {
    map: HashMap<K, usize>,
    nodes: Vec<Node<K, V>>, // 只覆寫不 remove ⇒ 索引永遠有效
    head: usize,            // MRU
    tail: usize,            // LRU(淘汰端)
    cap: usize,
}

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0);
        Self {
            map: HashMap::with_capacity(cap),
            nodes: Vec::with_capacity(cap),
            head: NIL,
            tail: NIL,
            cap,
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// spec:把節點 i 從鏈表摘下(不動 map)。四種情況:
    /// i 是頭(head 改指 next)、i 是尾(tail 改指 prev)、兩者皆是、中間。
    /// 提示:先取出 (prev, next),再分別修 prev 端與 next 端。
    fn unlink(&mut self, i: usize) {
        todo!("spec: 修 nodes[prev].next / nodes[next].prev,邊界時改 head/tail")
    }

    /// spec:把節點 i 掛到頭(MRU 端)。
    /// i 的 prev=NIL、next=舊 head;舊 head 的 prev 指回 i;head=i;
    /// 空表時(tail==NIL)tail 也要指 i。
    fn push_front(&mut self, i: usize) {
        todo!("spec: 接四條線:i.prev, i.next, 舊head.prev, head(+空表時 tail)")
    }

    /// spec:命中 → promote(unlink + push_front,已在頭可短路)→ Some(&value);
    /// 未命中 → None。
    pub fn get(&mut self, key: &K) -> Option<&V> {
        todo!("spec: map 查索引; promote; 回 &self.nodes[i].value")
    }

    /// spec:三條路——
    /// 1. key 已存在:更新 value + promote,回 None
    /// 2. 未滿:新節點 push 進 nodes(索引 = nodes.len()),map 記錄,push_front,回 None
    /// 3. 滿:淘汰 tail——unlink、map.remove(舊 key)、
    ///    用 std::mem::replace 原地換入新節點、map.insert、push_front,
    ///    回 Some((舊 key, 舊 value))
    pub fn put(&mut self, key: K, value: V) -> Option<(K, V)> {
        todo!("spec: 更新 / 新增 / 淘汰重用槽位 三條路")
    }

    /// 只讀不 promote。
    pub fn peek(&self, key: &K) -> Option<&V> {
        self.map.get(key).map(|&i| &self.nodes[i].value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// boundary:經典淘汰序——promotion 改變受害者。
    #[test]
    #[ignore = "填完四個函式後移除"]
    fn promotion_changes_eviction_victim() {
        let mut c = LruCache::new(2);
        assert_eq!(c.put("a", 1), None);
        assert_eq!(c.put("b", 2), None);
        assert_eq!(c.get(&"a"), Some(&1)); // promote a
        assert_eq!(c.put("c", 3), Some(("b", 2))); // 淘汰 b 不是 a
        assert_eq!(c.get(&"b"), None);
        assert_eq!(c.get(&"a"), Some(&1));
    }

    /// boundary:cap=1、同 key 更新、空 get。
    #[test]
    #[ignore = "填完四個函式後移除"]
    fn cap_one_and_update() {
        let mut c = LruCache::new(1);
        assert_eq!(c.get(&1), None);
        c.put(1, "one");
        assert_eq!(c.put(1, "uno"), None); // 更新非淘汰
        assert_eq!(c.len(), 1);
        assert_eq!(c.put(2, "two"), Some((1, "uno")));
    }

    /// boundary:get 頭部元素(promote 短路)不得破壞鏈表。
    #[test]
    #[ignore = "填完四個函式後移除"]
    fn get_head_then_evict_correct_victim() {
        let mut c = LruCache::new(2);
        c.put("a", 1);
        c.put("b", 2); // list=[b,a]
        assert_eq!(c.get(&"b"), Some(&2)); // b 已在頭
        assert_eq!(c.put("c", 3), Some(("a", 1)));
    }
}
