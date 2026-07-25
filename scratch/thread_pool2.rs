// 3:26

use std::collections::VecDeque;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

type Job = dyn FnOnce() + Send + 'static;

struct Shared {
    jobs: Mutex<VecDeque<Box<Job>>>,
    shutdown: AtomicBool,
    wait_job: Condvar,
}

impl Shared {
    fn new() -> Self {
        Self {
            jobs: Mutex::new(VecDeque::new()),
            shutdown: AtomicBool::new(false),
            wait_job: Condvar::new(),
        }
    }
}

struct JobHandle<T> {
    // fisrt is return, second is wait value
    slot: Arc<(Mutex<Option<thread::Result<T>>>, Condvar)>,
}

impl<T> JobHandle<T> {
    fn new() -> Self {
        Self {
            slot: Arc::new((Mutex::new(None), Condvar::new())),
        }
    }
    fn join(self) -> thread::Result<T> {
        let mut slot_gaurd = self.slot.0.lock().unwrap();
        slot_gaurd = self.slot.1.wait_while(slot_gaurd, |s| s.is_none()).unwrap();
        slot_gaurd.take().expect("it should be some")
    }
}
/*

API(簽名 = 合約)
- ThreadPool::new(num_threads: usize) -> Self — num_threads > 0
- execute(&self, job: impl FnOnce() + Send + 'static) — 射後不理
- submit<T: Send + 'static>(&self, f: impl FnOnce() -> T + Send + 'static) -> JobHandle<T> — 有回傳值
- JobHandle<T>::join(self) -> thread::Result<T> — 阻塞等結果
- impl Drop for ThreadPool

行為合約
1. N 條 worker 共享一條 job 佇列;閒著要睡,不准 busy-spin。
2. Graceful shutdown:pool 被 drop 時,佇列裡已提交的 job 全部跑完才收工;drop 要等到所有 worker join 完才返回。
3. job panic 隔離:單一 job panic 不可毒鎖、不可拖垮其他 worker 或整個 pool。
4. submit 的回傳 = oneshot promise:job 跑完把結果塞進共享 slot 再通知,join 等到有值才取。
*/

struct ThreadPool {
    joins: Vec<JoinHandle<()>>,
    shared: Arc<Shared>,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> Self {
        assert!(num_threads > 0);
        let shared = Arc::new(Shared::new());
        Self {
            joins: (0..num_threads)
                .map(|_| {
                    let arc_sha = shared.clone();
                    thread::spawn(move || {
                        let mut job_gaurd = arc_sha.jobs.lock().unwrap();
                        while !job_gaurd.is_empty() || !arc_sha.shutdown.load(Ordering::Acquire) {
                            job_gaurd = arc_sha
                                .wait_job
                                .wait_while(job_gaurd, |s| {
                                    s.is_empty() && !arc_sha.shutdown.load(Ordering::Acquire)
                                })
                                .unwrap();
                            let job_op = job_gaurd.pop_front();
                            drop(job_gaurd);
                            if let Some(job) = job_op {
                                let _ = catch_unwind(AssertUnwindSafe(job));
                            }
                            job_gaurd = arc_sha.jobs.lock().unwrap();
                        }
                    })
                })
                .collect(),
            shared,
        }
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, job: F) -> Result<(), F> {
        if self.shared.shutdown.load(Ordering::Acquire) {
            return Err(job);
        }
        let mut jobs = self.shared.jobs.lock().unwrap();
        jobs.push_back(Box::new(job));
        drop(jobs);
        self.shared.wait_job.notify_one();
        Ok(())
    }

    pub fn submit<T: Send + 'static, F: FnOnce() -> T + Send + 'static>(
        &self,
        job: F,
    ) -> Result<JobHandle<T>, F> {
        if self.shared.shutdown.load(Ordering::Acquire) {
            return Err(job);
        }
        let job_handle: JobHandle<T> = JobHandle::new();
        let mut jobs = self.shared.jobs.lock().unwrap();
        let slot = job_handle.slot.clone();
        jobs.push_back(Box::new(move || {
            let ret = catch_unwind(AssertUnwindSafe(job));
            *slot.0.lock().unwrap() = Some(ret);
            slot.1.notify_one();
        }));
        self.shared.wait_job.notify_one();
        Ok(job_handle)
    }

    pub fn shutdown(&mut self) {
        {
            let _job_lock = self.shared.jobs.lock().unwrap();
            self.shared.shutdown.store(true, Ordering::Release);
        }
        self.shared.wait_job.notify_all();
        self.joins
            .drain(..)
            .for_each(|join_hand| join_hand.join().expect("Safe join."));
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// 3:56

#[test]
fn dryrun() {
    let mut pool = ThreadPool::new(3);
    let ans = pool.submit(|| 4);
    assert_eq!(ans.join(), 4);
}

fn main() {
    let mut pool = ThreadPool::new(3);
    let ans = pool.submit(|| 4);
    assert_eq!(ans.ok().unwrap().join().unwrap(), 4);
}
