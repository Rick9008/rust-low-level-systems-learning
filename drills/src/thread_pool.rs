//! drill:thread_pool —— 填 worker 迴圈、shutdown,與有回傳值的 submit。
//!
//! 已給:結構、new、execute、JobHandle 的結構。
//! 要填:`worker_loop`(醒來先查 stop!)、`Drop`(join 全部)、
//! `submit` / `JobHandle::join`(one-shot rendezvous 的 condvar 版——
//! async 版對照 reference 的 `file_io_offload::JoinFuture`)。
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

    /// spec:execute 的有回傳版。
    /// 1. 建 `Arc<(Mutex<Option<thread::Result<T>>>, Condvar)>`,slot 起始 None。
    /// 2. clone 一份給 job;job 裡:`catch_unwind(AssertUnwindSafe(f))` 的結果
    ///    放進 slot(`Some(result)`),然後 notify_one。
    /// 3. 回 `JobHandle { state }`。
    pub fn submit<T, F>(&self, f: F) -> JobHandle<T>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        todo!("spec: 建 rendezvous state; execute(放結果+notify); 回 JobHandle")
    }
}

/// `submit` 的收據:一次性 rendezvous 的同步版(condvar 睡)。
pub struct JobHandle<T> {
    /// None:還沒好;Some(Ok):結果;Some(Err):worker 的 panic payload。
    state: Arc<(Mutex<Option<thread::Result<T>>>, Condvar)>,
}

impl<T> JobHandle<T> {
    /// spec:阻塞直到結果出現。
    /// 1. `wait_while(slot, |s| s.is_none())`——「醒來重查」跟 bounded_queue
    ///    的 predicate-wait 是同一顆肌肉。
    /// 2. take 出來:Ok(v) → v;Err(panic) → `std::panic::resume_unwind(panic)`
    ///    (錯誤跟著在乎它的人走,worker 不陪葬)。
    pub fn join(self) -> T {
        todo!("spec: wait_while None; take; Ok 回值 / Err resume_unwind")
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

    /// submit 取值 + 先完成後 join(condvar 沒用上的路徑)。
    #[test]
    #[ignore = "填完 submit/join 後移除"]
    fn submit_returns_value() {
        let pool = ThreadPool::new(2);
        assert_eq!(pool.submit(|| 6 * 7).join(), 42);

        let h = pool.submit(|| "done");
        thread::sleep(std::time::Duration::from_millis(50));
        assert_eq!(h.join(), "done");
    }

    /// boundary:job panic → join 端重拋,且 worker 活著。
    #[test]
    #[ignore = "填完 submit/join 後移除"]
    fn submit_panic_rethrown_worker_survives() {
        let pool = ThreadPool::new(1);
        let h = pool.submit(|| panic!("kaboom (expected in test output)"));
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| h.join()));
        assert!(caught.is_err());
        assert_eq!(pool.submit(|| 7).join(), 7);
    }

    /// boundary:execute 的 job panic 不殺 worker——單 worker 最嚴格:
    /// 沒 catch_unwind 的話,第二個 job 永遠不會跑。
    #[test]
    #[ignore = "填完 worker_loop/Drop 後移除"]
    fn panicking_job_does_not_kill_worker() {
        let ran_after = Arc::new(AtomicUsize::new(0));
        {
            let pool = ThreadPool::new(1);
            pool.execute(|| panic!("boom (expected in test output)"));
            let c = Arc::clone(&ran_after);
            pool.execute(move || {
                c.fetch_add(1, Ordering::Relaxed);
            });
        }
        assert_eq!(ran_after.load(Ordering::Relaxed), 1);
    }

    /// boundary:drop 等慢 job 做完(drain 語意)——不是丟棄。
    #[test]
    #[ignore = "填完 worker_loop/Drop 後移除"]
    fn drop_waits_for_slow_jobs() {
        let counter = Arc::new(AtomicUsize::new(0));
        {
            let pool = ThreadPool::new(2);
            for _ in 0..6 {
                let c = Arc::clone(&counter);
                pool.execute(move || {
                    thread::sleep(std::time::Duration::from_millis(20));
                    c.fetch_add(1, Ordering::Relaxed);
                });
            }
        }
        assert_eq!(counter.load(Ordering::Relaxed), 6);
    }

    /// 並發多 handle:結果各歸各的收據,不串音。
    #[test]
    #[ignore = "填完 submit/join 後移除"]
    fn submit_many_results_isolated() {
        let pool = ThreadPool::new(4);
        let handles: Vec<_> = (0..8).map(|i| pool.submit(move || i * i)).collect();
        let results: Vec<i32> = handles.into_iter().map(JobHandle::join).collect();
        assert_eq!(results, vec![0, 1, 4, 9, 16, 25, 36, 49]);
    }
}
