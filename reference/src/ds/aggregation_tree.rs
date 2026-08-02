//! # aggregation_tree —— 聚合樹修復:collector 死亡後的孤兒安置(認題卡 AG-T 的正式版)
//!
//! ## [Clarify]
//! 解決:sensor → leaf collector → … → root 的聚合樹,每個 collector 有
//! fan-in 上限 `F`。collector `d` 死了,把它的孩子(**整棵子樹,不拆**)安置到
//! 倖存節點上,不得超過任何節點的 fan-in。
//! Constraints:`parent[v] = None` ⇔ root;**root 死亡是另一題**(assert 擋掉,
//! clarify 時點名);目標函數 spec 沉默——**必問「最小化什麼」**,本實作採
//! 「淺優先安置、大子樹先挑位」的 greedy(控制深度增長)。
//!
//! ## [Abstract]
//! 完美最小化(深度/負載雙目標)是 bin-packing 家族——聲明不做,greedy +
//! 理由即滿分;安置後的負載重平衡、連鎖故障(安置目標也死)不做。
//!
//! ## [Iterate]
//! naive:每個孤兒重掃全樹找位 → O(孤兒數 × V)。
//! 正解:一趟 BFS 由淺到深收集空位(slack = F − 現任孩子數;**dead 的
//! parent 因為 dead 離隊而多出一格**),孤兒按子樹大小降冪逐一入座。
//! O(V + E + 孤兒數 log 孤兒數)。
//!
//! ## [Trade-offs]
//! - **安置目標排除孤兒子樹內部**:掛到自己子孫底下 = 環(樹解體)。排除整片
//!   孤兒區最乾淨——也自動排除「A 掛進 B 的子樹、B 又被搬走」的糾纏。
//! - **淺優先 vs 均衡優先**:淺優先壓深度增長(聚合延遲),代價是負載集中;
//!   latency-sensitive 選前者、throughput-sensitive 選後者——收尾句素材。
//! - **大子樹先挑位**:讓最重的孤兒拿到最淺的位子,深度增長的上界最小;
//!   同大小以 id 升冪破平手(決定性輸出)。
//!
//! ## [Dry-Run]
//! 手排 trace 見 [`tests::orphans_go_shallow_largest_first`];boundary:
//! 容量不足 → Err、葉節點死亡(零孤兒)、dead 的 parent 空位被用上。

/// 容量不足:倖存樹的空位總數 < 孤兒數。
#[derive(Debug, PartialEq, Eq)]
pub struct NoCapacity;

/// 安置 `dead` 的孩子(整棵子樹)。回傳 `(孤兒, 新 parent)` 對,
/// 依安置順序(大子樹先)。`parent[v] = None` ⇔ root;`dead` 不得為 root。
pub fn rehome(
    parent: &[Option<u32>],
    fan_cap: usize,
    dead: u32,
) -> Result<Vec<(u32, u32)>, NoCapacity> {
    let n = parent.len();
    let d = dead as usize;
    assert!(d < n);
    assert!(parent[d].is_some(), "root 死亡是另一題(clarify 特例)");

    let mut children = vec![Vec::new(); n];
    let mut root = usize::MAX;
    for (v, p) in parent.iter().enumerate() {
        match p {
            Some(p) => children[*p as usize].push(v),
            None => root = v,
        }
    }
    for c in &mut children {
        c.sort_unstable(); // 決定性輸出
    }

    // 孤兒與其子樹大小;banned = dead + 全部孤兒子樹(安置目標不得在其中)。
    let orphans: Vec<usize> = children[d].clone();
    let mut banned = vec![false; n];
    banned[d] = true;
    let mut sizes = vec![0usize; n];
    for &o in &orphans {
        // 迭代 DFS 算子樹大小,同時標 banned。
        let mut stack = vec![o];
        while let Some(u) = stack.pop() {
            banned[u] = true;
            sizes[o] += 1;
            stack.extend(children[u].iter().copied());
        }
    }

    // 空位收集:由淺到深 BFS 倖存樹;dead 的 parent 因 dead 離隊 +1 格。
    let mut slots: Vec<usize> = Vec::new(); // 依淺→深展開,同節點的多格連續
    let mut queue = std::collections::VecDeque::from([root]);
    while let Some(u) = queue.pop_front() {
        if banned[u] {
            continue;
        }
        // dead 與孤兒子樹都 banned ⇒ dead 的 parent 自然多出一格,不用特判。
        let occupied = children[u].iter().filter(|&&c| !banned[c]).count();
        for _ in occupied..fan_cap {
            slots.push(u);
        }
        for &c in &children[u] {
            if !banned[c] {
                queue.push_back(c);
            }
        }
    }

    if slots.len() < orphans.len() {
        return Err(NoCapacity);
    }

    // 大子樹先挑位(同大小 id 升冪)。
    let mut order = orphans.clone();
    order.sort_unstable_by_key(|&o| (std::cmp::Reverse(sizes[o]), o));

    Ok(order
        .into_iter()
        .zip(slots)
        .map(|(o, s)| (o as u32, s as u32))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 手排 trace:F=2,樹 root=0,0 的孩子 {1, 2},1 的孩子 {3, 4},
    /// 2 的孩子 {5}。dead=1 → 孤兒 {3(葉,size 1), 4(葉,size 1)}。
    /// - banned = {1, 3, 4};倖存樹 BFS:0 → 2 → 5。
    /// - 空位:0 的現任=({1,2} 去掉 banned 1)={2} → 1 格;2 的現任={5} → 1 格;
    ///   5 → 2 格。slots(淺→深)= [0, 2, 5, 5]。
    /// - 孤兒排序(同 size,id 升冪)= [3, 4] → 3→0、4→2。
    #[test]
    fn orphans_go_shallow_largest_first() {
        let parent = [None, Some(0), Some(0), Some(1), Some(1), Some(2)];
        let moves = rehome(&parent, 2, 1).unwrap();
        assert_eq!(moves, vec![(3, 0), (4, 2)]);
    }

    /// 大子樹先挑最淺的位:dead=1 的孤兒 3(帶子樹 {3,5,6},size 3)與 4(葉)。
    /// F=2:root 0 空 1 格(1 離隊)、2 空 1 格。大的 3 拿 root 位。
    #[test]
    fn larger_subtree_takes_shallower_slot() {
        //      0
        //    1   2
        //  3  4      (3 的子樹:5、6)
        let parent = [None, Some(0), Some(0), Some(1), Some(1), Some(3), Some(3)];
        let moves = rehome(&parent, 2, 1).unwrap();
        assert_eq!(moves[0], (3, 0), "size-3 的孤兒先拿最淺位");
        assert_eq!(moves[1], (4, 2));
    }

    #[test]
    fn dead_leaf_means_no_moves() {
        let parent = [None, Some(0)];
        assert_eq!(rehome(&parent, 2, 1).unwrap(), Vec::new());
    }

    #[test]
    fn no_capacity_is_reported() {
        // F=1:root 0 唯一孩子是 1;1 有孩子 {2, 3}。dead=1 → 孤兒 2 個,
        // 倖存樹只有 root,slack = 1(1 離隊)→ 塞不下。
        let parent = [None, Some(0), Some(1), Some(1)];
        assert_eq!(rehome(&parent, 1, 1), Err(NoCapacity));
    }
}
