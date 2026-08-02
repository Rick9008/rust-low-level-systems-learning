// ⚠ 防雷:本檔是認題卡 AG-R(widest path)的填空版,spec 註解含解法方向。
// 排程:8/4 下午卡 AG-R 之後接打(20m,選配);或 8/6 後固化。

//! drill:route_planner —— widest path(max-min 頻寬;AG-R 的填空版)。
//!
//! 要填:`widest_path` 一支。
//!
//! 核心不變量(Dijkstra 骨架,**兩處變形**):
//! - 鬆弛:`cand = min(bott[u], w)`,`cand > bott[v]` 才更新(min 取代 +);
//! - 堆:**max-heap——std `BinaryHeap` 原生就是,不包 `Reverse`**(和最短路鏡像);
//! - 懶刪除:pop 出來 `b < bott[u]` 就 continue;`u == to` 即可 break(貪婪定案);
//! - `from == to` 瓶頸定義 `u64::MAX`、路徑 `[from]`;不連通回 `None`;
//! - 無向:每邊建兩個方向。
//!
//! 完整推導與貪婪論證見 reference 同名模組檔頭。

/// spec:widest path。回 `(瓶頸頻寬, from→to 節點序列)`;不連通 `None`。
/// `edges` 無向 `(a, b, bandwidth)`。O((V+E) log E)。
pub fn widest_path(
    n: usize,
    edges: &[(u32, u32, u64)],
    from: u32,
    to: u32,
) -> Option<(u64, Vec<u32>)> {
    assert!((from as usize) < n && (to as usize) < n);
    let _ = edges;
    todo!("spec: max-heap + min 鬆弛 + 懶刪除 + parent 回溯")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "drill:填完 widest_path 後拔掉"]
    fn wider_but_longer_beats_shorter_narrow() {
        let edges = [(0, 1, 2), (1, 3, 9), (0, 2, 7), (2, 3, 5)];
        let (bw, path) = widest_path(4, &edges, 0, 3).unwrap();
        assert_eq!(bw, 5);
        assert_eq!(path, vec![0, 2, 3]);
    }

    #[test]
    #[ignore = "drill:填完 widest_path 後拔掉"]
    fn single_edge_and_unreachable() {
        assert_eq!(widest_path(3, &[(0, 1, 4)], 0, 1), Some((4, vec![0, 1])));
        assert_eq!(widest_path(3, &[(0, 1, 4)], 0, 2), None);
    }

    #[test]
    #[ignore = "drill:填完 widest_path 後拔掉"]
    fn from_equals_to_is_max() {
        let (bw, path) = widest_path(2, &[(0, 1, 3)], 0, 0).unwrap();
        assert_eq!(bw, u64::MAX);
        assert_eq!(path, vec![0]);
    }
}
