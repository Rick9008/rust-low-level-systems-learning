//! 第 2 關：使用 arena 儲存 Binary Search Tree。
//!
//! `BstArena` 擁有所有 Node。Node 之間只記錄 Vec 索引，不保存彼此的引用。

use std::cmp::Ordering;

struct Node<T> {
    value: T,
    left: Option<usize>,
    right: Option<usize>,
}

pub struct BstArena<T> {
    nodes: Vec<Node<T>>,
    root: Option<usize>,
}

impl<T: Ord> BstArena<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// 插入新值。重複值不插入。
    ///
    /// 提示：先讀出目前節點的比較結果與下一個索引，讓不可變借用結束，
    /// 再 push 新節點或修改父節點的 left/right。
    pub fn insert(&mut self, value: T) {
        todo!("處理空樹，再從 root 迭代尋找新節點的位置")
    }

    pub fn contains(&self, target: &T) -> bool {
        let mut current = self.root;

        while let Some(index) = current {
            match target.cmp(&self.nodes[index].value) {
                Ordering::Less => current = self.nodes[index].left,
                Ordering::Equal => return true,
                Ordering::Greater => current = self.nodes[index].right,
            }
        }

        false
    }

    /// 使用顯式 stack 做 inorder traversal，回傳節點值的借用。
    pub fn inorder(&self) -> Vec<&T> {
        todo!("一路向左 push 索引，pop 後收集 value，再走向右子樹")
    }
}

impl<T: Ord> Default for BstArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_inorder_is_sorted() {
        let mut tree = BstArena::new();
        for value in [5, 3, 8, 1, 4, 7, 9] {
            tree.insert(value);
        }

        let values: Vec<i32> = tree.inorder().into_iter().copied().collect();
        assert_eq!(values, vec![1, 3, 4, 5, 7, 8, 9]);
    }

    #[test]
    fn tree_ignores_duplicates_and_searches() {
        let mut tree = BstArena::new();
        tree.insert(4);
        tree.insert(2);
        tree.insert(4);

        assert_eq!(tree.len(), 2);
        assert!(tree.contains(&2));
        assert!(!tree.contains(&3));
    }

    #[test]
    fn tree_empty_inorder_is_empty() {
        let tree: BstArena<i32> = BstArena::new();
        assert!(tree.inorder().is_empty());
    }
}

