/*
Warm-up: Fixed-size thread pool (std only, ~15 min, skeleton to compile — no tests needed)

Implement a basic thread pool:

- ThreadPool::new(n: usize) -> ThreadPool — spawns n worker threads.
- submit(&self, job: F) where F: FnOnce() + Send + 'static — enqueues a job; any idle worker picks it up.
- Graceful shutdown on Drop (or an explicit shutdown(&self)): workers finish all queued jobs, then exit; all threads are joined.

Constraints: std only (Mutex / Condvar / VecDeque / thread), no busy-waiting, no channels.
*/

// ============================================================
// 批改紀錄(Claude,2026-07-23 白天;pool 骨架默寫 rep #1)
// 用時:白紙 22m(目標 15)+ 3 輪修到 0 error
//
// 🔴 主傷疤(7/24 重默的驗收點):
//    「退出 = shutdown ∧ 空」翻成 while 繼續條件時,∧/∨ 連翻錯三次(De Morgan)。
//    第三輪連英文註解(or)都寫對了,code 還是打成 &&。
//    正解推導:while 繼續 = ¬(shutdown ∧ 空) = ¬shutdown ∨ ¬空。
//    處方:高壓下不寫否定式——`loop` + 正面條件 `break`
//    (醒來 pop 到 None ⇒ 空 ∧ shutdown ⇒ break,謂詞已保證)。
//
// ✅ 已固化的舊傷疤(b#1 六洞裡三個,零提示寫對):
//    ⑤ shutdown store 進鎖再 notify(loom 綠變體)
//    ⑥ drop(guard) 再跑 job,跑完再拿鎖(0.40s→0.10s 那課)
//    ④' join().expect 不吞屍
//
// 首編錯誤分類(首波 9 錯,對照 spsc 五類清單):
//    容器/API:Deque→VecDeque、AtomicBool::new() 少 false、wait_while 少 .unwrap()
//    import:只 use JoinHandle 沒 use thread(→ use std::thread::{self, JoinHandle})
//    拼字:job_gaurd/job_guard 混用、submut、self.shard
//    型別語法:欄位 JoinHandle<_>、Result((), …) 圓括號
//    雜項:.len() 當 bool、map 少 .collect()、push_back 少 Box::new、guard 少 mut
//
// 設計備註(面試 trade-off 彈藥):
//    - shutdown 每個讀點都在鎖內/謂詞裡 → bool 直接放進 Mutex 比 AtomicBool
//      混搭更簡單,lost-wakeup 之門從構造上關死(reference 做法)
//    - 修 unused warning 別用 `let _ =`:guard 當場 drop,store 逃出鎖 → ⑤ 回歸
//    - lock-free MPMC ring 換得掉排隊、換不掉睡覺(停車層還是鎖/futex);
//      channel 把兩件事都打包:drop-senders 的 Disconnected 語意 =「空 ∧ 斷線才 Err」,
//      正是退出條件的 API 封裝(std::mpmc 仍 nightly #126840;stable 走 mpsc+Mutex<Receiver>)
//    - submit(&self) vs shutdown(&mut self):borrowck 擋掉並行呼叫,無鎖快查 shutdown 無 TOCTOU
//
// 7/24 開機重默目標(秒殺線):白紙全骨架 10m、首編 ≤3 錯、兩條件一次寫對
// ============================================================

// 11:35
// to do a thread pool we need these things:
// 1. Save the JoinHandle vector
// 2. a Mutex on Job Dequeue
// 3. a atomic bool for shutdown
// 4. Capacity
// 5. a Condvar
//
// we can put Condvar and Mutex of Job Dequeue into a Shared

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

struct Shared {
    jobs: Mutex<VecDeque<Box<dyn FnOnce() + Send + 'static>>>,
    wait_job: Condvar,
    shutdown: AtomicBool,
}

impl Shared {
    fn new() -> Self {
        Self {
            jobs: Mutex::new(VecDeque::new()),
            wait_job: Condvar::new(),
            shutdown: AtomicBool::new(false),
        }
    }
}

struct ThreadPool {
    join_handles: Vec<JoinHandle<()>>,
    shared: Arc<Shared>,
    cap: usize,
}

impl ThreadPool {
    fn new(cap: usize) -> Self {
        assert!(cap >= 1);
        let shared = Arc::new(Shared::new());
        Self {
            join_handles: (0..cap)
                .map(|_| {
                    let cloned = shared.clone();
                    thread::spawn(move || {
                        let mut job_gaurd = cloned.jobs.lock().unwrap();
                        // if there's job or not shutdown, continue loop
                        while !job_gaurd.is_empty() || !cloned.shutdown.load(Ordering::Acquire) {
                            job_gaurd = cloned
                                .wait_job
                                .wait_while(job_gaurd, |s| {
                                    // if there's no jobs and not shutdown, wait
                                    s.is_empty() && !cloned.shutdown.load(Ordering::Acquire)
                                })
                                .unwrap();

                            let job = match job_gaurd.pop_front() {
                                Some(job) => job,
                                None => break,
                            };
                            drop(job_gaurd);
                            job();
                            job_gaurd = cloned.jobs.lock().unwrap();
                        }
                    })
                })
                .collect(),
            shared,
            cap,
        }
    }

    fn shutdown(&mut self) {
        {
            let job_lock = self.shared.jobs.lock().unwrap();
            self.shared.shutdown.store(true, Ordering::Release);
        }
        self.shared.wait_job.notify_all();

        self.join_handles
            .drain(..)
            .for_each(|join_hand| join_hand.join().expect("Join success."));
    }

    fn submut(
        &self,
        job: impl FnOnce() + Send + 'static,
    ) -> Result<(), impl FnOnce() + Send + 'static> {
        if self.shared.shutdown.load(Ordering::Acquire) {
            return Err(job);
        }
        let mut job_gaurd = self.shared.jobs.lock().unwrap();
        job_gaurd.push_back(Box::new(job));
        drop(job_gaurd);
        self.shared.wait_job.notify_one();
        Ok(())
    }
}
