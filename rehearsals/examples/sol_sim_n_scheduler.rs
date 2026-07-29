//! solution:sim n —— priority job scheduler。**寫完彩排才開。**
//!
//! 核心 = 兩層閘門:
//! 1. **DAG 入場閘**:job 缺幾個相依(indegree)記在 `waiting`;每個完成事件把 dependents
//!    的 missing 減一,歸零才放進 ready——**priority 不能穿越 DAG**,再急也得等料。
//! 2. **ready 的優先權閘**:`BinaryHeap<(priority, Reverse(seq), id)>`——priority 大者先,
//!    同權用 `Reverse(seq)` 保 FIFO(seq 在**到達時**發,不是入場時,平手順序才穩定)。
//!
//! 後到而相依已完成的 job(dep_already_completed)靠 `completed` 集合秒判——
//! 沒這個集合就會等一個永遠不會再來的完成事件,經典死等。
//!
//! 講出來加分的 trade-off:高優先權 job 等低優先權相依 = **priority inversion**;
//! 真系統的解是 priority inheritance(把相依鏈上的 job 暫時提權),這裡指出即可。
//!
//! 驗證:`cargo run -p rehearsals --example sol_sim_n_scheduler`

use rehearsals::sim_n_scheduler::{Job, JobBus, SimBus, WORKER_COUNT};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

/// 還在等相依的 job:記優先權、到達序、缺幾個相依。
struct Waiting {
    priority: u8,
    seq: u64,
    missing: usize,
}

#[derive(Default)]
struct Scheduler {
    free: VecDeque<u32>,
    owner: [Option<u64>; WORKER_COUNT as usize], // worker -> job(done 只給 worker id)
    ready: BinaryHeap<(u8, Reverse<u64>, u64)>,  // (priority, Reverse(到達序), job_id)
    seq: u64,
    waiting: HashMap<u64, Waiting>,
    dependents: HashMap<u64, Vec<u64>>, // dep -> 等它的人
    completed: HashSet<u64>,
    live: usize, // 還沒 submit 的 job 數(模擬退場條件用)
}

impl Scheduler {
    fn new() -> Self {
        let mut s = Self::default();
        s.free.extend(0..WORKER_COUNT);
        s
    }

    fn register(&mut self, j: Job) {
        let seq = self.seq;
        self.seq += 1;
        self.live += 1;
        // 只數「還沒完成」的相依——已完成的不欠。
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

    fn dispatch(&mut self, bus: &mut impl JobBus) {
        while !self.free.is_empty() {
            let Some((_, _, id)) = self.ready.pop() else {
                return;
            };
            let w = self.free.pop_front().unwrap();
            bus.assign_job_to_worker(w, id);
            self.owner[w as usize] = Some(id);
        }
    }

    fn on_done(&mut self, bus: &mut impl JobBus, w: u32) {
        let id = self.owner[w as usize]
            .take()
            .expect("done 來自沒派工的 worker");
        self.free.push_back(w);
        bus.submit_job_done(id);
        self.live -= 1;
        self.completed.insert(id);
        // 完成事件推 DAG:等它的人 missing 減一,歸零放行進 ready。
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

fn run(bus: &mut impl JobBus) {
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

fn main() {
    // scenario 1:優先權波次(同權 FIFO 破平手)。
    let mut bus = SimBus::new()
        .job_at(0, 1, 1, &[])
        .job_at(0, 2, 9, &[])
        .job_at(0, 3, 5, &[])
        .job_at(0, 4, 9, &[])
        .job_at(0, 5, 3, &[]);
    run(&mut bus);
    let order: Vec<_> = bus.assigned_log.iter().map(|&(_, j)| j).collect();
    assert_eq!(order, vec![2, 4, 3, 5, 1]);

    // scenario 2:同權全 FIFO。
    let mut bus = SimBus::new()
        .job_at(0, 10, 7, &[])
        .job_at(0, 11, 7, &[])
        .job_at(0, 12, 7, &[])
        .job_at(0, 13, 7, &[])
        .job_at(0, 14, 7, &[])
        .job_at(0, 15, 7, &[]);
    run(&mut bus);
    let order: Vec<_> = bus.assigned_log.iter().map(|&(_, j)| j).collect();
    assert_eq!(order, vec![10, 11, 12, 13, 14, 15]);

    // scenario 3:DAG 入場——高優先權也得等相依。
    let mut bus = SimBus::new()
        .job_at(0, 1, 5, &[])
        .job_at(0, 2, 5, &[])
        .job_at(0, 3, 9, &[1, 2])
        .job_at(0, 4, 9, &[3]);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![1, 2, 3, 4]);

    // scenario 4:相依早已完成的後到者,立刻可派。
    let mut bus = SimBus::new().job_at(0, 1, 5, &[]).job_at(5, 2, 5, &[1]);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![1, 2]);

    println!("sol_sim_n_scheduler: all green");
}
