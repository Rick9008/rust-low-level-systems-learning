# graph 設計取捨

對應程式碼:`reference/src/ds/graph.rs`。

## 表示法:adjacency list vs matrix

稀疏圖(E ≪ V²,現實網路幾乎都是)用 list:空間 O(V+E)、遍歷鄰居 O(deg)。
matrix 只在 dense 或需要 O(1) 邊查詢時划算,空間 O(V²)。
節點用 dense id `0..n`:外部實體(字串、IP)先過一層 HashMap 映射——
跟 [dsu](dsu.md) 同一招,把「識別」與「演算法」分離。

## 四個演算法的面試要點

**BFS**:dist 陣列兼 visited(`None` = 未訪)。標記時機在**入隊時**——
出隊才標記的版本同一節點會入隊多次,最壞 O(V²) 佇列膨脹。

**DFS 迭代版**:顯式 stack,深度不再受 call stack 限制(遞迴版在 10⁶ 節點
鏈上爆 stack;Rust 預設 main thread 8MB、spawn 的 thread 2MB)。
兩個細節:(1) 彈出時去重(同節點可能被多條路徑壓入);
(2) 鄰居反向 push 才能得到與遞迴版相同的前序。

**Kahn's topo**:in-degree 歸零才入隊 ⇒ 輸出天然是拓撲序;
輸出長度 < n ⇔ 有環(環上節點 in-degree 永不歸零)——**cycle 偵測免費附贈**。
DFS 後序反轉也能 topo,但 cycle 偵測要三色標記,面試容易寫錯。

**Dijkstra**:兩個 std 特性決定寫法——
1. `BinaryHeap` 是 max-heap → `Reverse((dist, node))` 反轉成 min-heap。
   tuple 排序先比 dist(這就是把 dist 放前面的原因)。
2. 沒有 decrease-key → **懶刪除**:鬆弛成功就再 push 一筆,彈出時
   `d > dist[u]` 即 stale 跳過。堆最多 O(E) 條目,O((V+E) log E);
   比手寫可索引堆(decrease-key 版 O((V+E) log V))簡單得多,常數差異面試可忽略。
   **前提:邊權非負**;有負邊換 Bellman-Ford。

## Production 對照

petgraph(泛型圖 + 演算法庫)。實務大圖常用 CSR(壓縮稀疏行)取代
`Vec<Vec<_>>` 消除二層指標跳躍——原理同 arena:把散落 heap 的東西攤平成連續陣列。

## 互動教材

[artifacts/graph.html](artifacts/graph.html) —— 一張 8 節點有向加權圖、四個演算法、一個 stepper。
重點是**永遠把前緣容器擺在圖旁邊**:BFS 的 queue、DFS 的 stack、Kahn 的 in-degree 陣列、
Dijkstra 的 `BinaryHeap<Reverse<_>>`。可以按鈕加一條 back-edge 製造環,
看 Kahn 的輸出從 8 個掉到 3 個(這就是 cycle detection),而 BFS/DFS/Dijkstra 的答案完全不受影響。
Dijkstra 跑到底會看到三次 stale pop 被跳過 —— 懶刪除的關鍵路徑。
