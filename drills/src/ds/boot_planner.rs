// ⚠ 防雷:本檔是 sim o(boot planner,algo 系)的填空版,spec 註解含解法方向。
// 排程:8/3 開機槽(取代原「graph 一題」——它就是圖論開機題的正式版)。

//! drill:boot_planner —— Zero-Touch 開機規劃(sim o 的填空版)。
//!
//! 已給:`BootPlan` / `Cycle` 型別、`extract_cycle`(環回報,已寫好)。
//! 要填:`plan_boot`(Kahn 分層 + 最長路徑 DP)與 `blast_radius`(前向可達)。
//!
//! 核心不變量:
//! - **一趟 Kahn 做三件事**:frontier 整批 = 一波;沿 topo 序鬆弛
//!   `dist[v] = max(dist[v], dist[u] + boot_ms[v])`(DAG 才敢做最長路徑!);
//!   `processed < n` ⇔ 有環 → 呼叫已給的 `extract_cycle`。
//! - 每波輸出前 `sort_unstable`——決定性輸出,測試斷言波內容。
//! - `critical_path` 靠 parent 鏈回溯再 reverse;`makespan = max(dist)`。
//! - `blast_radius`:前向 BFS/DFS,**不含 failed 自身**,升冪回傳。
//!
//! 完整推導與 trade-offs 見 reference 同名模組檔頭。

use std::collections::VecDeque;

/// 開機計畫:波次(每波可同時開)+ makespan + 一條關鍵路徑。
#[derive(Debug)]
pub struct BootPlan {
    /// 第 i 波可同時開機的節點(波內升冪;波 0 = 無依賴者)。
    pub waves: Vec<Vec<u32>>,
    /// 關鍵路徑總時長。
    pub makespan_ms: u64,
    /// 一條實現 makespan 的依賴鏈(依 boot 順序)。
    pub critical_path: Vec<u32>,
}

/// 依賴環(依 a→b 依賴方向排列;起點任意)。
#[derive(Debug, PartialEq, Eq)]
pub struct Cycle(pub Vec<u32>);

/// spec:規劃開機波次。`(a, b)` = a 先完成 b 才能開始。
/// Kahn 分層(frontier 批次 = 波)+ 同趟最長路徑 DP(dist/parent)+
/// `processed < n` 時回 `Err(extract_cycle(..))`。O(V + E + V log V)。
pub fn plan_boot(n: usize, deps: &[(u32, u32)], boot_ms: &[u64]) -> Result<BootPlan, Cycle> {
    assert_eq!(boot_ms.len(), n, "boot_ms 與節點數不符");
    if n == 0 {
        return Ok(BootPlan {
            waves: Vec::new(),
            makespan_ms: 0,
            critical_path: Vec::new(),
        });
    }
    // todo!("spec: Kahn 分層 + 沿 topo 序鬆弛最長路徑 + 環偵測")
    // (a, b): a -> b
    let mut ind = vec![0; n];
    // (nxt_node, boot_time)
    let mut adj_list: Vec<Vec<(usize, u64)>> = vec![vec![]; n];
    for &(a, b) in deps {
        adj_list[a as usize].push((b as usize, boot_ms[b as usize]));
        ind[b as usize] += 1;
    }
    let mut queue: VecDeque<usize> = VecDeque::new();
    let mut processed = 0;
    let mut dis = vec![0u64; n];
    let mut parent = vec![0usize; n];
    for (nodeid, _ind) in ind.iter().enumerate().filter(|(node_id, ind)| **ind == 0) {
        queue.push_back(nodeid);
        dis[nodeid] = boot_ms[nodeid];
        parent[nodeid] = nodeid;
    }

    let mut waves: Vec<Vec<u32>> = Vec::new();
    let mut max_node_time = (0, 0);
    while !queue.is_empty() {
        let sz = queue.len();
        let mut wave = Vec::new();
        for _ in 0..sz {
            let nodenow = queue.pop_front().unwrap();
            let time = dis[nodenow];
            if time > max_node_time.1 {
                max_node_time = (nodenow, time);
            }
            wave.push(nodenow as u32);
            processed += 1;
            for &(nxt_node, delta_time) in &adj_list[nodenow] {
                let nxt_time = time + delta_time;
                ind[nxt_node] -= 1;
                if dis[nxt_node] < nxt_time {
                    dis[nxt_node] = nxt_time;
                    parent[nxt_node] = nodenow;
                }
                if ind[nxt_node] > 0 {
                    continue;
                }
                queue.push_back(nxt_node);
            }
        }
        wave.sort_unstable();
        waves.push(wave);
    }

    if processed != n {
        return Err(extract_cycle(n, deps, &ind));
    }
    let mut critical_path = Vec::new();
    let mut cur = max_node_time.0;
    critical_path.push(cur as u32);
    while cur != parent[cur] {
        cur = parent[cur];
        critical_path.push(cur as u32);
    }
    critical_path.reverse();
    Ok(BootPlan {
        waves,
        makespan_ms: max_node_time.1,
        critical_path,
    })
}

/// 已給:Kahn 收不完 ⇒ 殘餘節點(indeg > 0)中必有環——每個殘餘節點都
/// 至少有一個殘餘前驅,沿前驅走必重訪。呼叫時把 Kahn 結束後的 indeg 傳進來。
#[allow(dead_code)]
fn extract_cycle(n: usize, deps: &[(u32, u32)], indeg: &[u32]) -> Cycle {
    let remaining: Vec<bool> = indeg.iter().map(|&d| d > 0).collect();
    let mut pred = vec![usize::MAX; n];
    for &(a, b) in deps {
        if remaining[a as usize] && remaining[b as usize] {
            pred[b as usize] = a as usize;
        }
    }
    let start = (0..n)
        .find(|&v| remaining[v])
        .expect("processed < n 保證有殘餘");
    let mut seen = vec![false; n];
    let mut cur = start;
    while !seen[cur] {
        seen[cur] = true;
        cur = pred[cur];
    }
    let mut cycle = vec![cur as u32];
    let mut x = pred[cur];
    while x != cur {
        cycle.push(x as u32);
        x = pred[x];
    }
    cycle.reverse();
    Cycle(cycle)
}

/// spec:故障影響面——`failed` 掛掉後再也開不了機的下游集合
/// (前向可達;**不含 failed 自身**;升冪)。O(V + E)。
pub fn blast_radius(n: usize, deps: &[(u32, u32)], failed: u32) -> Vec<u32> {
    assert!((failed as usize) < n);
    // todo!("spec: 前向 BFS/DFS 可達集合,排除自身,sort")
    let mut adj_list = vec![vec![]; n];
    for &(u, v) in deps {
        adj_list[u as usize].push(v as usize);
    }
    let mut ans = Vec::new();
    let mut qe = VecDeque::new();
    let mut visited = vec![false; n];
    qe.push_back(failed as usize);
    visited[failed as usize] = true;
    while let Some(front) = qe.pop_front() {
        for &nxt in &adj_list[front] {
            if visited[nxt] {
                continue;
            }
            ans.push(nxt as u32);
            qe.push_back(nxt);
            visited[nxt] = true;
        }
    }
    ans.sort_unstable();
    ans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diamond_waves_and_critical_path() {
        let deps = [(0, 1), (0, 2), (1, 3), (2, 3)];
        let plan = plan_boot(4, &deps, &[5, 10, 20, 1]).unwrap();
        assert_eq!(plan.waves, vec![vec![0], vec![1, 2], vec![3]]);
        assert_eq!(plan.makespan_ms, 26);
        assert_eq!(plan.critical_path, vec![0, 2, 3]);
    }

    #[test]
    fn chain_makespan_is_sum() {
        let plan = plan_boot(3, &[(0, 1), (1, 2)], &[1, 2, 3]).unwrap();
        assert_eq!(plan.waves, vec![vec![0], vec![1], vec![2]]);
        assert_eq!(plan.makespan_ms, 6);
        assert_eq!(plan.critical_path, vec![0, 1, 2]);
    }

    #[test]
    fn no_deps_single_wave() {
        let plan = plan_boot(3, &[], &[4, 9, 2]).unwrap();
        assert_eq!(plan.waves, vec![vec![0, 1, 2]]);
        assert_eq!(plan.makespan_ms, 9);
        assert_eq!(plan.critical_path, vec![1]);
    }

    #[test]
    fn cycle_is_reported_with_members() {
        let err = plan_boot(3, &[(0, 1), (1, 2), (2, 1)], &[1, 1, 1]).unwrap_err();
        let mut members = err.0.clone();
        members.sort_unstable();
        assert_eq!(members, vec![1, 2]);
        assert_eq!(err.0.len(), 2);
    }

    #[test]
    fn blast_radius_excludes_failed_and_finds_descendants() {
        let deps = [(0, 1), (0, 2), (1, 3), (2, 3)];
        assert_eq!(blast_radius(4, &deps, 0), vec![1, 2, 3]);
        assert_eq!(blast_radius(4, &deps, 1), vec![3]);
        assert_eq!(blast_radius(4, &deps, 3), Vec::<u32>::new());
    }

    #[test]
    fn empty_plan_is_empty() {
        let plan = plan_boot(0, &[], &[]).unwrap();
        assert!(plan.waves.is_empty());
        assert_eq!(plan.makespan_ms, 0);
        assert!(plan.critical_path.is_empty());
    }

    #[test]
    fn source_not_in_prefix() {
        let plan = plan_boot(2, &[(1, 0)], &[3, 5]).unwrap();
        assert_eq!(plan.waves, vec![[1], [0]]);
        assert_eq!(plan.makespan_ms, 8);
        assert_eq!(plan.critical_path, vec![1, 0]);
    }

    #[test]
    fn fake_circle_with_same_weight() {
        let plan = plan_boot(4, &[(0, 2), (0, 1), (2, 3), (1, 3)], &[5, 10, 20, 1]).unwrap();
        assert_eq!(plan.waves, vec![vec![0], vec![1, 2], vec![3]]);
        assert_eq!(plan.makespan_ms, 26);
        assert_eq!(plan.critical_path, vec![0, 2, 3]);
    }
}
