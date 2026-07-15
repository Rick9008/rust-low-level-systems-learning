//! solution:題 b pool_graceful_shutdown——**寫完彩排才開**。
//! canonical 設計:`Arc<(Mutex<State>, Condvar)>` 家族;worker 醒來先查條件、
//! 執行 job 前一定放鎖。驗證:rehearsals/tests/pool_graceful_shutdown_test.rs 全綠。

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

#[derive(Debug, PartialEq, Eq)]
pub struct Rejected;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct State {
    jobs: VecDeque<Job>,
    shutdown: bool,
}

struct Inner {
    state: Mutex<State>,
    cv: Condvar,
}

pub struct Pool {
    inner: Arc<Inner>,
    workers: Mutex<Vec<JoinHandle<()>>>,
}

impl Pool {
    pub fn new(workers: usize) -> Self {
        assert!(workers >= 1);
        let inner = Arc::new(Inner {
            state: Mutex::new(State {
                jobs: VecDeque::new(),
                shutdown: false,
            }),
            cv: Condvar::new(),
        });
        let handles = (0..workers)
            .map(|_| {
                let inner = Arc::clone(&inner);
                std::thread::spawn(move || {
                    loop {
                        let mut st = inner.state.lock().unwrap();
                        // predicate-wait:被喚醒只是提示,條件要自己再查
                        while st.jobs.is_empty() && !st.shutdown {
                            st = inner.cv.wait(st).unwrap();
                        }
                        match st.jobs.pop_front() {
                            Some(job) => {
                                drop(st); // 執行前放鎖——拿著鎖跑 job = 假 pool
                                job();
                            }
                            None => break, // shutdown 且 queue 已空
                        }
                    }
                })
            })
            .collect();
        Self {
            inner,
            workers: Mutex::new(handles),
        }
    }

    pub fn submit<F>(&self, job: F) -> Result<(), Rejected>
    where
        F: FnOnce() + Send + 'static,
    {
        let mut st = self.inner.state.lock().unwrap();
        if st.shutdown {
            return Err(Rejected);
        }
        st.jobs.push_back(Box::new(job));
        self.inner.cv.notify_one();
        Ok(())
    }

    /// 等舊的(已接受的全部跑完)、拒新的;可重複呼叫(第二次 join 空集合)。
    pub fn shutdown(&self) {
        {
            let mut st = self.inner.state.lock().unwrap();
            st.shutdown = true;
            self.inner.cv.notify_all();
        }
        let handles: Vec<_> = self.workers.lock().unwrap().drain(..).collect();
        for h in handles {
            h.join().unwrap();
        }
    }
}

fn main() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let pool = Pool::new(4);
    let done = Arc::new(AtomicUsize::new(0));
    for _ in 0..16 {
        let d = Arc::clone(&done);
        pool.submit(move || {
            d.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();
    }
    pool.shutdown();
    pool.shutdown(); // 冪等
    assert_eq!(done.load(Ordering::SeqCst), 16);
    assert_eq!(pool.submit(|| {}), Err(Rejected));
    println!("sol_pool_graceful_shutdown: ok");
}
