// ═══ 背靠背 8/4 — 場二(25–30m;clarify → state 表 → 骨架,body 全 todo!)═══
//
// Problem 2 — Build-Farm Admission Gate
//
// Rust, std only, single file. You own the admission gate for a shared
// build farm.
//
// - Handler threads (owned by the RPC framework, not by you) call
//   `gate.run(tenant_id, job)`, where `job` is a closure.
// - The whole farm may execute at most N jobs at the same time.
// - No single tenant may have more than M jobs executing at the same
//   time (M < N).
// - The job executes on the calling handler thread — the gate admits;
//   it does not own worker threads.
// - The gate must support clean shutdown.
//
// Deliverable: clarify questions first (typed, English, in chat), then
// your state table (as comments here), then the skeleton with todo!()
// bodies.
//
// Part B (sizing — five-line format, whenever ready): 40,000 distinct
// tenants seen per day, 800 handler threads, N = 64, M = 4. Two numbers:
// (a) steady-state memory of the gate itself, and (b) — the one I
// actually care about — the worst-case number of handler threads parked
// inside your gate, and what that costs the system.
//
//
//
// 規則:裁決抄紙(當下寫成註解)、規則 read back、多段問編號逐答。

// RULING: only impl a struct to restrict running jobs counts and how many a tenant running
// RULING: gate.run fully blocking
// RULING: we don't have to impl thread pool
// RULING: N is small from 10 ~ hundreds
// RULING: no job size limiting
// RULING: two numbers and a map: running_total, per_tenant: HashMap<tenant_id, count>, all behind one Mutex, with one Condvar for the waiters. That's the whole gate.
// RULING: shutdown flag first, notify_all second, and every waiter's wake-up path re-checks the flag before the predicate and bails with Err. That's the complete shutdown story.
// RULING: no-FIFO, wake up check the shutdown flag, run job without holding lock, heterogeneous notify_all
// RULING: after shutdown new job return err imediately

use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};

struct InnerState {
    running: usize,
    tenant_running: HashMap<usize, usize>,
    shutdown: bool,
    cap_running: usize,
    cap_tenant: usize,
}

impl InnerState {
    pub fn new(m: usize, n: usize) -> Self {
        assert!(n > m);
        Self {
            running: 0,
            tenant_running: HashMap::new(),
            shutdown: false,
            cap_running: n,
            cap_tenant: m,
        }
    }
}

struct Inner {
    state: Mutex<InnerState>,
    wait_run: Condvar,
}
impl Inner {
    pub fn new(m: usize, n: usize) -> Self {
        assert!(n > m);
        Self {
            state: Mutex::new(InnerState::new(m, n)),
            wait_run: Condvar::new(),
        }
    }
}
#[derive(Clone)]
struct Gate {
    inner: Arc<Inner>,
}

struct Shutdown;
impl Gate {
    pub fn new(m: usize, n: usize) -> Self {
        assert!(n > m);
        Self {
            inner: Arc::new(Inner::new(m, n)),
        }
    }

    pub fn run<T>(&self, tenant_id: usize, job: impl FnOnce() -> T) -> Result<T, Shutdown> {
        // get the state
        // check can we run or wait
        // wait_while inner_state.running >= n or
        // inner_state.tenant_running.entry(tenant_id).or_default() >= m
        // add 1 in both running and tenant running

        let item = job();

        // get the state
        // decrease 1
        // if tenant running go to zero, remove
        // notify_all -> because a tenant might be still full, so we notify_all
        Ok(item)
    }

    pub fn shutdown(&self) {
        // get the state
        // turn on shutdown flag
        // notify_all
    }
}

fn main() {
    todo!()
}

// ═══════════════════════════════════════════════════════════════════
// ═══ 場二中場批改(Claude,8/4;Part B 欠帳——回家寫在這塊下面)═══
//
// 骨架已驗收 ✓:Inner{Mutex+Condvar} 同居一個 Arc、&self、remove-at-zero、
// notify_all(含理由)、shutdown = flag→notify_all、RULING 落地(補催後)。
//
// 回家先修兩筆(註解層,1 分鐘):
// 1. ✗ predicate 的 "and" → ✓ or:進場條件 = running<N **AND** tenant<M;
//    取反(De Morgan)→ **等待**條件 = running>=N **OR** tenant>=M。
//    照 "and" 寫:tenant 自家滿 M、全場未滿 N 時直接放行 = M 形同虛設。
//    (sim o 的 OR/AND 閘老題換皮:先寫進場條件,再取反,不要直接寫等待條件。)
// 2. ✗ 讀 predicate 用 entry(tenant_id).or_default() —— 看一眼就插 0 條目,
//    等不到位子的 tenant 留殭屍零,而 remove 只在釋放路徑跑,永遠掃不到
//    → 「只進不出」第四次,姿勢最隱蔽的一次。
//    ✓ 讀:tenant_running.get(&id).copied().unwrap_or(0)
//    ✓ entry() 留給確定入場後的 +1 那行。
//
// ── Part B 作答區(五行頭;參數:40k tenants/day、800 handler threads、
//    N=64、M=4;考官點名要 (a) gate 本體記憶體 (b) 最壞 parked 數+代價)──
// Given:   (題面參數全部重述——跟你設計矛盾的假設會在這行現形)
// Chain:   (主計算鏈,數字一路算到 (a) 和 (b))
// Cross:   (換一條獨立的軸驗算,不是同一條鏈換寫法)
// Sanity:  (跟這台機器的規格比量級,像不像話)
// Verdict: (數字 + 比較 + 行動;(b) 是考官真正在乎的)
// ═══════════════════════════════════════════════════════════════════
