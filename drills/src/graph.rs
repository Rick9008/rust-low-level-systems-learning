//! drill:graph —— 填 BFS / Kahn topo / Dijkstra。
//!
//! 已給:結構、add_edge、`dfs_preorder`(當作已完成的範例讀)。
//! 要填:`bfs_dist` / `topo_sort` / `dijkstra`。
//! 三個經典陷阱:BFS 在**入隊時**標記;Kahn 靠「輸出長度 < n ⇔ 有環」;
//! Dijkstra 用 Reverse 做 min-heap + 懶刪除(彈出比 dist 舊就 continue)。

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

pub struct Graph {
    adj: Vec<Vec<(usize, u64)>>,
    directed: bool,
}

impl Graph {
    pub fn new(n: usize, directed: bool) -> Self {
        Self {
            adj: vec![Vec::new(); n],
            directed,
        }
    }

    pub fn add_edge(&mut self, u: usize, v: usize, w: u64) {
        self.adj[u].push((v, w));
        if !self.directed && u != v {
            self.adj[v].push((u, w));
        }
    }

    /// 範例(已完成):迭代 DFS 前序。注意鄰居反向 push 保序、彈出時去重。
    pub fn dfs_preorder(&self, src: usize) -> Vec<usize> {
        let mut visited = vec![false; self.adj.len()];
        let mut stack = vec![src];
        let mut order = Vec::new();
        while let Some(u) = stack.pop() {
            if visited[u] {
                continue;
            }
            visited[u] = true;
            order.push(u);
            for &(v, _) in self.adj[u].iter().rev() {
                if !visited[v] {
                    stack.push(v);
                }
            }
        }
        order
    }

    /// spec:無權最短路(hop 數)。dist[v]=None 表示不可達,dist 兼 visited。
    /// **標記時機在入隊時**——出隊才標記會讓同一節點入隊多次。O(V+E)。
    pub fn bfs_dist(&self, src: usize) -> Vec<Option<u32>> {
        todo!("spec: VecDeque; dist[src]=Some(0); 鄰居未訪(None)才標記+入隊")
    }

    /// spec:Kahn's topo(僅有向圖)。
    /// 1. 算全圖 in-degree;2. in-degree 0 者入隊;
    /// 3. 出隊入結果,鄰居 in-degree -1,歸零者入隊;
    /// 4. 結果長度 == n → Some(order);< n → None(有環)。
    pub fn topo_sort(&self) -> Option<Vec<usize>> {
        assert!(self.directed);
        todo!("spec: in-degree 表 + 零度佇列;輸出湊不滿 n 即有環")
    }

    /// spec:單源最短路(非負權)。BinaryHeap 是 max-heap ⇒
    /// push `Reverse((dist, node))`。std 沒有 decrease-key ⇒ 懶刪除:
    /// 鬆弛成功就再 push 一筆;彈出時 `d > dist[u]` 即 stale,continue。
    /// O((V+E) log E)。
    pub fn dijkstra(&self, src: usize) -> Vec<Option<u64>> {
        todo!("spec: Reverse min-heap + 懶刪除 + 鬆弛 nd < dist[v] 才更新")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// boundary:BFS 距離 + 不可達 None。
    #[test]
    #[ignore = "填完 bfs_dist 後移除"]
    fn bfs_with_unreachable() {
        let mut g = Graph::new(5, false);
        g.add_edge(0, 1, 1);
        g.add_edge(0, 2, 1);
        g.add_edge(1, 3, 1);
        assert_eq!(
            g.bfs_dist(0),
            vec![Some(0), Some(1), Some(1), Some(2), None]
        );
    }

    /// boundary:DAG 出拓撲序;有環 → None。
    #[test]
    #[ignore = "填完 topo_sort 後移除"]
    fn topo_dag_and_cycle() {
        let mut dag = Graph::new(4, true);
        dag.add_edge(0, 1, 1);
        dag.add_edge(0, 2, 1);
        dag.add_edge(1, 3, 1);
        dag.add_edge(2, 3, 1);
        let order = dag.topo_sort().unwrap();
        let pos = |v: usize| order.iter().position(|&x| x == v).unwrap();
        assert!(pos(0) < pos(1) && pos(0) < pos(2) && pos(1) < pos(3) && pos(2) < pos(3));

        let mut cyc = Graph::new(2, true);
        cyc.add_edge(0, 1, 1);
        cyc.add_edge(1, 0, 1);
        assert_eq!(cyc.topo_sort(), None);
    }

    /// boundary:Dijkstra 鬆弛(繞路更短)+ stale heap entry + 不可達。
    #[test]
    #[ignore = "填完 dijkstra 後移除"]
    fn dijkstra_relaxation_and_stale() {
        let mut g = Graph::new(5, true);
        g.add_edge(0, 1, 4);
        g.add_edge(0, 2, 1);
        g.add_edge(2, 1, 1); // 繞 2 到 1 只要 2 < 4:直邊那筆變 stale
        g.add_edge(1, 3, 1);
        assert_eq!(
            g.dijkstra(0),
            vec![Some(0), Some(2), Some(1), Some(3), None]
        );
    }
}
