//! sim n —— priority job scheduler(題幹:`docs/interviews/sim-problems.md`;Phase 2 由面試官放)。
//!
//! 彩排規則同 sim_i:實作+自寫測試在本檔;`tests/sim_n_scheduler_test.rs` 跑完才開。

use std::collections::{HashMap, HashSet, VecDeque};

// ===================== 題目給的介面(可讀)=====================

#[derive(Clone, Debug)]
pub struct Job {
    pub job_id: u64,
    pub priority: u8, // 越大越急
    /// Phase 2 才會出現非空 deps;Phase 1 期間永遠是空的,可以先不理它。
    pub deps: Vec<u64>,
}

pub const WORKER_COUNT: u32 = 4;

/// 與 R1 同款的 bus 形狀:done 一樣**只給 worker id**。
pub trait JobBus {
    fn get_job(&mut self) -> Option<Job>;
    fn assign_job_to_worker(&mut self, worker_id: u32, job_id: u64);
    /// 哪台 worker 剛做完。
    fn get_worker_done(&mut self) -> Option<u32>;
    /// 回傳 `false` = 模擬結束(真面試 = 永遠 `true`)。
    fn wait_event(&mut self) -> bool;
    fn submit_job_done(&mut self, job_id: u64);
}

// ===================== 作答區 =====================

/// 接 job、按優先權派給 4 台 worker,做完回報。Phase 2 會加相依。
pub fn run(bus: &mut impl JobBus) {
    todo!("彩排時實作;ready 的資料結構選擇是考點之一")
}

// ============ SimBus(模擬硬體;⚠ 跑題前不准細讀)============

/// worker 完成順序:預設照派工序(FIFO),`lifo()` 反轉。
/// oracle:**deps 沒做完就派工直接 panic**;重派、忙台派工、假 submit 同罪。
#[derive(Default)]
pub struct SimBus {
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

impl SimBus {
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

impl JobBus for SimBus {
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
