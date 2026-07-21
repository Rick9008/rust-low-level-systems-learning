//! # list_fine —— 交手鎖(hand-over-hand / lock coupling)排序鏈表
//!
//! ## [Clarify]
//! 解決:多執行緒共享的排序 set(contains / insert / remove),
//! 鎖粒度是**每節點一把 Mutex**,走訪時「鎖下一個、才放上一個」。
//! 這是全域鎖與無鎖之間**缺的那一階**:不同執行緒可以同時在鏈表的
//! 不同區段上工作(pipeline 並行)。
//! Constraints:容量固定(index arena);key 是 i64,`i64::MIN` 保留給哨兵;
//! 只支援 set 語意(無 payload——加 value 是機械擴充)。
//!
//! ## [Abstract]
//! 「排序」是為了讓 remove/insert 能在走訪中確定位置提早停;
//! 集合語意之外的一切(迭代器、range 查詢)都不做——
//! 交手鎖的迭代器會把鎖持有權暴露給 caller,是 API 災難,面試先聲明不提供。
//!
//! ## [Iterate]
//! 鎖階梯(Herlihy 教科書的經典演進,本 repo 各階對應):
//! 1. **coarse**:`Mutex<整個結構>`——arena_locked / lru_locked 的做法
//! 2. **fine(本模組)**:每節點一鎖 + 交手——並行度來自「不同區段互不擋」
//! 3. optimistic / lazy:先無鎖走訪、到點才鎖 + 驗證(或 mark 後懶刪)——
//!    需要「節點被摘走後記憶體仍可讀」的保證,index arena 天然給
//! 4. lock-free(Harris list):CAS + mark bit 邏輯刪除——研究級,見 docs
//!
//! ## [Trade-offs]
//! - **死鎖自由的證明**:所有執行緒都按「鏈表位置順序」拿鎖(全序),
//!   free-list 鎖是葉鎖(持有期間不再拿其他鎖)⇒ 等待圖無環。
//!   交手鎖的正確性繫在這條全序上——任何「回頭鎖前面節點」的操作都禁止。
//! - **unlink 為什麼要同時持 prev + cur 兩把鎖**:改 `prev.next` 需要 prev 鎖;
//!   持有 prev 鎖同時擋掉了「別人 unlink cur」(那也需要 prev 鎖)——
//!   所以 splice 點只需 prev 鎖,remove 需要 prev+cur(cur 的內容要讀)。
//! - 每一步都付一次 lock/unlock:**單點熱點下比一把大鎖還慢**
//!   (鎖多 ≠ 快)。贏面只在「長鏈 + 存取分散」——pipeline 並行才成立。
//! - `len` 是 Relaxed 計數快照(同 dsu_lockfree 的 components,不當同步旗標)。
//! - 時間 O(n) 走訪 + 每步鎖開銷;空間 O(cap)。
//!
//! ## [Dry-Run]
//! 見 `sorted_set_basic_trace` 的手 trace;哨兵邊界(插最小/最大 key)、
//! 滿/空、重複;並發:不相交 key 8 執行緒 + 混合 insert/remove 錘,
//! 收斂後驗證「排序、無重複、計數一致」三不變量。

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// 哨兵節點索引(永遠在鏈頭,key = i64::MIN,不可被移除)。
const HEAD: u32 = 0;
/// 鏈尾哨兵值。
const NIL: u32 = u32::MAX;

struct Node {
    key: i64,
    next: u32,
}

/// 交手鎖排序 set。所有操作 `&self`。
pub struct FineList {
    /// nodes[0] 是哨兵;其餘 cap 個是可分配槽位。
    nodes: Box<[Mutex<Node>]>,
    /// 可分配槽位(葉鎖:持有期間不得再拿任何節點鎖)。
    free: Mutex<Vec<u32>>,
    /// 元素數。Relaxed 統計計數——快照語意,不當同步旗標。
    count: AtomicUsize,
}

impl FineList {
    /// 容量固定 `cap`。O(cap) 初始化。
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0, "capacity must be at least 1");
        assert!(
            u32::try_from(cap + 1).is_ok() && (cap as u32) < NIL,
            "index space is u32 with MAX reserved as NIL"
        );
        let nodes: Box<[Mutex<Node>]> = (0..=cap)
            .map(|i| {
                Mutex::new(Node {
                    key: if i == 0 { i64::MIN } else { 0 },
                    next: NIL,
                })
            })
            .collect();
        Self {
            nodes,
            free: Mutex::new((1..=cap as u32).rev().collect()),
            count: AtomicUsize::new(0),
        }
    }

    pub fn capacity(&self) -> usize {
        self.nodes.len() - 1
    }

    /// 元素數(快照語意)。O(1)。
    pub fn len(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 是否包含 key。O(n) + 每步一次交手。
    ///
    /// 交手的本體就是 `prev = cur` 這行:guard 賦值 = 舊鎖釋放、新鎖已持
    /// ——任何瞬間至少持有一把鎖,走訪路徑上不會被人抽走腳下的節點。
    pub fn contains(&self, key: i64) -> bool {
        let mut prev = self.nodes[HEAD as usize].lock().unwrap();
        loop {
            let cur_idx = prev.next;
            if cur_idx == NIL {
                return false;
            }
            let cur = self.nodes[cur_idx as usize].lock().unwrap();
            if cur.key >= key {
                return cur.key == key;
            }
            prev = cur; // 交手:舊 prev 的鎖在賦值時釋放
        }
    }

    /// 插入。`Ok(true)` 新插入、`Ok(false)` 已存在、`Err(key)` 滿。
    /// O(n) + 每步交手;splice 只需 prev 鎖(不變量見模組 doc)。
    pub fn insert(&self, key: i64) -> Result<bool, i64> {
        assert!(key > i64::MIN, "i64::MIN 保留給哨兵");
        let mut prev = self.nodes[HEAD as usize].lock().unwrap();
        loop {
            let cur_idx = prev.next;
            if cur_idx != NIL {
                let cur = self.nodes[cur_idx as usize].lock().unwrap();
                if cur.key == key {
                    return Ok(false);
                }
                if cur.key < key {
                    prev = cur; // 交手前進
                    continue;
                }
                // cur.key > key:插點就在 prev 之後。cur 鎖在此釋放——
                // 持 prev 鎖已擋掉「別人 unlink cur」(那需要 prev 鎖)。
            }
            // 取槽位:free 是葉鎖,拿它時只持 prev(節點鎖 → 葉鎖,無環)。
            let Some(new_idx) = self.free.lock().unwrap().pop() else {
                return Err(key); // 滿:bounded 語意
            };
            {
                // 新節點還不在鏈上,無人可及;上鎖只為滿足型別,必不競爭。
                let mut new = self.nodes[new_idx as usize].lock().unwrap();
                new.key = key;
                new.next = cur_idx;
            }
            prev.next = new_idx; // 發佈:持 prev 鎖的人才看得到新節點
            self.count.fetch_add(1, Ordering::Relaxed);
            return Ok(true);
        }
    }

    /// 移除。回傳是否真的移除。O(n) + 每步交手;
    /// unlink 同時持 prev + cur 兩把鎖(這正是交手鎖存在的理由)。
    pub fn remove(&self, key: i64) -> bool {
        let mut prev = self.nodes[HEAD as usize].lock().unwrap();
        loop {
            let cur_idx = prev.next;
            if cur_idx == NIL {
                return false;
            }
            let cur = self.nodes[cur_idx as usize].lock().unwrap();
            if cur.key > key {
                return false; // 排序讓我們提早停
            }
            if cur.key == key {
                prev.next = cur.next; // unlink:此刻同時持 prev + cur
                drop(cur);
                drop(prev); // 先放節點鎖、再拿葉鎖:維持鎖序
                self.free.lock().unwrap().push(cur_idx);
                self.count.fetch_sub(1, Ordering::Relaxed);
                return true;
            }
            prev = cur; // 交手前進
        }
    }

    /// 測試用:交手走訪收集所有 key(照鏈序)。
    #[cfg(test)]
    fn collect(&self) -> Vec<i64> {
        let mut out = Vec::new();
        let mut prev = self.nodes[HEAD as usize].lock().unwrap();
        loop {
            let cur_idx = prev.next;
            if cur_idx == NIL {
                return out;
            }
            let cur = self.nodes[cur_idx as usize].lock().unwrap();
            out.push(cur.key);
            prev = cur;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// [Dry-Run] 手 trace(cap=4):
    ///   insert(5):prev=哨兵,next=NIL → 取槽 1,new{5,NIL},哨兵.next=1
    ///   insert(2):prev=哨兵,cur=槽1(key 5)≥ 2 且 ≠ → 插 prev 後:
    ///             取槽 2,new{2, next=1},哨兵.next=2 → 鏈序 [2, 5]
    ///   insert(5):走到槽 1,cur.key == 5 → Ok(false),鏈不動
    ///   remove(2):prev=哨兵,cur=槽2 key==2 → 哨兵.next=1,槽 2 回 free
    ///   collect → [5]
    #[test]
    fn sorted_set_basic_trace() {
        let l = FineList::new(4);
        assert_eq!(l.insert(5), Ok(true));
        assert_eq!(l.insert(2), Ok(true));
        assert_eq!(l.collect(), vec![2, 5]);
        assert_eq!(l.insert(5), Ok(false)); // 重複
        assert!(l.remove(2));
        assert!(!l.remove(2)); // 已移除
        assert_eq!(l.collect(), vec![5]);
        assert_eq!(l.len(), 1);
    }

    /// boundary:空表 remove/contains、滿 → Err(key)、槽位回收後再插。
    #[test]
    fn boundary_empty_full_recycle() {
        let l = FineList::new(2);
        assert!(!l.remove(1));
        assert!(!l.contains(1));
        assert_eq!(l.insert(1), Ok(true));
        assert_eq!(l.insert(2), Ok(true));
        assert_eq!(l.insert(3), Err(3)); // 滿
        assert!(l.remove(1));
        assert_eq!(l.insert(3), Ok(true)); // 回收重用
        assert_eq!(l.collect(), vec![2, 3]);
    }

    /// boundary:極值 key(哨兵是 i64::MIN,MIN+1 與 MAX 都要能住)。
    #[test]
    fn boundary_extreme_keys() {
        let l = FineList::new(3);
        assert_eq!(l.insert(i64::MAX), Ok(true));
        assert_eq!(l.insert(i64::MIN + 1), Ok(true));
        assert_eq!(l.insert(0), Ok(true));
        assert_eq!(l.collect(), vec![i64::MIN + 1, 0, i64::MAX]);
    }

    /// 並發煙霧測試:8 執行緒插不相交 key 段,收斂後三不變量:
    /// 排序、無重複、len 一致。
    #[test]
    fn concurrent_disjoint_inserts_sorted_unique() {
        let l = Arc::new(FineList::new(800));
        let handles: Vec<_> = (0..8i64)
            .map(|t| {
                let l = Arc::clone(&l);
                thread::spawn(move || {
                    for i in 0..100 {
                        assert_eq!(l.insert(t * 100 + i), Ok(true));
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        let got = l.collect();
        assert_eq!(got.len(), 800);
        assert!(got.windows(2).all(|w| w[0] < w[1]), "排序且無重複");
        assert_eq!(l.len(), 800);
    }

    /// 混合錘:同 key 空間上並發 insert/remove,收斂後只驗結構不變量
    /// (排序、無重複、len 與實際一致)——結果集合依交錯而異,不斷言內容。
    #[test]
    fn concurrent_mixed_hammer_invariants_hold() {
        let l = Arc::new(FineList::new(64));
        let handles: Vec<_> = (0..4i64)
            .map(|t| {
                let l = Arc::clone(&l);
                thread::spawn(move || {
                    for round in 0..50 {
                        for k in 0..16 {
                            if (t + round + k) % 3 == 0 {
                                let _ = l.remove(k);
                            } else {
                                let _ = l.insert(k);
                            }
                        }
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        let got = l.collect();
        assert!(got.windows(2).all(|w| w[0] < w[1]), "排序且無重複");
        assert_eq!(got.len(), l.len(), "計數與實際一致");
    }
}
