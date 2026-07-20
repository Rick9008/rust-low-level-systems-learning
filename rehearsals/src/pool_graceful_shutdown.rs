//! rehearsal b:pool_graceful_shutdown —— 題目見 rehearsals/README.md。
//!
//! 只給 API 簽名。std-only;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`。

// provide std::thread / std::sync
// 1. what's graceful means?
// So we need to holds a thread pool to handle the services check
// and provide new(workers: usize) and submit(job: FnOnce)
// We need to holds a shutdown() for pool to ends -> Atomic Flag to handle

use std::collections::VecDeque;
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::JoinHandle;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct Pool {
    // ↓ 佔位:動手時整個換成你的設計。
    workers: usize,
    pool: Mutex<Vec<JoinHandle<()>>>,
    no_jobs: Arc<Condvar>,
    jobs: Arc<Mutex<VecDeque<Job>>>,
    shutdown: Arc<AtomicBool>,
}

/// submit 被拒(pool 已 shutdown)。
#[derive(Debug, PartialEq, Eq)]
pub struct Rejected;

impl Pool {
    /// 起 `workers` 條 worker 執行緒;`workers >= 1`。
    pub fn new(workers: usize) -> Self {
        assert!(workers >= 1);
        let jobs = Arc::new(Mutex::new(
            VecDeque::<Box<dyn FnOnce() + Send + 'static>>::new(),
        ));
        let shutdown = Arc::new(AtomicBool::new(false));
        let no_jobs = Arc::new(Condvar::new());
        Pool {
            workers,
            pool: Mutex::new(
                (0..workers)
                    .map(|_| {
                        let arc_jobs = jobs.clone();
                        let arc_shutdown = shutdown.clone();
                        let arc_no_jobs = no_jobs.clone();
                        std::thread::spawn(move || {
                            let mut jobs_guard = arc_jobs.lock().unwrap();
                            while !arc_shutdown.load(Ordering::Acquire) || !jobs_guard.is_empty() {
                                jobs_guard = arc_no_jobs
                                    .wait_while(jobs_guard, |s| {
                                        s.is_empty() && !arc_shutdown.load(Ordering::Acquire)
                                    })
                                    .unwrap();
                                let job = jobs_guard.pop_front().unwrap();
                                job()
                            }
                        })
                    })
                    .collect(),
            ),
            no_jobs,
            jobs,
            shutdown,
        }
    }

    /// 已 shutdown → `Err(Rejected)`;回 `Ok` 代表任務保證會被執行。
    pub fn submit<F>(&self, job: F) -> Result<(), Rejected>
    where
        F: FnOnce() + Send + 'static,
    {
        // todo!("rehearsal")
        if self.shutdown.load(Ordering::Acquire) {
            return Err(Rejected);
        }
        self.jobs.lock().unwrap().push_back(Box::new(job));
        self.no_jobs.notify_one();
        Ok(())
    }

    /// 阻塞到所有已接受的任務執行完;之後的 submit 一律拒絕;可重複呼叫。
    pub fn shutdown(&self) {
        // todo!("rehearsal")
        self.shutdown.store(true, Ordering::Release);
        self.no_jobs.notify_all();
        self.pool.lock().unwrap().drain(..).for_each(|join_handle| {
            let _ = join_handle.join();
        });
    }
}

// Boundary we need to check

// 1. with no jobs and shutdown instantly, to check is every job all down

// tc:
// new: O(workers)
// submit: O(1) average
// shutdown: O(len of jobs redundant*fn time)
//
// sc: O(workers + jobs size)
