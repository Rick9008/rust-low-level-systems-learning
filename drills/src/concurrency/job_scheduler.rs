// ⚠⚠ 防雷:本檔是 sim n(priority job scheduler + DAG)的填空版,spec 註解含解法方向。
// 本檔就是 8/3 lite 場材料(7/31 改制,自 8/9 前移):開跑即用、開跑前不要讀。自定規則:跑題前不開 oracle/sol。

//! drill:job_scheduler —— 填兩層閘門的三個轉移函式(sim n 的填空版)。
//!
//! 已給:題目介面與 `MockBus`(借 reference 的)、`Scheduler` 結構
//! (兩層閘門的資料結構)、`run` 事件迴圈。
//! 要填:`register` / `dispatch` / `on_done`。
//!
//! 核心不變量:
//! - **DAG 入場閘**:missing(還缺幾個相依)歸零才放進 ready——priority
//!   不能穿越 DAG,再急也得等料(oracle 會抓提前派工);
//! - **優先權閘**:heap 序 = (priority, Reverse(到達序))——同權 FIFO;
//!   **seq 在到達時發**,等相依的 job 不因入場晚而喪失位次;
//! - 相依早已完成的後到者靠 `completed` 集合秒判,立刻可派——
//!   不然就是等一個永遠不會再來的完成事件。
//!
//! 設計取捨(heap vs 線性掃、priority inversion、循環相依怎辦)見 reference
//! 同名模組檔頭(**跑過 sim n 之後**再讀)。

use reference::concurrency::job_scheduler::{Job, JobBus, WORKER_COUNT};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

/// 還在等相依的 job:記優先權、**到達序**、缺幾個相依。
pub struct Waiting {
    pub priority: u8,
    pub seq: u64,
    pub missing: usize,
}

/// 兩層閘門 + R1 骨架(free 隊、owner 表)。
pub struct Scheduler {
    pub free: VecDeque<u32>,
    /// worker → job:done 只給 worker id,路由靠它。
    pub owner: [Option<u64>; WORKER_COUNT as usize],
    /// 優先權閘:(priority, Reverse(到達序), job_id)。
    pub ready: BinaryHeap<(u8, Reverse<u64>, u64)>,
    /// 到達序發號機。
    pub seq: u64,
    /// DAG 入場閘:job_id → Waiting。
    pub waiting: HashMap<u64, Waiting>,
    /// 反向邊:dep → 等它的人。
    pub dependents: HashMap<u64, Vec<u64>>,
    /// 「相依早已完成」的秒判集合。
    pub completed: HashSet<u64>,
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

    /// spec:收 job。發到達序(`seq`,發完 +1);missing = deps 裡**還沒完成**的
    /// 數量(查 `completed`);未完成的每個 dep 都要登記反向邊 `dependents[dep]`。
    /// missing == 0 → 進 `ready`(帶 priority 與 Reverse(seq));否則掛 `waiting`。
    pub fn register(&mut self, j: Job) {
        // todo!("spec: 發 seq;數未完成相依;0 → ready,>0 → waiting + 反向邊")
        let Job {
            job_id: jid,
            priority: p,
            mut deps,
        } = j;
        let seq = self.seq;
        self.seq += 1;
        deps.retain(|dep| !self.completed.contains(dep));
        if deps.is_empty() {
            self.ready.push((p, Reverse(seq), jid));
            return;
        }
        for dep in &deps {
            self.dependents.entry(*dep).or_default().push(jid);
        }
        self.waiting.insert(
            jid,
            Waiting {
                priority: p,
                seq,
                missing: deps.len(),
            },
        );
    }

    /// spec:派工。有空 worker 且 ready 非空就 pop(heap 序自動給出「大 pri 先、
    /// 同 pri 早到先」)→ `assign_job_to_worker` → 記 `owner[w]`。
    pub fn dispatch(&mut self, bus: &mut impl JobBus) {
        // todo!("spec: 有空有 ready 就派;owner 當下記")
        loop {
            if self.free.is_empty() || self.ready.is_empty() {
                return;
            }
            let free_wid = self.free.pop_front().unwrap();
            assert!((free_wid as usize) < self.owner.len());
            let (p, Reverse(seq), jid) = self.ready.pop().unwrap();
            bus.assign_job_to_worker(free_wid, jid);
            self.owner[free_wid as usize] = Some(jid);
        }
    }

    /// spec:收工。`owner[w].take()` 路由回 job → 還 worker → `submit_job_done`
    /// → 記 `completed` → 完成事件推 DAG:`dependents.remove(&id)` 的每個等待者
    /// missing 減一,**歸零的移出 waiting、帶原到達序進 ready**。
    pub fn on_done(&mut self, bus: &mut impl JobBus, w: u32) {
        // todo!("spec: owner 路由;submit;completed;推 DAG 放行歸零者")
        assert!(self.owner[w as usize].is_some());
        let jid = self.owner[w as usize].take().unwrap();
        if let Some(childs) = self.dependents.remove(&jid) {
            for child in childs {
                // Invariant, in register it must register the jid into waiting.
                let wait_job = self.waiting.get_mut(&child).unwrap();
                wait_job.missing -= 1;
                if wait_job.missing == 0 {
                    self.ready
                        .push((wait_job.priority, Reverse(wait_job.seq), child));
                    self.waiting.remove(&child);
                }
            }
        }
        self.free.push_back(w);
        self.completed.insert(jid);
        bus.submit_job_done(jid);
    }
}

/// event loop(已給):收 job → 派工 → 收工;收工後不睡
/// (完成事件可能放行了 waiting 的 job,回頭再派一輪)。
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

#[cfg(test)]
mod tests {
    use super::*;
    use reference::concurrency::job_scheduler::MockBus;

    /// 5 job 同時到、4 台 worker:派工序 [2, 4, 3, 5, 1](pri 9,9,5,3,1;同 pri 早到先)。
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

    /// 同權全 FIFO:6 個 pri7,派工序 == 到達序(heap 少了 Reverse(seq) 就會亂)。
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

    /// DAG 入場閘:pri9 等 pri5 的相依——priority 不能穿越 DAG,完成序 [1,2,3,4]。
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

    /// 後到而相依早已完成:completed 集合秒判、立刻可派。
    #[test]
    fn late_arrival_with_completed_dep_dispatches_immediately() {
        let mut bus = MockBus::new().job_at(0, 1, 5, &[]).job_at(5, 2, 5, &[1]);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![1, 2]);
    }

    /// 循環相依(2↔3):不派工、不 submit、不 hang;其餘照常。
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
