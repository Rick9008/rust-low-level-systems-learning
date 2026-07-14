//! # dsu —— union-find(path compression + union by rank)
//!
//! ## [Clarify]
//! 解決:動態維護「哪些元素在同一集合」,支援 union(合併)與 find(查代表)。
//! 典型場景:連通性、Kruskal MST、等價類、網路 partition 偵測。
//! Constraints:元素是 `0..n` 的整數(外部型別自己先映射到索引);
//! 只合不拆(不支援 un-union——那需要 rollback/offline 技巧,面試先聲明不做)。
//!
//! ## [Abstract]
//! 「外部實體 → 索引」的映射 stub 掉(caller 拿 HashMap 自己做),
//! 本模組只管索引世界——面試時先把介面定在 usize 往前走。
//!
//! ## [Iterate]
//! naive find = 沿 parent 爬到根,最壞 O(n)(退化成鏈)。
//! 兩個優化疊加後攤銷 O(α(n)),α = 反 Ackermann,宇宙尺度 n 也 ≤ 4,實務視為常數:
//! 1. **path compression**:find 途中把沿路節點直接掛到根
//! 2. **union by rank**:矮樹掛到高樹下,樹高只在同 rank 相遇時 +1
//!
//! ## [Trade-offs]
//! - rank 存 `u8` 夠:rank 只在兩棵同 rank 樹合併時 +1,樹高 ≥ 2^rank,
//!   u8 上限 255 ⇒ 要溢位得有 2^255 個元素。空間 n + n/8 字。
//! - path compression 選「兩趟迭代」而非遞迴:遞迴深度 = 樹高,
//!   compression 前可能 O(n),1e6 元素的鏈會爆 stack。
//! - by rank vs by size:等效攤銷界;size 版順便提供集合大小查詢,
//!   rank 版少存一個 usize(u8 vs usize)。這裡按面試題面用 rank。
//!
//! ## [Dry-Run]
//! 見測試:自反 union、重複 union、鏈狀合併後 find 全部指根(壓縮可觀察)、
//! n=1 退化、components 計數、proptest 對照天真連通標號模型。

pub struct Dsu {
    parent: Vec<usize>,
    rank: Vec<u8>,
    components: usize,
}

impl Dsu {
    /// n 個 singleton。O(n)。
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(), // parent[i] == i ⇔ i 是根
            rank: vec![0; n],
            components: n,
        }
    }

    pub fn len(&self) -> usize {
        self.parent.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parent.is_empty()
    }

    /// 目前集合數。O(1)(union 成功時遞減維護)。
    pub fn components(&self) -> usize {
        self.components
    }

    /// 查代表(根)。攤銷 O(α(n))。
    ///
    /// 兩趟迭代式 path compression:
    /// 第一趟爬到根;第二趟把沿路每個節點的 parent 直接改成根。
    /// (遞迴版一行 `parent[x] = find(parent[x])` 更短,但深鏈爆 stack。)
    pub fn find(&mut self, mut x: usize) -> usize {
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        // 第二趟:沿路全部直掛根,下次 find 這條路徑 O(1)
        while self.parent[x] != root {
            let next = self.parent[x];
            self.parent[x] = root;
            x = next;
        }
        root
    }

    /// 合併 x, y 所在集合。攤銷 O(α(n))。回傳是否真的發生合併
    /// (false = 本來就同集合)。
    pub fn union(&mut self, x: usize, y: usize) -> bool {
        let (rx, ry) = (self.find(x), self.find(y));
        if rx == ry {
            return false;
        }
        // union by rank:矮樹掛高樹,樹高不變;同 rank 時任選一邊、rank+1。
        // 沒有這步,鏈狀 union 讓樹高 O(n),find 退化 O(n)。
        let (low, high) = if self.rank[rx] < self.rank[ry] {
            (rx, ry)
        } else {
            (ry, rx)
        };
        self.parent[low] = high;
        if self.rank[low] == self.rank[high] {
            self.rank[high] += 1;
        }
        self.components -= 1;
        true
    }

    /// 同集合判定。攤銷 O(α(n))。
    pub fn connected(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// [Dry-Run] 手 trace(n=4):
    ///   union(0,1):同 rank 0 → 0 掛 1(或反向,實作取 high=rx=0…
    ///   本實作 rank 相等時 (low,high)=(ry,rx) ⇒ 1 掛 0,rank[0]=1)
    ///   union(1,2):find(1)=0,find(2)=2;rank[0]=1 > rank[2]=0 ⇒ 2 掛 0
    ///   connected(0,2) → find 兩邊都是 0 → true
    ///   components:4 → 3 → 2
    #[test]
    fn union_then_connected_trace() {
        let mut d = Dsu::new(4);
        assert_eq!(d.components(), 4);
        assert!(d.union(0, 1));
        assert!(d.union(1, 2));
        assert!(d.connected(0, 2));
        assert!(!d.connected(0, 3));
        assert_eq!(d.components(), 2); // {0,1,2} 與 {3}
    }

    /// boundary:自反 union(x,x)與重複 union——都不得改變 components。
    #[test]
    fn boundary_self_and_duplicate_union_are_noops() {
        let mut d = Dsu::new(3);
        assert!(!d.union(1, 1)); // 自反:本來就同集合
        assert!(d.union(0, 1));
        assert!(!d.union(0, 1)); // 重複
        assert!(!d.union(1, 0)); // 反向重複
        assert_eq!(d.components(), 2);
    }

    /// boundary:n=1 與 n=0 退化。
    #[test]
    fn boundary_tiny_sizes() {
        let mut one = Dsu::new(1);
        assert_eq!(one.find(0), 0);
        assert_eq!(one.components(), 1);
        let zero = Dsu::new(0);
        assert_eq!(zero.components(), 0);
        assert!(zero.is_empty());
    }

    /// path compression 可觀察:鏈狀 union 後,對鏈尾 find 一次,
    /// 沿路全部直掛根 ⇒ 再 find 任一節點只走 1 步。
    /// 這裡用行為驗證(全部 find 到同一根)+ 內部不變量(parent[x] 距根 ≤ 1)。
    #[test]
    fn path_compression_flattens_chain() {
        let mut d = Dsu::new(6);
        for i in 0..5 {
            d.union(i, i + 1);
        }
        let root = d.find(0);
        for i in 0..6 {
            assert_eq!(d.find(i), root);
        }
        // find 過後每個節點最多離根 1 步(root 自己 0 步)
        for i in 0..6 {
            let p = d.parent[i];
            assert!(p == root || p == i && i == root, "node {i} not flattened");
        }
    }

    // 天真模型:label 陣列,union = O(n) 全表重標。當 oracle。
    proptest! {
        #[test]
        fn prop_matches_naive_labeling(unions in proptest::collection::vec((0usize..12, 0usize..12), 0..40)) {
            let mut d = Dsu::new(12);
            let mut label: Vec<usize> = (0..12).collect();
            for (a, b) in unions {
                d.union(a, b);
                let (la, lb) = (label[a], label[b]);
                if la != lb {
                    for l in label.iter_mut() {
                        if *l == lb { *l = la; }
                    }
                }
            }
            // 每一對的連通性都要一致
            for i in 0..12 {
                for j in 0..12 {
                    prop_assert_eq!(d.connected(i, j), label[i] == label[j], "pair ({}, {})", i, j);
                }
            }
            // components 數 = 相異 label 數
            let mut labels: Vec<usize> = label.clone();
            labels.sort_unstable();
            labels.dedup();
            prop_assert_eq!(d.components(), labels.len());
        }
    }
}
