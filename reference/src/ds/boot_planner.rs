//! # boot_planner —— Zero-Touch 開機規劃:Kahn 分層 + DAG 最長路徑 + 環回報(sim o)
//!
//! ## [Clarify]
//! 解決:N 台節點帶依賴關係(a 必須先 boot 完 b 才能開始),求「哪些節點可以
//! 同時開機」的波次計畫、整機 makespan、以及故障時的下游影響面。
//! Constraints:節點 id 是 `0..n`(dense;外部主機名先映射);依賴圖必須是
//! DAG——**有環要把環抓出來回報**(運維要看的是「哪幾台互相等」,不是一個
//! bool);`boot_ms[v]` 是 v 自己的開機時長,非負。
//! 這題是 sim n(scheduler)的演算法向姊妹題:n 考「執行時的兩道閘」,
//! o 考「執行前的靜態分析」——同一張 DAG,兩個時態。
//!
//! ## [Abstract]
//! 電力域上限(同時最多 K 台在 boot)、每台 boot 失敗率、網路開機風暴——
//! 全部聲明不做:K 上限讓波次變成 list scheduling(NP-hard 家族),面試裡
//! 用「波次內再切 K 批,makespan 變上界」一句話帶過即可。
//!
//! ## [Iterate]
//! naive:對每台節點 DFS 找最深依賴鏈 → O(V·(V+E))。
//! 正解:一次 Kahn 拓撲掃描,順路做三件事——分層(frontier 整批 = 一波)、
//! 最長路徑 DP(沿 topo 序鬆弛,無環所以一遍就對)、環偵測(processed < n)。
//! 全部 O(V + E)。
//!
//! ## [Trade-offs]
//! - **波次 = Kahn 的 frontier 批次**:天然回答「哪些可以同時開」;代價是
//!   波內不排序就輸出不穩定 → 每波 `sort_unstable`(決定性輸出,測試可斷言)。
//! - **最長路徑跟拓撲同一趟做**:DAG 上最長路徑是 P(沿 topo 序 DP);一般圖
//!   是 NP-hard——「因為無環所以才敢做」這句要講出來。
//! - **環回報用 pred-walk**:Kahn 結束後殘餘節點必有「殘餘前驅」,沿前驅走
//!   必落入環(有限狀態必重訪)。比 DFS 三色標記少一套狀態機。
//! - `critical_path` 回傳一條實現 makespan 的路徑(可能多條,回傳 parent 鏈
//!   那條)——運維拿它去催最慢的鏈。
//!
//! ## [Dry-Run]
//! 菱形圖逐行 trace 見 [`tests::diamond_waves_and_critical_path`];boundary:
//! 空圖、無依賴(單波)、鏈、環(回報環員)、故障影響面(不含故障者本身)。

/// 開機計畫:波次(每波可同時開)+ makespan + 一條關鍵路徑。
#[derive(Debug)]
pub struct BootPlan {
    /// 第 i 波可同時開機的節點(波內升冪;波 0 = 無依賴者)。
    pub waves: Vec<Vec<u32>>,
    /// 關鍵路徑總時長:`max_v (沿依賴鏈到 v 的 boot_ms 累計)`。
    pub makespan_ms: u64,
    /// 一條實現 makespan 的依賴鏈(依 boot 順序)。
    pub critical_path: Vec<u32>,
}

/// 依賴環(依 a→b 依賴方向排列;起點任意)。
#[derive(Debug, PartialEq, Eq)]
pub struct Cycle(pub Vec<u32>);

/// 規劃開機波次。`deps` 的 `(a, b)` = 「a 必須先完成,b 才能開始」。
///
/// 回傳 `Err(Cycle)` 當依賴成環。O(V + E + V log V)(每波排序)。
pub fn plan_boot(n: usize, deps: &[(u32, u32)], boot_ms: &[u64]) -> Result<BootPlan, Cycle> {
    assert_eq!(boot_ms.len(), n, "boot_ms 與節點數不符");
    if n == 0 {
        return Ok(BootPlan {
            waves: Vec::new(),
            makespan_ms: 0,
            critical_path: Vec::new(),
        });
    }
    let mut adj = vec![Vec::new(); n];
    let mut indeg = vec![0u32; n];
    for &(a, b) in deps {
        adj[a as usize].push(b as usize);
        indeg[b as usize] += 1;
    }

    // dist[v] = 以 v 收尾的最長依賴鏈總時長(含 v 自己);沿 topo 序鬆弛。
    let mut dist: Vec<u64> = boot_ms.to_vec();
    let mut parent = vec![usize::MAX; n];

    let mut frontier: Vec<usize> = (0..n).filter(|&v| indeg[v] == 0).collect();
    frontier.sort_unstable();
    let mut waves: Vec<Vec<u32>> = Vec::new();
    let mut processed = 0usize;

    while !frontier.is_empty() {
        let mut next = Vec::new();
        for &u in &frontier {
            processed += 1;
            for &v in &adj[u] {
                // 最長路徑 DP:u 已定案(topo 序保證),鬆弛 u→v。
                if dist[u] + boot_ms[v] > dist[v] {
                    dist[v] = dist[u] + boot_ms[v];
                    parent[v] = u;
                }
                indeg[v] -= 1;
                if indeg[v] == 0 {
                    next.push(v);
                }
            }
        }
        waves.push(frontier.iter().map(|&u| u as u32).collect());
        next.sort_unstable();
        frontier = next;
    }

    if processed < n {
        return Err(extract_cycle(n, deps, &indeg));
    }

    let end = (0..n).max_by_key(|&v| dist[v]).unwrap();
    let mut critical_path = Vec::new();
    let mut cur = end;
    loop {
        critical_path.push(cur as u32);
        if parent[cur] == usize::MAX {
            break;
        }
        cur = parent[cur];
    }
    critical_path.reverse();

    Ok(BootPlan {
        waves,
        makespan_ms: dist[end],
        critical_path,
    })
}

/// Kahn 收不完 ⇒ 殘餘節點(indeg > 0)中必有環:每個殘餘節點都至少有一個
/// 殘餘前驅(已處理的前驅早把它的 indeg 減掉了),沿前驅走必重訪。
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
    // cur 此刻在環上;沿 pred 收集環員後反轉成依賴方向。
    let mut cycle = vec![cur as u32];
    let mut x = pred[cur];
    while x != cur {
        cycle.push(x as u32);
        x = pred[x];
    }
    cycle.reverse();
    Cycle(cycle)
}

/// 故障影響面:`failed` 掛掉後,再也開不了機的下游集合(不含 `failed` 自身;
/// 升冪)。BFS 前向可達,O(V + E)。
pub fn blast_radius(n: usize, deps: &[(u32, u32)], failed: u32) -> Vec<u32> {
    assert!((failed as usize) < n);
    let mut adj = vec![Vec::new(); n];
    for &(a, b) in deps {
        adj[a as usize].push(b as usize);
    }
    let mut hit = vec![false; n];
    let mut stack = vec![failed as usize];
    while let Some(u) = stack.pop() {
        for &v in &adj[u] {
            if !hit[v] {
                hit[v] = true;
                stack.push(v);
            }
        }
    }
    let mut out: Vec<u32> = (0..n).filter(|&v| hit[v]).map(|v| v as u32).collect();
    out.retain(|&v| v != failed);
    out.sort_unstable();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 菱形逐行 trace:n=4,deps 0→1、0→2、1→3、2→3,boot_ms [5,10,20,1]。
    /// - 初始 indeg [0,1,1,2] → frontier [0],dist=[5,10,20,1]。
    /// - 波 0 處理 0:鬆弛 0→1(5+10=15>10 → dist[1]=15,p=0)、
    ///   0→2(5+20=25>20 → dist[2]=25,p=0);indeg[1]=indeg[2]=0 → next=[1,2]。
    /// - 波 1 處理 1:鬆弛 1→3(15+1=16>1 → dist[3]=16,p=1);
    ///   處理 2:鬆弛 2→3(25+1=26>16 → dist[3]=26,p=2);indeg[3]=0。
    /// - 波 2 處理 3。dist=[5,15,25,26] → end=3,makespan=26;
    ///   parent 鏈 3→2→0 反轉 = [0,2,3](慢的那條臂,對)。
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
        // 0 → 1 ⇄ 2:1、2 互等,0 可開。
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
}
