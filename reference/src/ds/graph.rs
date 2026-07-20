//! # graph —— adjacency list + BFS / DFS / Kahn topo / Dijkstra
//!
//! ## [Clarify]
//! 解決:面試圖論四件套的 std-only 標準寫法。
//! Constraints:節點是 `0..n`(dense id;外部實體先映射,同 [`crate::ds::dsu`]);
//! 邊權 `u64` 非負(Dijkstra 前提!有負邊要 Bellman-Ford,聲明不做);
//! 稀疏圖(E ≪ V²)——所以 adjacency list 不是 matrix。
//!
//! ## [Abstract]
//! 節點 payload、邊的屬性(名字、容量)都不進 Graph——caller 用平行陣列存。
//! 面試時聲明:「圖只管拓撲,屬性外掛」往前走。
//!
//! ## [Trade-offs]
//! - adjacency list `Vec<Vec<(usize, u64)>>`:空間 O(V+E);
//!   遍歷鄰居 O(deg);查「u→v 有邊嗎」O(deg)(matrix O(1) 但空間 O(V²))。
//! - BFS 用 `VecDeque`,dist 同時兼 visited(`None` = 未訪)——少一個陣列。
//! - DFS 給迭代版(顯式 stack):遞迴深度 = 最長路徑,1e6 節點鏈會爆 stack。
//! - Kahn's topo 而非 DFS 後序:順便偵測 cycle(輸出長度 < n ⇔ 有環),
//!   且不需要三色標記。
//! - Dijkstra:std 的 `BinaryHeap` 是 max-heap,用 `Reverse` 包成 min-heap;
//!   std 沒有 decrease-key ⇒ 用「懶刪除」:同節點可重複入堆,
//!   彈出時比 dist 舊就跳過。堆內最多 O(E) 條目 ⇒ O((V+E) log E)。
//!
//! ## [Dry-Run]
//! 每個演算法一個逐行 trace 測試;boundary:空圖、單節點、不連通、
//! 自環、有環 topo → None、Dijkstra stale entry 跳過。

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

pub struct Graph {
    adj: Vec<Vec<(usize, u64)>>, // adj[u] = [(v, w), ...]
    directed: bool,
}

impl Graph {
    pub fn new(n: usize, directed: bool) -> Self {
        Self {
            adj: vec![Vec::new(); n],
            directed,
        }
    }

    pub fn len(&self) -> usize {
        self.adj.len()
    }

    pub fn is_empty(&self) -> bool {
        self.adj.is_empty()
    }

    /// O(1) amortized。無向圖存兩份(空間換遍歷時不用判方向)。
    pub fn add_edge(&mut self, u: usize, v: usize, w: u64) {
        self.adj[u].push((v, w));
        if !self.directed && u != v {
            self.adj[v].push((u, w));
        }
    }

    /// 無權最短路(邊數)。O(V+E) 時間、O(V) 空間。
    /// 回傳 dist[v] = Some(hops) 或 None(不可達)——dist 兼 visited。
    pub fn bfs_dist(&self, src: usize) -> Vec<Option<u32>> {
        let mut dist = vec![None; self.adj.len()];
        let mut queue = VecDeque::new();
        dist[src] = Some(0);
        queue.push_back(src);
        while let Some(u) = queue.pop_front() {
            let du = dist[u].unwrap(); // 入過佇列必有值
            for &(v, _) in &self.adj[u] {
                if dist[v].is_none() {
                    // 標記在「入隊時」而非「出隊時」:同一節點才不會入隊多次
                    dist[v] = Some(du + 1);
                    queue.push_back(v);
                }
            }
        }
        dist
    }

    /// 迭代 DFS 前序。O(V+E) 時間、O(V) 空間(stack + visited)。
    ///
    /// 注意 stack 版的訪問順序與遞迴版鏡像(後 push 先 pop);
    /// 為了穩定輸出,鄰居**反向** push,使前序與遞迴版一致。
    pub fn dfs_preorder(&self, src: usize) -> Vec<usize> {
        let mut visited = vec![false; self.adj.len()];
        let mut stack = vec![src];
        let mut order = Vec::new();
        while let Some(u) = stack.pop() {
            if visited[u] {
                continue; // 可能重複入 stack(晚到的路徑),彈出時去重
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

    /// Kahn's topological sort(僅有向圖)。O(V+E)。
    /// 回傳 None ⇔ 有環(環上節點 in-degree 永不歸零,輸出湊不滿 n)。
    pub fn topo_sort(&self) -> Option<Vec<usize>> {
        assert!(
            self.directed,
            "topo sort on undirected graph is meaningless"
        );
        let n = self.adj.len();
        let mut indeg = vec![0usize; n];
        for u in 0..n {
            for &(v, _) in &self.adj[u] {
                indeg[v] += 1;
            }
        }
        // 種子:所有 in-degree 0 的節點(沒有先決條件的任務)
        let mut queue: VecDeque<usize> = (0..n).filter(|&v| indeg[v] == 0).collect();
        let mut order = Vec::with_capacity(n);
        while let Some(u) = queue.pop_front() {
            order.push(u);
            for &(v, _) in &self.adj[u] {
                indeg[v] -= 1;
                if indeg[v] == 0 {
                    queue.push_back(v);
                }
            }
        }
        (order.len() == n).then_some(order)
    }

    /// 單源最短路(非負權)。O((V+E) log E) 時間、O(V+E) 空間(堆懶刪除)。
    pub fn dijkstra(&self, src: usize) -> Vec<Option<u64>> {
        let mut dist: Vec<Option<u64>> = vec![None; self.adj.len()];
        // min-heap:BinaryHeap 是 max-heap,Reverse 反轉比較。
        // 元組 (d, v):按 d 排序,d 相同再比 v(無所謂,只要全序)。
        let mut heap = BinaryHeap::new();
        dist[src] = Some(0);
        heap.push(Reverse((0u64, src)));
        while let Some(Reverse((d, u))) = heap.pop() {
            // 懶刪除:std 堆沒有 decrease-key,同節點可能有多筆過期條目;
            // 彈出的 d 比已定案的 dist[u] 大 ⇒ stale,跳過。
            if dist[u].is_some_and(|best| d > best) {
                continue;
            }
            for &(v, w) in &self.adj[u] {
                let nd = d + w;
                if dist[v].is_none_or(|cur| nd < cur) {
                    dist[v] = Some(nd);
                    heap.push(Reverse((nd, v)));
                }
            }
        }
        dist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [Dry-Run] BFS trace(無向):
    ///   0-1, 0-2, 1-3;src=0
    ///   init: dist[0]=0, q=[0]
    ///   pop 0: 鄰 1,2 未訪 → dist[1]=1, dist[2]=1, q=[1,2]
    ///   pop 1: 鄰 0(訪過), 3 → dist[3]=2, q=[2,3]
    ///   pop 2: 鄰 0 訪過        pop 3: 鄰 1 訪過 → q 空,結束
    /// boundary:節點 4 不連通 → None。
    #[test]
    fn bfs_dist_trace_with_unreachable() {
        let mut g = Graph::new(5, false);
        g.add_edge(0, 1, 1);
        g.add_edge(0, 2, 1);
        g.add_edge(1, 3, 1);
        assert_eq!(
            g.bfs_dist(0),
            vec![Some(0), Some(1), Some(1), Some(2), None]
        );
    }

    /// DFS 前序 = 遞迴版順序(鄰居反向 push 保序)。
    /// trace:0→(1,2);1→(3);preorder = 0,1,3,2。
    #[test]
    fn dfs_preorder_matches_recursive_order() {
        let mut g = Graph::new(4, true);
        g.add_edge(0, 1, 1);
        g.add_edge(0, 2, 1);
        g.add_edge(1, 3, 1);
        assert_eq!(g.dfs_preorder(0), vec![0, 1, 3, 2]);
    }

    /// Kahn trace(DAG):0→1→3, 0→2→3
    ///   indeg = [0,1,1,2];seed q=[0]
    ///   pop 0 → indeg[1]=0 入隊、indeg[2]=0 入隊;pop 1 → indeg[3]=1
    ///   pop 2 → indeg[3]=0 入隊;pop 3。order=[0,1,2,3],len=4=n ✓
    #[test]
    fn topo_sort_dag_diamond() {
        let mut g = Graph::new(4, true);
        g.add_edge(0, 1, 1);
        g.add_edge(0, 2, 1);
        g.add_edge(1, 3, 1);
        g.add_edge(2, 3, 1);
        let order = g.topo_sort().unwrap();
        // 驗證拓撲性質而非固定序列:每條邊 u 都在 v 前
        let pos: Vec<usize> = {
            let mut p = vec![0; 4];
            for (i, &v) in order.iter().enumerate() {
                p[v] = i;
            }
            p
        };
        assert!(pos[0] < pos[1] && pos[0] < pos[2] && pos[1] < pos[3] && pos[2] < pos[3]);
    }

    /// boundary:有環 → None(1→2→1;兩節點 in-degree 永不歸零)。
    #[test]
    fn boundary_topo_cycle_returns_none() {
        let mut g = Graph::new(3, true);
        g.add_edge(0, 1, 1);
        g.add_edge(1, 2, 1);
        g.add_edge(2, 1, 1); // 環
        assert_eq!(g.topo_sort(), None);
    }

    /// Dijkstra trace(有向):
    ///   0→1 w4;0→2 w1;2→1 w1;1→3 w1
    ///   pop (0,0):dist[1]=4 入堆、dist[2]=1 入堆
    ///   pop (1,2):經 2 到 1 是 1+1=2 < 4 → dist[1]=2 再入堆(4 那筆變 stale)
    ///   pop (2,1):dist[3]=3 入堆
    ///   pop (4,1):**stale(4 > dist[1]=2)→ 跳過** ← 懶刪除的關鍵路徑
    ///   pop (3,3):無鄰。結果 [0,2,1,3]
    #[test]
    fn dijkstra_relaxation_with_stale_heap_entry() {
        let mut g = Graph::new(4, true);
        g.add_edge(0, 1, 4);
        g.add_edge(0, 2, 1);
        g.add_edge(2, 1, 1);
        g.add_edge(1, 3, 1);
        assert_eq!(g.dijkstra(0), vec![Some(0), Some(2), Some(1), Some(3)]);
    }

    /// boundary:不可達節點 None、零權邊、自環(不得影響結果)。
    #[test]
    fn boundary_dijkstra_unreachable_zero_weight_self_loop() {
        let mut g = Graph::new(4, true);
        g.add_edge(0, 1, 0); // 零權
        g.add_edge(1, 1, 5); // 自環:1+5 > 1 的 dist,永不更新
        assert_eq!(g.dijkstra(0), vec![Some(0), Some(0), None, None]);
    }

    /// boundary:單節點與空圖。
    #[test]
    fn boundary_single_node_and_empty() {
        let g = Graph::new(1, true);
        assert_eq!(g.bfs_dist(0), vec![Some(0)]);
        assert_eq!(g.dfs_preorder(0), vec![0]);
        assert_eq!(g.topo_sort(), Some(vec![0]));
        let empty = Graph::new(0, true);
        assert_eq!(empty.topo_sort(), Some(vec![]));
    }
}
