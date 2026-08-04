// ═══ 8/5 taper 修 code 段(小量版,~20m)═══
//
// 規則:兩題、**每題各埋 2 個 bug**,全部編譯得過(bug 在邏輯層)。
// 流程:①先只用眼睛讀,把 4 個 bug 圈出來(提前抓錯=面試 review 肌肉)
//       ②改碼 → `cargo run -p rehearsals --example bughunt_0805` → 直到「全綠」
// 答案鍵在 `scratch/taper_0805.md` 檔尾 §F′——圈完才准開。
// 兩題都是舊識:練習 1 = a 題(ring drop-oldest),練習 2 = sim o(Kahn 波次)。

/// 練習 1:固定容量 ring,滿了丟最舊(drop-oldest),`dropped` 記丟了幾筆。
struct Ring {
    buf: Vec<u64>,
    head: usize,
    len: usize,
    dropped: u64,
}

impl Ring {
    fn new(cap: usize) -> Self {
        assert!(cap > 0);
        Ring {
            buf: vec![0; cap],
            head: 0,
            len: 0,
            dropped: 0,
        }
    }

    fn push(&mut self, v: u64) {
        let cap = self.buf.len();
        if self.len == cap {
            // 滿:淘汰一筆、收下新值
            let tail = (self.head + self.len - 1) % cap;
            self.buf[tail] = v;
            return;
        }
        let tail = (self.head + self.len) % cap;
        self.buf[tail] = v;
        self.len += 1;
    }

    fn pop(&mut self) -> Option<u64> {
        if self.len == 0 {
            return None;
        }
        let v = self.buf[self.head];
        self.head = (self.head + 1) % self.buf.len();
        self.len -= 1;
        Some(v)
    }
}

/// 練習 2:Kahn 波次——回傳每一波可同時處理的節點(各波內排序),有環回 Err。
#[derive(Debug)]
struct CycleError;

fn kahn_waves(n: usize, edges: &[(usize, usize)]) -> Result<Vec<Vec<usize>>, CycleError> {
    let mut indeg = vec![0usize; n];
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &(u, v) in edges {
        adj[u].push(v);
        indeg[v] += 1;
    }
    let mut frontier: Vec<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
    let mut waves = Vec::new();
    while !frontier.is_empty() {
        frontier.sort_unstable();
        let mut next = Vec::new();
        for &u in &frontier {
            for &v in &adj[u] {
                indeg[v] -= 1;
                next.push(v);
            }
        }
        waves.push(frontier);
        frontier = next;
    }
    Ok(waves)
}

fn main() {
    // 練習 1 驗收
    let mut r = Ring::new(3);
    for v in 1..=5 {
        r.push(v);
    }
    assert_eq!(r.dropped, 2, "ring: dropped 計數");
    let out: Vec<u64> = std::iter::from_fn(|| r.pop()).collect();
    assert_eq!(
        out,
        vec![3, 4, 5],
        "ring: drop-oldest 序(留下的該是最新三筆)"
    );

    // 練習 2 驗收(菱形 DAG + 兩節點環)
    let waves = kahn_waves(4, &[(0, 1), (0, 2), (1, 3), (2, 3)]).expect("DAG 不該報環");
    assert_eq!(waves, vec![vec![0], vec![1, 2], vec![3]], "kahn: 波次");
    assert!(
        kahn_waves(2, &[(0, 1), (1, 0)]).is_err(),
        "kahn: 環要報 Err"
    );

    println!("全綠——四個 bug 都修掉了");
}
