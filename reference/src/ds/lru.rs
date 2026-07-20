//! # lru —— O(1) get/put 的 LRU cache(index-based 雙向鏈表)
//!
//! ## [Clarify]
//! 解決:固定容量 cache,超出時淘汰「最久未用」。get/put 都必須 O(1)——
//! 所以要 HashMap(O(1) 定位)+ 雙向鏈表(O(1) 調整 recency 順序)的組合;
//! 只用其中一個都會有一邊退化成 O(n)。
//! Constraints:單執行緒;K: Hash + Eq + Clone(key 存兩份:map 一份、node 一份,
//! 見 Trade-offs)。容量 ≥ 1。
//!
//! ## [Abstract]
//! 淘汰時的 callback(寫回 dirty page 之類)不做——面試時聲明 stub 掉,
//! 回傳被淘汰的 (K, V) 讓 caller 自己處理,關注點已隔離。
//!
//! ## [Iterate]
//! 鏈表不用 `Rc<RefCell<Node>>` 也不用裸指標,而是 **node 放 `Vec`、
//! prev/next 存索引**:借用檢查零阻力、cache locality 好、沒有 unsafe。
//! 這是 Rust 寫鏈式結構的第一選擇(對照見 [`crate::ds::tree`] 的兩版並列)。
//!
//! ## [Trade-offs]
//! - key 存兩份(map 的 key + node 裡的 key):淘汰 tail 時要能從 node 反查
//!   map 條目來刪。省掉這份 clone 的方法(map key 用 Rc / raw entry / 索引再反查)
//!   都更複雜——面試先付一次 K: Clone 往前走。
//! - 哨兵:NIL = usize::MAX 代替 `Option<usize>`,prev/next 各省一個 discriminant
//!   位元組並讓 unlink 分支平坦;代價是「usize::MAX 是保留值」這條隱形規則,
//!   用 debug_assert 守住。
//! - 時間:get/put 期望 O(1)(hash)+ 確定 O(1)(鏈表);空間 O(cap)。
//!
//! ## [Dry-Run]
//! 見測試:經典淘汰序(cap=2:put a,b;get a;put c → 淘汰 b)、cap=1、
//! 同 key 重複 put、promotion 逐步 trace、proptest 對照 O(n) 天真模型。

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
    map: HashMap<K, usize>, // key → node 索引
    nodes: Vec<Node<K, V>>, // 索引穩定:只覆寫、不 remove(len ≤ cap ⇒ 不會 realloc 失效索引)
    head: usize,            // 最新(MRU)
    tail: usize,            // 最舊(LRU,淘汰端)
    cap: usize,
}

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0, "zero-capacity cache cannot hold anything");
        Self {
            map: HashMap::with_capacity(cap),
            nodes: Vec::with_capacity(cap), // 預配:活躍期零 realloc
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

    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// 從鏈表摘下節點 i(不動 map)。O(1):最多改 2 個鄰居 + head/tail。
    fn unlink(&mut self, i: usize) {
        let (prev, next) = (self.nodes[i].prev, self.nodes[i].next);
        if prev == NIL {
            self.head = next; // i 是頭
        } else {
            self.nodes[prev].next = next;
        }
        if next == NIL {
            self.tail = prev; // i 是尾
        } else {
            self.nodes[next].prev = prev;
        }
    }

    /// 把節點 i 掛到頭(MRU 端)。O(1)。
    fn push_front(&mut self, i: usize) {
        self.nodes[i].prev = NIL;
        self.nodes[i].next = self.head;
        if self.head != NIL {
            self.nodes[self.head].prev = i;
        }
        self.head = i;
        if self.tail == NIL {
            self.tail = i; // 空表:唯一節點同時是頭尾
        }
    }

    /// O(1)。命中即 promote(移到 MRU 端)。
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let &i = self.map.get(key)?;
        // promote = unlink + push_front;i 已在頭時也正確(兩步都是冪等形狀),
        // 但短路掉可省 5 次寫——常見面試追問點。
        if self.head != i {
            self.unlink(i);
            self.push_front(i);
        }
        Some(&self.nodes[i].value)
    }

    /// 只讀不 promote(觀測用,不干擾 recency)。O(1)。
    pub fn peek(&self, key: &K) -> Option<&V> {
        self.map.get(key).map(|&i| &self.nodes[i].value)
    }

    /// O(1)。回傳因容量而淘汰的 (K, V)(若有)。
    /// 三條路:同 key 更新、有空位新增、滿了淘汰 tail 重用其槽位。
    pub fn put(&mut self, key: K, value: V) -> Option<(K, V)> {
        if let Some(&i) = self.map.get(&key) {
            // 同 key:更新值 + promote,不淘汰任何人
            self.nodes[i].value = value;
            if self.head != i {
                self.unlink(i);
                self.push_front(i);
            }
            return None;
        }
        if self.map.len() < self.cap {
            // 有空位:新節點 append(索引 = 目前長度,穩定不變)
            let i = self.nodes.len();
            self.nodes.push(Node {
                key: key.clone(),
                value,
                prev: NIL,
                next: NIL,
            });
            self.map.insert(key, i);
            self.push_front(i);
            return None;
        }
        // 滿:淘汰 LRU 端,槽位原地重用(不 remove ⇒ 其他索引全部保持有效)
        debug_assert!(self.tail != NIL);
        let i = self.tail;
        self.unlink(i);
        let evicted = std::mem::replace(
            &mut self.nodes[i],
            Node {
                key: key.clone(),
                value,
                prev: NIL,
                next: NIL,
            },
        );
        self.map.remove(&evicted.key);
        self.map.insert(key, i);
        self.push_front(i);
        Some((evicted.key, evicted.value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// [Dry-Run] 經典淘汰序 trace(cap=2):
    ///   put(a,1): list=[a]          put(b,2): list=[b,a]
    ///   get(a)→1: promote,list=[a,b]
    ///   put(c,3): 滿,淘汰 tail=b → Some((b,2)),list=[c,a]
    ///   get(b)→None(已淘汰)  get(a)→Some(1)  get(c)→Some(3)
    /// boundary:promotion 改變淘汰對象——沒 promote 的話被淘汰的會是 a。
    #[test]
    fn boundary_promotion_changes_eviction_victim() {
        let mut c = LruCache::new(2);
        assert_eq!(c.put("a", 1), None);
        assert_eq!(c.put("b", 2), None);
        assert_eq!(c.get(&"a"), Some(&1)); // promote a
        assert_eq!(c.put("c", 3), Some(("b", 2))); // b 是 LRU
        assert_eq!(c.get(&"b"), None);
        assert_eq!(c.get(&"a"), Some(&1));
        assert_eq!(c.get(&"c"), Some(&3));
    }

    /// boundary:cap=1——每次 put 新 key 都淘汰唯一住戶。
    #[test]
    fn boundary_cap_one_every_put_evicts() {
        let mut c = LruCache::new(1);
        assert_eq!(c.put(1, "one"), None);
        assert_eq!(c.put(2, "two"), Some((1, "one")));
        assert_eq!(c.get(&1), None);
        assert_eq!(c.get(&2), Some(&"two"));
    }

    /// boundary:同 key 重複 put 是更新不是插入——len 不變、無淘汰、值換新。
    #[test]
    fn boundary_same_key_put_updates_in_place() {
        let mut c = LruCache::new(2);
        c.put("k", 1);
        assert_eq!(c.put("k", 2), None);
        assert_eq!(c.len(), 1);
        assert_eq!(c.get(&"k"), Some(&2));
    }

    /// boundary:get 頭部元素(已是 MRU,promote 短路)不得破壞鏈表。
    #[test]
    fn boundary_get_mru_head_is_noop_promote() {
        let mut c = LruCache::new(2);
        c.put("a", 1);
        c.put("b", 2); // list=[b,a]
        assert_eq!(c.get(&"b"), Some(&2)); // b 已在頭
        assert_eq!(c.put("c", 3), Some(("a", 1))); // 淘汰的仍是 a
    }

    /// peek 不 promote:淘汰對象不受影響。
    #[test]
    fn peek_does_not_promote() {
        let mut c = LruCache::new(2);
        c.put("a", 1);
        c.put("b", 2); // list=[b,a]
        assert_eq!(c.peek(&"a"), Some(&1)); // 不 promote
        assert_eq!(c.put("c", 3), Some(("a", 1))); // a 仍是 LRU
    }

    /// 空 cache 的 get/peek。
    #[test]
    fn boundary_get_on_empty() {
        let mut c: LruCache<i32, i32> = LruCache::new(2);
        assert_eq!(c.get(&1), None);
        assert_eq!(c.peek(&1), None);
    }

    /// O(n) 天真模型:Vec 按 recency 排(頭 = MRU),當 oracle。
    struct NaiveLru {
        entries: Vec<(u8, i32)>,
        cap: usize,
    }
    impl NaiveLru {
        fn get(&mut self, k: u8) -> Option<i32> {
            let pos = self.entries.iter().position(|e| e.0 == k)?;
            let e = self.entries.remove(pos);
            let v = e.1;
            self.entries.insert(0, e);
            Some(v)
        }
        fn put(&mut self, k: u8, v: i32) -> Option<(u8, i32)> {
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

    proptest! {
        /// property:任意 get/put 序列與天真模型完全一致。
        /// key 空間刻意小(0..6)+ cap=3:高碰撞率逼出淘汰/promotion 的交互。
        #[test]
        fn prop_matches_naive_model(ops in proptest::collection::vec((0u8..2, 0u8..6, 0i32..100), 1..300)) {
            let mut lru = LruCache::new(3);
            let mut model = NaiveLru { entries: Vec::new(), cap: 3 };
            for (op, k, v) in ops {
                if op == 0 {
                    prop_assert_eq!(lru.get(&k).copied(), model.get(k));
                } else {
                    prop_assert_eq!(lru.put(k, v), model.put(k, v));
                }
                prop_assert_eq!(lru.len(), model.entries.len());
            }
        }
    }
}
