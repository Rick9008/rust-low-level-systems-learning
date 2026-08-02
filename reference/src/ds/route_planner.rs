//! # route_planner —— Interconnect 路由:widest path(max-min 頻寬;認題卡 AG-R 的正式版)
//!
//! ## [Clarify]
//! 解決:rack 內加速器經 switch 相連(帶頻寬的無向圖),為一筆大傳輸選路,
//! 路徑的可用頻寬 = 沿途**最小**的那條 link——求瓶頸頻寬最大的路。
//! Constraints:節點 `0..n` dense;無向(NVLink/PCIe link 雙向同頻寬,
//! 有向版只差建圖);頻寬 `u64` 正數;找不到路回 `None`。
//! **認題訊號:題面出現 "minimum along the path" → 秒認 widest path,不是最短路。**
//!
//! ## [Abstract]
//! 並發傳輸的容量預留(flow/max-flow 領域)、多路徑分流(ECMP)、延遲混合目標
//! ——全部聲明不做;被追問就一句:「reservation 版把 link 頻寬扣掉重跑同一支」。
//!
//! ## [Iterate]
//! naive:枚舉路徑取 max(min(...)) → 指數。
//! 正解:Dijkstra 骨架**兩處變形**——鬆弛從 `dist[u] + w` 改 `min(bott[u], w)`、
//! 堆從 min-heap(`Reverse`)改 **max-heap(不包 Reverse)**。貪婪論證不變:
//! 瓶頸值沿路徑單調不增(min 只會變小),所以「目前最寬的未定案節點」可以定案
//! ——與 Dijkstra 的「目前最近」同構。O((V+E) log E),懶刪除同 `graph` 模組。
//!
//! ## [Trade-offs]
//! - **max-heap 直接放 `(bott, node)`**:std `BinaryHeap` 本來就是 max-heap,
//!   這題反而不用 `Reverse`——跟最短路互為鏡像,面試講出這個對比是加分句。
//! - 無 decrease-key ⇒ 懶刪除:pop 出來比 `bott[]` 舊就跳過(同 Dijkstra)。
//! - 平手(同瓶頸)不特別破:要「同寬取 hop 少」就把鍵換 `(bott, Reverse(hops))`,
//!   多 3 行,聲明即可。
//!
//! ## [Dry-Run]
//! 菱形反例逐行 trace 見 [`tests::wider_but_longer_beats_shorter_narrow`]:
//! 短路窄(bw 2)、長路寬(bw 7)→ 正解走長路。boundary:不連通 → None、
//! 起點=終點(瓶頸=`u64::MAX`、路徑=[from])、單邊。

use std::collections::BinaryHeap;

/// widest path:回傳 `(瓶頸頻寬, from→to 的節點序列)`;不連通回 `None`。
///
/// `edges` 為無向邊 `(a, b, bandwidth)`。`from == to` 時瓶頸定義為
/// `u64::MAX`(不經過任何邊)。O((V+E) log E)。
pub fn widest_path(
    n: usize,
    edges: &[(u32, u32, u64)],
    from: u32,
    to: u32,
) -> Option<(u64, Vec<u32>)> {
    assert!((from as usize) < n && (to as usize) < n);
    let mut adj = vec![Vec::new(); n];
    for &(a, b, w) in edges {
        adj[a as usize].push((b as usize, w));
        adj[b as usize].push((a as usize, w));
    }

    // bott[v] = 目前已知「from → v」的最大瓶頸;0 = 還沒路。
    let mut bott = vec![0u64; n];
    let mut parent = vec![usize::MAX; n];
    bott[from as usize] = u64::MAX;

    // max-heap:std BinaryHeap 原生就是,不包 Reverse(和最短路互為鏡像)。
    let mut heap = BinaryHeap::new();
    heap.push((u64::MAX, from as usize));

    while let Some((b, u)) = heap.pop() {
        if b < bott[u] {
            continue; // 懶刪除:過期條目
        }
        if u == to as usize {
            break; // 目標已定案(貪婪論證:pop 到的瓶頸單調不增)
        }
        for &(v, w) in &adj[u] {
            let cand = b.min(w); // 兩處變形之一:min 取代 +
            if cand > bott[v] {
                bott[v] = cand;
                parent[v] = u;
                heap.push((cand, v));
            }
        }
    }

    if bott[to as usize] == 0 && from != to {
        return None;
    }
    let mut path = Vec::new();
    let mut cur = to as usize;
    loop {
        path.push(cur as u32);
        if cur == from as usize {
            break;
        }
        cur = parent[cur];
    }
    path.reverse();
    Some((bott[to as usize], path))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 菱形反例逐行 trace:0—1(bw 2)—3 是短窄路;0—2(bw 7)—3(bw 5)是長寬路。
    /// - 初始:bott=[MAX,0,0,0],heap=[(MAX,0)]。
    /// - pop 0:鬆弛 0→1 cand=min(MAX,2)=2 → bott[1]=2;0→2 cand=7 → bott[2]=7。
    /// - pop (7,2)(max-heap 先出大的):鬆弛 2→3 cand=min(7,5)=5 → bott[3]=5,p=2。
    /// - pop (5,3):u==to,break——注意 (2,1) 還躺在堆裡,貪婪保證不用看它。
    /// - 答案 (5, [0,2,3]):寬的長路贏,最短路答案 [0,1,3](瓶頸 2)是錯的。
    #[test]
    fn wider_but_longer_beats_shorter_narrow() {
        let edges = [(0, 1, 2), (1, 3, 9), (0, 2, 7), (2, 3, 5)];
        let (bw, path) = widest_path(4, &edges, 0, 3).unwrap();
        assert_eq!(bw, 5);
        assert_eq!(path, vec![0, 2, 3]);
    }

    #[test]
    fn single_edge_and_unreachable() {
        assert_eq!(widest_path(3, &[(0, 1, 4)], 0, 1), Some((4, vec![0, 1])));
        assert_eq!(widest_path(3, &[(0, 1, 4)], 0, 2), None);
    }

    #[test]
    fn from_equals_to_is_max() {
        let (bw, path) = widest_path(2, &[(0, 1, 3)], 0, 0).unwrap();
        assert_eq!(bw, u64::MAX);
        assert_eq!(path, vec![0]);
    }

    #[test]
    fn undirected_works_both_ways() {
        let edges = [(0, 1, 2), (1, 3, 9), (0, 2, 7), (2, 3, 5)];
        let (bw, path) = widest_path(4, &edges, 3, 0).unwrap();
        assert_eq!(bw, 5);
        assert_eq!(path, vec![3, 2, 0]);
    }
}
