// ⚠ 防雷:本檔是認題卡 AG-T(聚合樹修復)的填空版,spec 註解含解法方向。
// 排程:8/4 下午卡 AG-T 之後接打(20m,選配);或 8/6 後固化。

//! drill:aggregation_tree —— collector 死亡後的孤兒安置(AG-T 的填空版)。
//!
//! 要填:`rehome` 一支。
//!
//! 核心不變量:
//! - 孤兒 = `children(dead)`,**整棵子樹搬、不拆**;
//! - banned = dead + 全部孤兒子樹(安置目標不得在其中——掛進自己子孫 = 環);
//! - 空位 = 倖存樹 BFS 由淺到深,每節點 `fan_cap − 現任未 banned 孩子數` 格
//!   (dead 與孤兒都 banned ⇒ dead 的 parent 自然多一格,不用特判);
//! - 孤兒按子樹大小**降冪**(同大小 id 升冪)入座;位子不夠 → `Err(NoCapacity)`;
//! - 決定性:children 排序後再 BFS。
//!
//! 完整推導與 trade-offs(淺優先 vs 均衡、bin-packing 聲明)見 reference 同名檔頭。

/// 容量不足:倖存樹的空位總數 < 孤兒數。
#[derive(Debug, PartialEq, Eq)]
pub struct NoCapacity;

/// spec:安置 `dead` 的孩子。回 `(孤兒, 新 parent)` 對,大子樹先。
/// `parent[v] = None` ⇔ root;`dead` 不得為 root(assert)。
pub fn rehome(
    parent: &[Option<u32>],
    fan_cap: usize,
    dead: u32,
) -> Result<Vec<(u32, u32)>, NoCapacity> {
    let d = dead as usize;
    assert!(d < parent.len());
    assert!(parent[d].is_some(), "root 死亡是另一題(clarify 特例)");
    let _ = fan_cap;
    todo!("spec: 建 children → 標 banned+算 size → BFS 收空位 → 大子樹先入座")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "drill:填完 rehome 後拔掉"]
    fn orphans_go_shallow_largest_first() {
        let parent = [None, Some(0), Some(0), Some(1), Some(1), Some(2)];
        let moves = rehome(&parent, 2, 1).unwrap();
        assert_eq!(moves, vec![(3, 0), (4, 2)]);
    }

    #[test]
    #[ignore = "drill:填完 rehome 後拔掉"]
    fn larger_subtree_takes_shallower_slot() {
        let parent = [None, Some(0), Some(0), Some(1), Some(1), Some(3), Some(3)];
        let moves = rehome(&parent, 2, 1).unwrap();
        assert_eq!(moves[0], (3, 0));
        assert_eq!(moves[1], (4, 2));
    }

    #[test]
    #[ignore = "drill:填完 rehome 後拔掉"]
    fn dead_leaf_means_no_moves() {
        let parent = [None, Some(0)];
        assert_eq!(rehome(&parent, 2, 1).unwrap(), Vec::new());
    }

    #[test]
    #[ignore = "drill:填完 rehome 後拔掉"]
    fn no_capacity_is_reported() {
        let parent = [None, Some(0), Some(1), Some(1)];
        assert_eq!(rehome(&parent, 1, 1), Err(NoCapacity));
    }
}
