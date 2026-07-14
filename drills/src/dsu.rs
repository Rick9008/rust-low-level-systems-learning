//! drill:dsu —— 填 find(path compression)與 union(by rank)。
//!
//! 已給:結構、new、connected、components。
//! 要填:`find` / `union`。
//! find 用**兩趟迭代**(遞迴在深鏈會爆 stack):第一趟找根,第二趟重掛。

pub struct Dsu {
    parent: Vec<usize>, // parent[i] == i ⇔ 根
    rank: Vec<u8>,
    components: usize,
}

impl Dsu {
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
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

    pub fn components(&self) -> usize {
        self.components
    }

    /// spec:回傳 x 的根,並做 path compression(沿路節點全部直掛根)。
    /// 兩趟迭代:
    /// 1. root 從 x 沿 parent 爬到 parent[root]==root
    /// 2. 從 x 再走一遍,把每個節點的 parent 改成 root
    ///
    /// 攤銷 O(α(n))。
    pub fn find(&mut self, x: usize) -> usize {
        todo!("spec: 兩趟迭代——先找根,再壓縮路徑")
    }

    /// spec:合併 x、y 所在集合。回傳是否真的合併(同集合 → false)。
    /// union by rank:矮樹掛高樹;同 rank 任選一邊掛、掛完 rank+1。
    /// 成功合併時 components -= 1。
    pub fn union(&mut self, x: usize, y: usize) -> bool {
        todo!("spec: find 兩根; 相同 → false; by rank 掛樹; components 遞減")
    }

    pub fn connected(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// boundary:鏈狀 union 後全部連通、components 正確。
    #[test]
    #[ignore = "填完 find/union 後移除"]
    fn chain_union_connectivity() {
        let mut d = Dsu::new(6);
        for i in 0..5 {
            assert!(d.union(i, i + 1));
        }
        assert!(d.connected(0, 5));
        assert_eq!(d.components(), 1);
    }

    /// boundary:自反 union、重複 union 都是 no-op。
    #[test]
    #[ignore = "填完 find/union 後移除"]
    fn self_and_duplicate_union() {
        let mut d = Dsu::new(3);
        assert!(!d.union(1, 1));
        assert!(d.union(0, 1));
        assert!(!d.union(1, 0));
        assert_eq!(d.components(), 2);
    }

    /// path compression 可觀察:find 過後沿路節點距根 ≤ 1 步。
    #[test]
    #[ignore = "填完 find/union 後移除"]
    fn compression_flattens() {
        let mut d = Dsu::new(6);
        for i in 0..5 {
            d.union(i, i + 1);
        }
        let root = d.find(0);
        for i in 0..6 {
            d.find(i);
            assert!(d.parent[i] == root || i == root, "node {i} 未壓縮");
        }
    }
}
