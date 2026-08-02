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
    todo!("spec: Kahn 分層 + 沿 topo 序鬆弛最長路徑 + 環偵測")
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
    todo!("spec: 前向 BFS/DFS 可達集合,排除自身,sort")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "drill:填完 plan_boot 後拔掉"]
    fn diamond_waves_and_critical_path() {
        let deps = [(0, 1), (0, 2), (1, 3), (2, 3)];
        let plan = plan_boot(4, &deps, &[5, 10, 20, 1]).unwrap();
        assert_eq!(plan.waves, vec![vec![0], vec![1, 2], vec![3]]);
        assert_eq!(plan.makespan_ms, 26);
        assert_eq!(plan.critical_path, vec![0, 2, 3]);
    }

    #[test]
    #[ignore = "drill:填完 plan_boot 後拔掉"]
    fn chain_makespan_is_sum() {
        let plan = plan_boot(3, &[(0, 1), (1, 2)], &[1, 2, 3]).unwrap();
        assert_eq!(plan.waves, vec![vec![0], vec![1], vec![2]]);
        assert_eq!(plan.makespan_ms, 6);
        assert_eq!(plan.critical_path, vec![0, 1, 2]);
    }

    #[test]
    #[ignore = "drill:填完 plan_boot 後拔掉"]
    fn no_deps_single_wave() {
        let plan = plan_boot(3, &[], &[4, 9, 2]).unwrap();
        assert_eq!(plan.waves, vec![vec![0, 1, 2]]);
        assert_eq!(plan.makespan_ms, 9);
        assert_eq!(plan.critical_path, vec![1]);
    }

    #[test]
    #[ignore = "drill:填完 plan_boot 後拔掉"]
    fn cycle_is_reported_with_members() {
        let err = plan_boot(3, &[(0, 1), (1, 2), (2, 1)], &[1, 1, 1]).unwrap_err();
        let mut members = err.0.clone();
        members.sort_unstable();
        assert_eq!(members, vec![1, 2]);
        assert_eq!(err.0.len(), 2);
    }

    #[test]
    #[ignore = "drill:填完 blast_radius 後拔掉"]
    fn blast_radius_excludes_failed_and_finds_descendants() {
        let deps = [(0, 1), (0, 2), (1, 3), (2, 3)];
        assert_eq!(blast_radius(4, &deps, 0), vec![1, 2, 3]);
        assert_eq!(blast_radius(4, &deps, 1), vec![3]);
        assert_eq!(blast_radius(4, &deps, 3), Vec::<u32>::new());
    }

    #[test]
    #[ignore = "drill:填完 plan_boot 後拔掉"]
    fn empty_plan_is_empty() {
        let plan = plan_boot(0, &[], &[]).unwrap();
        assert!(plan.waves.is_empty());
        assert_eq!(plan.makespan_ms, 0);
        assert!(plan.critical_path.is_empty());
    }
}
