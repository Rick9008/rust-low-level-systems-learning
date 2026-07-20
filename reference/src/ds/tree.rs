//! # tree —— 同一棵 BST 的兩種寫法:index-based arena vs `Rc<RefCell>`
//!
//! ## [Clarify]
//! 解決:示範 Rust 寫「有共享/回指可能的鏈式結構」的兩條路,並排對照。
//! 載體選 BST(insert / contains / inorder / height),因為它小到能看清結構差異。
//! Constraints:set 語意(重複 insert 忽略)、不做平衡(AVL/紅黑是另一題,
//! 面試先聲明 degenerate 鏈的存在再往前走)。
//!
//! ## [Iterate]
//! 演進順序就是本模組的並排:`rc_refcell`(直覺的「指標」翻譯)→
//! `arena`(索引化之後,借用問題整個消失)。
//!
//! ## [Trade-offs]——兩版逐點對照
//! | | `arena`(索引) | `rc_refcell` |
//! |---|---|---|
//! | 記憶體 | 節點連續放 `Vec`,cache 友善 | 每節點一次 heap alloc + 2 個 refcount 字 |
//! | 借用檢查 | 編譯期,零成本 | **執行期**(RefCell),借用衝突 = panic |
//! | 讀取 API | 可回傳 `&T`(綁 `&self`) | 只能 clone(`Ref` guard 出不了函式) |
//! | parent 指標 | 再存一個 `usize` 即可 | 必須 `Weak`,否則 refcount 環 → 洩漏 |
//! | 刪除節點 | 留洞(需 free list / 世代,見 arena_lockfree) | drop 即回收 |
//! | 深樹 Drop | 迭代釋放 Vec,O(n) 無遞迴 | 遞迴析構,深鏈可能爆 stack |
//!
//! ## [Dry-Run]
//! 兩版跑同一組測試向量:空樹、單節點、退化鏈(height=n)、重複 insert、
//! inorder = 排序序(BST 不變量)。

/// index-based arena 版:節點放 `Vec`,left/right 存 `Option<usize>`。
pub mod arena {
    struct Node<T> {
        val: T,
        left: Option<usize>,
        right: Option<usize>,
    }

    pub struct BstArena<T> {
        nodes: Vec<Node<T>>, // 只 push 不 remove ⇒ 索引永遠有效
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

        /// 迭代 insert。平均 O(log n),退化鏈 O(n)(不平衡,聲明在前)。
        /// 重複值忽略(set 語意)。
        pub fn insert(&mut self, val: T) {
            let new_idx = self.nodes.len();
            let Some(mut cur) = self.root else {
                self.nodes.push(Node {
                    val,
                    left: None,
                    right: None,
                });
                self.root = Some(new_idx);
                return;
            };
            loop {
                match val.cmp(&self.nodes[cur].val) {
                    std::cmp::Ordering::Equal => return,
                    std::cmp::Ordering::Less => match self.nodes[cur].left {
                        Some(l) => cur = l,
                        None => {
                            self.nodes.push(Node {
                                val,
                                left: None,
                                right: None,
                            });
                            self.nodes[cur].left = Some(new_idx);
                            return;
                        }
                    },
                    std::cmp::Ordering::Greater => match self.nodes[cur].right {
                        Some(r) => cur = r,
                        None => {
                            self.nodes.push(Node {
                                val,
                                left: None,
                                right: None,
                            });
                            self.nodes[cur].right = Some(new_idx);
                            return;
                        }
                    },
                }
            }
        }

        /// 平均 O(log n)、退化 O(n)。
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

        /// 中序遍歷,**回傳借用**(arena 版獨有優勢;Rc 版做不到)。
        /// 顯式 stack 迭代:遞迴深度 = 樹高,退化鏈會爆 stack。O(n) 時間/空間。
        pub fn inorder(&self) -> Vec<&T> {
            let mut out = Vec::with_capacity(self.nodes.len());
            let mut stack = Vec::new();
            let mut cur = self.root;
            while cur.is_some() || !stack.is_empty() {
                while let Some(i) = cur {
                    stack.push(i); // 一路向左,沿途壓棧
                    cur = self.nodes[i].left;
                }
                let i = stack.pop().unwrap(); // 最左 → 訪問
                out.push(&self.nodes[i].val);
                cur = self.nodes[i].right; // 轉右子樹
            }
            out
        }

        /// 層序計數求高(空樹 0、單節點 1)。迭代,O(n)。
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
}

/// `Rc<RefCell>` 版:每個節點獨立 heap 配置,child 是共享指標。
pub mod rc_refcell {
    use std::cell::RefCell;
    use std::cmp::Ordering;
    use std::rc::Rc;

    type Link<T> = Option<Rc<RefCell<Node<T>>>>;

    struct Node<T> {
        val: T,
        left: Link<T>,
        right: Link<T>,
    }

    pub struct BstRc<T> {
        root: Link<T>,
        len: usize,
    }

    impl<T: Ord + Clone> Default for BstRc<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T: Ord + Clone> BstRc<T> {
        pub fn new() -> Self {
            Self { root: None, len: 0 }
        }

        pub fn len(&self) -> usize {
            self.len
        }

        pub fn is_empty(&self) -> bool {
            self.len == 0
        }

        fn new_node(val: T) -> Rc<RefCell<Node<T>>> {
            Rc::new(RefCell::new(Node {
                val,
                left: None,
                right: None,
            }))
        }

        /// 迭代 insert。注意每一步的 `borrow()` 區間都刻意縮到最小:
        /// RefCell 是執行期借用檢查,持著 `Ref` 再 `borrow_mut()` 同一節點
        /// 會直接 panic——這類 bug 編譯器不會幫你抓(arena 版沒有此類問題)。
        pub fn insert(&mut self, val: T) {
            let Some(root) = self.root.clone() else {
                self.root = Some(Self::new_node(val));
                self.len = 1;
                return;
            };
            let mut cur = root;
            loop {
                // 先只讀:決定方向並拿到下一步(clone Rc = refcount+1,O(1))
                let next = {
                    let n = cur.borrow();
                    match val.cmp(&n.val) {
                        Ordering::Equal => return,
                        Ordering::Less => n.left.clone(),
                        Ordering::Greater => n.right.clone(),
                    }
                }; // ← Ref 在此釋放,下面才能安全 borrow_mut
                match next {
                    Some(n) => cur = n,
                    None => {
                        let mut n = cur.borrow_mut();
                        if val < n.val {
                            n.left = Some(Self::new_node(val));
                        } else {
                            n.right = Some(Self::new_node(val));
                        }
                        self.len += 1;
                        return;
                    }
                }
            }
        }

        pub fn contains(&self, val: &T) -> bool {
            let mut cur = self.root.clone();
            while let Some(n) = cur {
                let n = n.borrow();
                match val.cmp(&n.val) {
                    Ordering::Equal => return true,
                    Ordering::Less => cur = n.left.clone(),
                    Ordering::Greater => cur = n.right.clone(),
                }
            }
            false
        }

        /// 中序遍歷。**只能回傳 clone**:`Ref<T>` guard 的生命週期綁在
        /// borrow() 呼叫點,無法把 `&T` 帶出函式——這是與 arena 版的關鍵差異。
        pub fn inorder(&self) -> Vec<T> {
            let mut out = Vec::with_capacity(self.len);
            let mut stack: Vec<Rc<RefCell<Node<T>>>> = Vec::new();
            let mut cur = self.root.clone();
            while cur.is_some() || !stack.is_empty() {
                while let Some(n) = cur {
                    cur = n.borrow().left.clone();
                    stack.push(n);
                }
                let n = stack.pop().unwrap();
                let node = n.borrow();
                out.push(node.val.clone());
                cur = node.right.clone();
            }
            out
        }

        /// 層序計數求高。O(n)。
        pub fn height(&self) -> usize {
            let Some(root) = self.root.clone() else {
                return 0;
            };
            let mut height = 0;
            let mut level = vec![root];
            while !level.is_empty() {
                height += 1;
                level = level
                    .iter()
                    .flat_map(|n| {
                        let n = n.borrow();
                        [n.left.clone(), n.right.clone()]
                    })
                    .flatten()
                    .collect();
            }
            height
        }
    }
}

#[cfg(test)]
mod tests {
    use super::arena::BstArena;
    use super::rc_refcell::BstRc;
    use proptest::prelude::*;

    /// [Dry-Run] 兩版同 trace:insert 5,3,8,1
    ///   5 成根;3<5 走左(空)掛左;8>5 掛右;1<5 左、1<3 掛 3 的左
    ///   inorder:1,3 回溯,5,8 → [1,3,5,8](排序序 = BST 不變量)
    ///   height:5(第1層)→3,8(第2層)→1(第3層)= 3
    #[test]
    fn both_versions_same_shape_trace() {
        let mut a = BstArena::new();
        let mut r = BstRc::new();
        for v in [5, 3, 8, 1] {
            a.insert(v);
            r.insert(v);
        }
        assert_eq!(
            a.inorder().into_iter().copied().collect::<Vec<_>>(),
            vec![1, 3, 5, 8]
        );
        assert_eq!(r.inorder(), vec![1, 3, 5, 8]);
        assert_eq!(a.height(), 3);
        assert_eq!(r.height(), 3);
        assert!(a.contains(&8) && r.contains(&8));
        assert!(!a.contains(&4) && !r.contains(&4));
    }

    /// boundary:空樹——height 0、inorder 空、contains false。
    #[test]
    fn boundary_empty_tree() {
        let a: BstArena<i32> = BstArena::new();
        let r: BstRc<i32> = BstRc::new();
        assert_eq!(a.height(), 0);
        assert_eq!(r.height(), 0);
        assert!(a.inorder().is_empty());
        assert!(r.inorder().is_empty());
        assert!(!a.contains(&1));
        assert!(!r.contains(&1));
    }

    /// boundary:單調插入 → 退化鏈,height == n(不平衡的代價可觀察)。
    /// 迭代遍歷在此不爆 stack(遞迴版會在 n 大時出事)。
    #[test]
    fn boundary_degenerate_chain_height_equals_n() {
        let mut a = BstArena::new();
        let mut r = BstRc::new();
        for v in 0..50 {
            a.insert(v);
            r.insert(v);
        }
        assert_eq!(a.height(), 50);
        assert_eq!(r.height(), 50);
    }

    /// boundary:重複 insert 忽略(set 語意)——len 不變。
    #[test]
    fn boundary_duplicate_insert_ignored() {
        let mut a = BstArena::new();
        let mut r = BstRc::new();
        a.insert(7);
        a.insert(7);
        r.insert(7);
        r.insert(7);
        assert_eq!(a.len(), 1);
        assert_eq!(r.len(), 1);
    }

    proptest! {
        /// property:任意插入序,inorder = 去重排序(BST 不變量);兩版一致。
        #[test]
        fn prop_inorder_is_sorted_dedup(vals in proptest::collection::vec(0i32..100, 0..60)) {
            let mut a = BstArena::new();
            let mut r = BstRc::new();
            for &v in &vals {
                a.insert(v);
                r.insert(v);
            }
            let mut expect = vals.clone();
            expect.sort_unstable();
            expect.dedup();
            prop_assert_eq!(a.inorder().into_iter().copied().collect::<Vec<_>>(), expect.clone());
            prop_assert_eq!(r.inorder(), expect);
        }
    }
}
