// ⚠⚠ 防雷:本檔是 sim n(priority job scheduler + DAG)的解法。認題輪排在 8/3、
// 計時場 8/9——在那之前不要讀本檔(含註解與測試)。自定規則:跑題前不開 oracle/sol。

//! # job_scheduler —— priority scheduler + dependency DAG(sim n 教學版)
//!
//! ## [Clarify]
//! 題幹:`docs/interviews/sim-problems.md` sim n(彩排 harness:`rehearsals/src/sim_n_scheduler.rs`)。
//! job 帶 priority(大者急)投進來,4 個 worker 消化;done 事件**只帶 worker id**
//! (R1 同款 bus 形狀)。Phase 2:job 帶 `deps`。隱藏 spec:
//! - 同優先權要 FIFO(不然同權 job 順序不穩定,難重現、難測);
//! - **priority 不能穿越 DAG**——再急也得等料;
//! - 相依「早已完成」的後到者要立刻可派(不能等一個永遠不會再來的完成事件)。
//!
//! ## [Abstract]
//! 核心 = 兩層閘門,各自獨立:
//! 1. **DAG 入場閘**:缺幾個相依(indegree)記在 `waiting`;每個完成事件把
//!    dependents 的 `missing` 減一,**歸零才放進 ready**(Kahn 拓撲排序的事件驅動版,
//!    對照單執行緒版 [`crate::ds::graph`] 的 toposort);
//! 2. **ready 的優先權閘**:`BinaryHeap<(priority, Reverse(seq), id)>`——priority
//!    大者先;同權用 `Reverse(seq)` 保 FIFO。**seq 在到達時發,不是入場時**——
//!    等相依的 job 不因入場晚而喪失它的到達位次,平手順序才穩定。
//!
//! ## [Iterate]
//! Phase 1(無 deps):free 隊 + owner 表 + ready heap,就是 sim i 骨架換 payload
//! → Phase 2:加 `waiting`/`dependents`/`completed` 三件套,dispatch 一行不改
//! ——入場閘只決定「誰進 ready」,不碰「誰先出 ready」。
//!
//! ## [Trade-offs]
//! - ready 用 heap(O(log n) push/pop)vs 每次線性掃 max(O(n)):n 小時掃也行,
//!   但 heap 順手就有平手規則的位置(tuple 序),面試檯面上直接選 heap。
//! - `completed` 集合是「相依早已完成」的秒判——不記它就得等已逝事件,經典死等。
//! - **priority inversion**:高優先權 job 等低優先權相依。真系統的解是 priority
//!   inheritance(相依鏈暫時提權);面試指出現象 + 名詞即可,不用實作。
//! - **循環相依**:missing 永遠不歸零,job 靜默卡死。本實作不偵測(spec 沒要求),
//!   真系統要 Kahn 全量檢查或 watchdog 掃 waiting 的滯留時間——說得出口就加分。
//!
//! ## [Dry-Run]
//! 測試 `priority_waves_tie_broken_by_arrival` 的手 trace:5 job 同 tick 到
//! (id1 pri1、id2 pri9、id3 pri5、id4 pri9、id5 pri3),4 台 worker:
//! register 全收(seq = 到達序 0..5)→ dispatch:heap 依 (pri, Reverse(seq)) 出
//! 2(9,seq1) → 4(9,seq3) → 3(5) → 5(3),四台佔滿;1(1) 續留 →
//! 首個 done 釋出 worker → 1 派上。assigned_log 的 job 序 = [2,4,3,5,1]。
//!
//! 對照:彩排解答 `rehearsals/examples/sol_sim_n_scheduler.rs`(同設計,單檔面試版)。

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

// ===================== 題目給的介面(與 sim n 相同,英文保留)=====================

#[derive(Clone, Debug)]
pub struct Job {
    pub job_id: u64,
    pub priority: u8, // higher = more urgent
    /// Non-empty only in Phase 2; always empty during Phase 1.
    pub deps: Vec<u64>,
}

pub const WORKER_COUNT: u32 = 4;

/// Same bus shape as R1: done events carry **only the worker id.**
pub trait JobBus {
    /// Pull the next incoming job, if any.
    fn get_job(&mut self) -> Option<Job>;
    fn assign_job_to_worker(&mut self, worker_id: u32, job_id: u64);
    /// Which worker just finished.
    fn get_worker_done(&mut self) -> Option<u32>;
    /// Returns `false` when the simulation is exhausted (real interview: always `true`).
    fn wait_event(&mut self) -> bool;
    fn submit_job_done(&mut self, job_id: u64);
}

// ===================== 實作 =====================

/// 還在等相依的 job:記優先權、**到達序**、缺幾個相依。
struct Waiting {
    priority: u8,
    seq: u64,
    missing: usize,
}

/// 兩層閘門 + R1 骨架(free 隊、owner 表)。
pub struct Scheduler {
    free: VecDeque<u32>,
    /// worker → job:done 只給 worker id,路由靠它(sim i 的 owner 表同款)。
    owner: [Option<u64>; WORKER_COUNT as usize],
    /// 優先權閘:(priority, Reverse(到達序), job_id)——大 pri 先、同 pri 早到先。
    ready: BinaryHeap<(u8, Reverse<u64>, u64)>,
    /// 到達序發號機(到達時發,不是入場時)。
    seq: u64,
    /// DAG 入場閘:job_id → 還缺幾個相依。
    waiting: HashMap<u64, Waiting>,
    /// 反向邊:dep → 等它的人(完成事件靠它推進)。
    dependents: HashMap<u64, Vec<u64>>,
    /// 「相依早已完成」的秒判集合——不記它就得等已逝事件。
    completed: HashSet<u64>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            free: (0..WORKER_COUNT).collect(),
            owner: [None; WORKER_COUNT as usize],
            ready: BinaryHeap::new(),
            seq: 0,
            waiting: HashMap::new(),
            dependents: HashMap::new(),
            completed: HashSet::new(),
        }
    }

    /// 收 job:發到達序;只數「還沒完成」的相依(已完成的不欠);
    /// missing == 0 直接進 ready,否則掛 waiting + 登記反向邊。
    pub fn register(&mut self, j: Job) {
        let seq = self.seq;
        self.seq += 1;
        let missing = j
            .deps
            .iter()
            .filter(|d| !self.completed.contains(d))
            .count();
        for d in &j.deps {
            if !self.completed.contains(d) {
                self.dependents.entry(*d).or_default().push(j.job_id);
            }
        }
        if missing == 0 {
            self.ready.push((j.priority, Reverse(seq), j.job_id));
        } else {
            self.waiting.insert(
                j.job_id,
                Waiting {
                    priority: j.priority,
                    seq,
                    missing,
                },
            );
        }
    }

    /// 派工:有空 worker 就從 ready 出堆(優先權閘決定順序)。O(log n) each。
    pub fn dispatch(&mut self, bus: &mut impl JobBus) {
        while !self.free.is_empty() {
            let Some((_, _, id)) = self.ready.pop() else {
                return;
            };
            let w = self.free.pop_front().unwrap();
            bus.assign_job_to_worker(w, id);
            self.owner[w as usize] = Some(id);
        }
    }

    /// 收工:owner 路由回 job → submit → 完成事件推 DAG:
    /// 等它的人 missing 減一,歸零放行進 ready(帶原到達序)。
    pub fn on_done(&mut self, bus: &mut impl JobBus, w: u32) {
        let id = self.owner[w as usize]
            .take()
            .expect("done 來自沒派工的 worker");
        self.free.push_back(w);
        bus.submit_job_done(id);
        self.completed.insert(id);
        for dep_id in self.dependents.remove(&id).unwrap_or_default() {
            let wtg = self
                .waiting
                .get_mut(&dep_id)
                .expect("dependents 指向不存在的 waiting");
            wtg.missing -= 1;
            if wtg.missing == 0 {
                let wtg = self.waiting.remove(&dep_id).unwrap();
                self.ready.push((wtg.priority, Reverse(wtg.seq), dep_id));
            }
        }
    }
}

/// event loop:收 job → 派工 → 收工;收工後不睡(完成事件可能放行了 waiting 的 job)。
pub fn run(bus: &mut impl JobBus) {
    let mut s = Scheduler::new();
    loop {
        while let Some(j) = bus.get_job() {
            s.register(j);
        }
        s.dispatch(bus);
        let mut progressed = false;
        while let Some(w) = bus.get_worker_done() {
            s.on_done(bus, w);
            progressed = true;
        }
        if progressed {
            continue;
        }
        if !bus.wait_event() {
            break;
        }
    }
}

// ===================== Mock(測試/教學 harness)=====================

/// 測試用假 bus(與彩排 `SimBus` 同協定)。worker 完成序預設照派工序(FIFO),
/// [`MockBus::lifo`] 反轉。oracle:**deps 沒做完就派工直接 panic**;
/// 重派、忙台派工、假 submit 同罪。
#[derive(Default)]
pub struct MockBus {
    tick: u64,
    lifo: bool,
    jobs: VecDeque<(u64, Job)>,
    in_flight: Vec<(u32, u64)>, // (worker, job),按派工序
    done_ready: VecDeque<u32>,
    worker_busy: [bool; WORKER_COUNT as usize],
    // oracle 帳
    known: HashMap<u64, Job>,
    assigned: HashSet<u64>,
    completed: HashSet<u64>,
    /// 對外斷言用:每次派工 (worker, job),順序就是你的排程決策順序。
    pub assigned_log: Vec<(u32, u64)>,
    /// 對外斷言用:成功回報的 job(按順序)。
    pub submitted: Vec<u64>,
}

impl MockBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// worker 完成順序改成「後派先完」。
    pub fn lifo(mut self) -> Self {
        self.lifo = true;
        self
    }

    /// 在第 `tick` 次 wait_event 之後投遞一個 job。
    pub fn job_at(mut self, tick: u64, id: u64, priority: u8, deps: &[u64]) -> Self {
        self.jobs.push_back((
            tick,
            Job {
                job_id: id,
                priority,
                deps: deps.to_vec(),
            },
        ));
        self
    }
}

impl JobBus for MockBus {
    fn get_job(&mut self) -> Option<Job> {
        if self.jobs.front().is_some_and(|(t, _)| *t <= self.tick) {
            let (_, j) = self.jobs.pop_front().unwrap();
            self.known.insert(j.job_id, j.clone());
            return Some(j);
        }
        None
    }

    fn assign_job_to_worker(&mut self, worker_id: u32, job_id: u64) {
        assert!(worker_id < WORKER_COUNT, "worker id {worker_id} 越界");
        let w = worker_id as usize;
        assert!(
            !self.worker_busy[w],
            "worker {worker_id} 還在忙(它的 done 還沒被收走)就再被派工"
        );
        let job = self
            .known
            .get(&job_id)
            .unwrap_or_else(|| panic!("派了不認識的 job {job_id}"));
        for d in &job.deps {
            assert!(
                self.completed.contains(d),
                "job {job_id} 的相依 {d} 還沒完成就被派工——DAG 入場控制破了"
            );
        }
        assert!(self.assigned.insert(job_id), "job {job_id} 被派了兩次");
        self.worker_busy[w] = true;
        self.in_flight.push((worker_id, job_id));
        self.assigned_log.push((worker_id, job_id));
    }

    fn get_worker_done(&mut self) -> Option<u32> {
        let w = self.done_ready.pop_front()?;
        self.worker_busy[w as usize] = false;
        Some(w)
    }

    fn wait_event(&mut self) -> bool {
        self.tick += 1;
        if !self.in_flight.is_empty() {
            let i = if self.lifo {
                self.in_flight.len() - 1
            } else {
                0
            };
            let (w, j) = self.in_flight.remove(i);
            self.completed.insert(j);
            self.done_ready.push_back(w);
            return true;
        }
        !self.jobs.is_empty() || !self.done_ready.is_empty()
    }

    fn submit_job_done(&mut self, job_id: u64) {
        assert!(
            self.known.contains_key(&job_id),
            "submit 了不認識的 job {job_id}"
        );
        assert!(
            self.completed.contains(&job_id),
            "job {job_id} 還沒做完就 submit"
        );
        assert!(
            !self.submitted.contains(&job_id),
            "job {job_id} 被 submit 兩次"
        );
        self.submitted.push(job_id);
    }
}

// ===================== 測試 =====================

#[cfg(test)]
mod tests {
    use super::*;

    /// 檔頭 [Dry-Run] 的劇本:5 job 同時到、4 台 worker,
    /// 派工序 = [2, 4, 3, 5, 1](pri 9,9,5,3,1;同 pri 9 者早到先)。
    #[test]
    fn priority_waves_tie_broken_by_arrival() {
        let mut bus = MockBus::new()
            .job_at(0, 1, 1, &[])
            .job_at(0, 2, 9, &[])
            .job_at(0, 3, 5, &[])
            .job_at(0, 4, 9, &[])
            .job_at(0, 5, 3, &[]);
        run(&mut bus);
        let order: Vec<_> = bus.assigned_log.iter().map(|&(_, j)| j).collect();
        assert_eq!(order, vec![2, 4, 3, 5, 1]);
    }

    /// 同權全 FIFO:6 個 pri7 的 job,派工序 == 到達序。
    /// (heap 的 tuple 少了 Reverse(seq) 這欄,這條就會亂。)
    #[test]
    fn equal_priority_is_fifo() {
        let mut bus = MockBus::new()
            .job_at(0, 10, 7, &[])
            .job_at(0, 11, 7, &[])
            .job_at(0, 12, 7, &[])
            .job_at(0, 13, 7, &[])
            .job_at(0, 14, 7, &[])
            .job_at(0, 15, 7, &[]);
        run(&mut bus);
        let order: Vec<_> = bus.assigned_log.iter().map(|&(_, j)| j).collect();
        assert_eq!(order, vec![10, 11, 12, 13, 14, 15]);
    }

    /// DAG 入場閘:pri9 的 job3 等 pri5 的 job1、job2;pri9 的 job4 等 job3——
    /// priority 不能穿越 DAG(oracle 會抓提前派工),完成序 [1, 2, 3, 4]。
    /// 這正是 priority inversion 的現場:真系統用 priority inheritance,講出即可。
    #[test]
    fn dag_gates_priority() {
        let mut bus = MockBus::new()
            .job_at(0, 1, 5, &[])
            .job_at(0, 2, 5, &[])
            .job_at(0, 3, 9, &[1, 2])
            .job_at(0, 4, 9, &[3]);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![1, 2, 3, 4]);
    }

    /// 相依早已完成的後到者(tick 5 才來、相依 tick 0 早跑完):
    /// completed 集合秒判、立刻可派——沒這個集合就是死等已逝事件。
    #[test]
    fn late_arrival_with_completed_dep_dispatches_immediately() {
        let mut bus = MockBus::new().job_at(0, 1, 5, &[]).job_at(5, 2, 5, &[1]);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![1, 2]);
    }

    /// 循環相依(2↔3):missing 永不歸零 → 兩個 job 不派工也不 submit,
    /// 其餘照常、事件耗盡後 run 正常退出(不 hang)。
    /// 本實作不偵測循環(spec 沒要求)——真系統要 Kahn 全量檢查或掃 waiting 滯留。
    #[test]
    fn dependency_cycle_stalls_only_the_cycle() {
        let mut bus = MockBus::new()
            .job_at(0, 1, 5, &[])
            .job_at(0, 2, 9, &[3])
            .job_at(0, 3, 9, &[2]);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![1]);
        assert_eq!(bus.assigned_log.len(), 1, "循環裡的 job 不准被派工");
    }
}
