//! # thread_pool —— std::thread + Condvar 的固定大小工作池
//!
//! ## [Clarify]
//! 解決:把 closure 丟給固定數量的 worker 執行,攤平 thread 建立成本
//! (spawn 一條 thread ~10μs 級 + 預設 8MB stack 保留)。
//! Constraints:std-only、job 為 `FnOnce() + Send + 'static`、無回傳值
//! (要結果就讓 job 自己寫回 channel/Arc)。
//! 預期規模:worker 數 ≈ CPU 核數,job 數不設限。
//!
//! ## [Abstract]
//! 不做 work-stealing、不做優先級、不做動態擴縮——面試時聲明 stub 掉往前走,
//! 核心是「worker 迴圈的 shutdown 正確性」。
//!
//! ## [Iterate]
//! 內部就是一個手寫的 unbounded MPMC 佇列(Mutex<VecDeque> + Condvar)——
//! 與 [`crate::bounded_queue`] 同一 idiom,差別在多了 worker 迴圈與 Drop join。
//!
//! ## [Trade-offs]
//! - shutdown 策略選「**drain 完再退**」:stop 置位後 worker 把佇列清空才退出。
//!   另一路「立即退,丟棄 pending job」適合 job 可安全丟棄的場景(如 cache 預熱)。
//!   本實作的 Drop 保證:回傳時所有已提交 job 都執行完——測試好寫、語意好講。
//! - job panic 用 `catch_unwind` 吞掉:否則 worker 悄悄死亡,池容量縮水且無人知。
//! - 對照 production:rayon(work-stealing data parallelism)、threadpool crate。
//!
//! ## [Dry-Run]
//! 見測試:全部執行、drop 排空、單 worker 序列化、panic 不殺 worker、零 job drop。

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
        assert!(num_threads > 0, "pool needs at least one worker");
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                jobs: VecDeque::new(),
                stop: false,
            }),
            cv: Condvar::new(),
        });
        let workers = (0..num_threads)
            .map(|i| {
                let shared = Arc::clone(&shared);
                thread::Builder::new()
                    .name(format!("pool-worker-{i}"))
                    .spawn(move || worker_loop(&shared))
                    .expect("spawn worker thread")
            })
            .collect();
        Self { shared, workers }
    }

    /// 提交一個 job。O(1) 持鎖時間(VecDeque push_back amortized O(1);
    /// 佇列 unbounded,如需 backpressure 换成 bounded_queue 的滿阻塞)。
    pub fn execute(&self, job: impl FnOnce() + Send + 'static) {
        let mut st = self.shared.state.lock().unwrap();
        st.jobs.push_back(Box::new(job));
        drop(st); // 先解鎖再 notify,被喚醒的 worker 不會立刻撞鎖
        self.shared.cv.notify_one();
    }
}

fn worker_loop(shared: &Shared) {
    loop {
        let job = {
            let mut st = shared.state.lock().unwrap();
            // 「醒來先查 stop」落實在 predicate 裡:等待條件 = 沒事做 且 未停。
            // 若 predicate 只查 jobs.is_empty(),Drop 置 stop 後 notify_all,
            // worker 醒來發現佇列仍空就睡回去 → join 永久卡死。
            st = shared
                .cv
                .wait_while(st, |s| s.jobs.is_empty() && !s.stop)
                .unwrap();
            match st.jobs.pop_front() {
                Some(job) => job,
                // 佇列空 ⇒(由 predicate)stop 已置位:drain 完畢,退出。
                None => return,
            }
        }; // 鎖在這裡釋放——job 在鎖外執行,不然池退化成序列執行
        // Safety of AssertUnwindSafe:job 是 FnOnce,panic 後不會再被呼叫,
        // 不存在「觀察到被 unwind 撕一半的狀態」的機會。
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(job));
    }
}

impl Drop for ThreadPool {
    /// 置 stop → notify_all → join 全部。
    /// 回傳時保證:所有已提交 job 執行完、所有 worker 已退出。
    fn drop(&mut self) {
        self.shared.state.lock().unwrap().stop = true;
        // notify_all 而非 notify_one:所有睡著的 worker 都要觀察到 stop。
        self.shared.cv.notify_all();
        for w in self.workers.drain(..) {
            // worker 本體不會 panic(job panic 已被 catch),join Err 不再傳播
            let _ = w.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    /// [Dry-Run] 100 個 job 全部執行。
    /// trace:execute×100 → 佇列累積 → 4 worker 競爭 pop → drop 置 stop、
    /// worker drain 到空才退 → join 返回 → counter 必為 100。
    /// boundary:job 數 >> worker 數(佇列會積壓)。
    #[test]
    fn executes_all_jobs_before_drop_returns() {
        let counter = Arc::new(AtomicUsize::new(0));
        {
            let pool = ThreadPool::new(4);
            for _ in 0..100 {
                let c = Arc::clone(&counter);
                pool.execute(move || {
                    c.fetch_add(1, Ordering::Relaxed);
                });
            }
        } // Drop:drain + join
        assert_eq!(counter.load(Ordering::Relaxed), 100);
    }

    /// boundary:慢 job——drop 必須等它們做完(drain 語意),不能丟棄。
    #[test]
    fn drop_waits_for_slow_jobs() {
        let counter = Arc::new(AtomicUsize::new(0));
        {
            let pool = ThreadPool::new(2);
            for _ in 0..6 {
                let c = Arc::clone(&counter);
                pool.execute(move || {
                    thread::sleep(Duration::from_millis(20));
                    c.fetch_add(1, Ordering::Relaxed);
                });
            }
        }
        assert_eq!(counter.load(Ordering::Relaxed), 6);
    }

    /// boundary:單 worker ⇒ 嚴格 FIFO 序列執行(觀察執行順序)。
    #[test]
    fn single_worker_runs_jobs_in_fifo_order() {
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

    /// boundary:job panic 不得殺死 worker——後續 job 仍要被執行。
    /// 沒有 catch_unwind 時:worker unwind 死亡,池只剩 0 條 worker,
    /// 第二個 job 永遠不會跑,drop 的 join 也不 hang(thread 已死)但工作遺失。
    #[test]
    fn panicking_job_does_not_kill_worker() {
        let ran_after_panic = Arc::new(AtomicUsize::new(0));
        {
            let pool = ThreadPool::new(1);
            pool.execute(|| panic!("boom (expected in test output)"));
            let c = Arc::clone(&ran_after_panic);
            pool.execute(move || {
                c.fetch_add(1, Ordering::Relaxed);
            });
        }
        assert_eq!(ran_after_panic.load(Ordering::Relaxed), 1);
    }

    /// boundary:零 job 直接 drop——worker 全在 wait 中,stop 必須能喚醒它們,
    /// 否則 join 永久卡死(這就是 predicate 缺 stop 檢查會炸的地方)。
    #[test]
    fn boundary_drop_with_zero_jobs_does_not_hang() {
        let pool = ThreadPool::new(4);
        drop(pool);
    }
}
