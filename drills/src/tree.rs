//! drill:tree(arena BST)—— 填 insert 與迭代 inorder。
//!
//! 已給:結構、contains、height。Rc<RefCell> 對照版不重複挖
//! (讀 reference/src/tree.rs 的 rc_refcell 模組,重點是並排差異)。
//! 要填:`insert` / `inorder`(顯式 stack,禁遞迴——深鏈會爆)。

struct Node<T> {
    val: T,
    left: Option<usize>,
    right: Option<usize>,
}

pub struct BstArena<T> {
    nodes: Vec<Node<T>>,
    root: Option<usize>,
}

impl<T: Ord> Default for BstArena<T> {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// spec:迭代 BST insert(set 語意:重複值直接 return)。
    /// 新節點永遠 push 在 nodes 尾端(索引 = push 前 nodes.len()),
    /// 再把父節點的 left/right 指過來;空樹直接設 root。
    /// 平均 O(log n)、退化鏈 O(n)。
    pub fn insert(&mut self, val: T) {
        todo!("spec: 空樹設 root;否則迭代走,Equal return、Less 左、Greater 右,掛葉")
    }

    /// spec:迭代中序遍歷(顯式 stack),回傳值的借用(&T)。
    /// 形狀:cur 一路向左沿途壓棧 → pop 訪問 → 轉向右子樹。
    /// 結果必為排序序(BST 不變量)。O(n)。
    pub fn inorder(&self) -> Vec<&T> {
        todo!("spec: while cur.is_some() || !stack.is_empty() 的雙迴圈")
    }

    pub fn contains(&self, val: &T) -> bool {
        let mut cur = self.root;
        while let Some(i) = cur {
            match val.cmp(&self.nodes[i].val) {
                std::cmp::Ordering::Equal => return true,
                std::cmp::Ordering::Less => cur = self.nodes[i].left,
                std::cmp::Ordering::Greater => cur = self.nodes[i].right,
            }
        }
        false
    }

    pub fn height(&self) -> usize {
        let Some(root) = self.root else { return 0 };
        let mut height = 0;
        let mut level = vec![root];
        while !level.is_empty() {
            height += 1;
            level = level
                .iter()
                .flat_map(|&i| [self.nodes[i].left, self.nodes[i].right])
                .flatten()
                .collect();
        }
        height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// boundary:inorder = 排序序;重複忽略。
    #[test]
    #[ignore = "填完 insert/inorder 後移除"]
    fn inorder_is_sorted_dedup() {
        let mut t = BstArena::new();
        for v in [5, 3, 8, 1, 3, 5] {
            t.insert(v);
        }
        assert_eq!(t.len(), 4);
        assert_eq!(
            t.inorder().into_iter().copied().collect::<Vec<_>>(),
            vec![1, 3, 5, 8]
        );
    }

    /// boundary:空樹、單調退化鏈(height == n,inorder 不爆 stack)。
    #[test]
    #[ignore = "填完 insert/inorder 後移除"]
    fn empty_and_degenerate_chain() {
        let empty: BstArena<i32> = BstArena::new();
        assert!(empty.inorder().is_empty());

        let mut chain = BstArena::new();
        for v in 0..100 {
            chain.insert(v);
        }
        assert_eq!(chain.height(), 100);
        assert_eq!(chain.inorder().len(), 100);
    }
}
