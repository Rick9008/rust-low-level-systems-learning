//! drill:thread_pool —— 填 worker 迴圈與 shutdown。
//!
//! 已給:結構、new、execute。
//! 要填:`worker_loop`(醒來先查 stop!)與 `Drop`(join 全部)。
//! 經典死法:predicate 忘了看 stop → drop 永久卡在 join。

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct State {
    jobs: VecDeque<Job>,
    stop: bool,
}

struct Shared {
    state: Mutex<State>,
    cv: Condvar,
}

pub struct ThreadPool {
    shared: Arc<Shared>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> Self {
        assert!(num_threads > 0);
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                jobs: VecDeque::new(),
                stop: false,
            }),
            cv: Condvar::new(),
        });
        let workers = (0..num_threads)
            .map(|_| {
                let shared = Arc::clone(&shared);
                thread::spawn(move || worker_loop(&shared))
            })
            .collect();
        Self { shared, workers }
    }

    pub fn execute(&self, job: impl FnOnce() + Send + 'static) {
        let mut st = self.shared.state.lock().unwrap();
        st.jobs.push_back(Box::new(job));
        drop(st);
        self.shared.cv.notify_one();
    }
}

/// spec:worker 主迴圈。
/// - 等待條件:「佇列空 **且** 未 stop」才睡(wait_while)
/// - 醒來:佇列有 job → 拿一個、**解鎖後**執行(鎖內執行 = 池退化成序列)
/// - 佇列空且 stop → return(drain-then-exit 語意:stop 後先清空佇列)
/// - 加分:job 用 catch_unwind 包住,panic 不殺 worker
fn worker_loop(shared: &Shared) {
    todo!("spec: loop {{ wait_while(空 && !stop); 有 job 拿出鎖外執行; 空+stop 則 return }}")
}

impl Drop for ThreadPool {
    /// spec:置 stop → notify_all(所有 worker 都要看到)→ join 全部。
    /// 返回時保證:所有已提交 job 執行完畢。
    fn drop(&mut self) {
        todo!("spec: set stop; notify_all; join workers(用 self.workers.drain(..))")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// boundary:100 job 全執行,drop 返回後 counter 必為 100。
    #[test]
    #[ignore = "填完 worker_loop/Drop 後移除"]
    fn executes_all_jobs() {
        let counter = Arc::new(AtomicUsize::new(0));
        {
            let pool = ThreadPool::new(4);
            for _ in 0..100 {
                let c = Arc::clone(&counter);
                pool.execute(move || {
                    c.fetch_add(1, Ordering::Relaxed);
                });
            }
        }
        assert_eq!(counter.load(Ordering::Relaxed), 100);
    }

    /// boundary:零 job 直接 drop 不 hang——predicate 沒查 stop 就會卡死在這。
    #[test]
    #[ignore = "填完 worker_loop/Drop 後移除"]
    fn drop_with_zero_jobs_does_not_hang() {
        let pool = ThreadPool::new(4);
        drop(pool);
    }

    /// boundary:單 worker FIFO 序列執行。
    #[test]
    #[ignore = "填完 worker_loop/Drop 後移除"]
    fn single_worker_fifo() {
        let order = Arc::new(Mutex::new(Vec::new()));
        {
            let pool = ThreadPool::new(1);
            for i in 0..10 {
                let order = Arc::clone(&order);
                pool.execute(move || order.lock().unwrap().push(i));
            }
        }
        assert_eq!(*order.lock().unwrap(), (0..10).collect::<Vec<_>>());
    }
}
