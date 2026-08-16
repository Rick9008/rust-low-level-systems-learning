//! 第 1 關：Disjoint Set Union（Union-Find）。
//!
//! `Dsu` 擁有 `parent`、`rank` 兩個 Vec。`find` 需要修改 parent 來壓縮路徑，
//! 所以接收的是 `&mut self`。

pub struct Dsu {
    parent: Vec<usize>,
    rank: Vec<u8>,
    components: usize,
}

impl Dsu {
    pub fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
            components: size,
        }
    }

    pub fn components(&self) -> usize {
        self.components
    }

    /// 找到 x 所屬集合的根，並做兩趟式 path compression。
    ///
    /// 第一趟：只讀 parent，找出 root。
    /// 第二趟：沿原路修改 parent，讓每個節點直接指向 root。
    pub fn find(&mut self, x: usize) -> usize {
        todo!("先找 root，再走一次路徑並修改 parent")
    }

    /// 合併 x、y 所屬的集合。真的發生合併時回傳 true。
    pub fn union(&mut self, x: usize, y: usize) -> bool {
        todo!("分開呼叫兩次 find，再用 rank 決定哪個 root 接到另一個 root")
    }

    pub fn connected(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dsu_connects_groups() {
        let mut dsu = Dsu::new(6);
        assert!(dsu.union(0, 1));
        assert!(dsu.union(1, 2));
        assert!(dsu.union(3, 4));
        assert!(dsu.connected(0, 2));
        assert!(!dsu.connected(0, 3));
        assert_eq!(dsu.components(), 3);
    }

    #[test]
    fn dsu_duplicate_union_is_a_no_op() {
        let mut dsu = Dsu::new(3);
        assert!(dsu.union(0, 1));
        assert!(!dsu.union(1, 0));
        assert!(!dsu.union(2, 2));
        assert_eq!(dsu.components(), 2);
    }

    #[test]
    fn dsu_find_compresses_the_path() {
        let mut dsu = Dsu::new(5);
        dsu.parent = vec![1, 2, 3, 4, 4];
        assert_eq!(dsu.find(0), 4);
        assert_eq!(dsu.parent, vec![4, 4, 4, 4, 4]);
    }
}

