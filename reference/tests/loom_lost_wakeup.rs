//! loom 窮舉驗證:condvar lost-wakeup(b#1 洞⑤的最小重現)。
//!
//! 問題形狀:waiter 拿著鎖「檢查條件 → 睡」;通知方「改條件(atomic)→ notify」。
//! 通知方**完全不碰鎖**時,存在一條交錯:waiter 檢查完(還沒睡著)、
//! store+notify 整段插進來、notify 落空、waiter 才睡——永遠沒有第二聲。
//! 這個窗在真機上只有奈秒寬,一般測試永遠綠;loom 不等時機,直接窮舉所有交錯。
//!
//! 三條測試 = 同一個 worker、三種通知側擺法:
//! - 完全不拿鎖          → loom 找到死鎖(#[ignore] 封存,手動跑觀賞)
//! - store 拿鎖(教科書) → 全交錯存活
//! - notify 拿鎖(b#1 實際採用)→ 全交錯存活——兩種擺法都關窗,擋點不同

use loom::sync::atomic::{AtomicBool, Ordering};
use loom::sync::{Arc, Condvar, Mutex};
use loom::thread;

#[derive(Clone, Copy)]
enum ShutdownSide {
    /// 洞⑤:store 與 notify 都不拿鎖——可精準插進「檢查完、還沒睡著」的窗。
    NoLock,
    /// 教科書解:store 在鎖內。waiter「檢查→睡」期間,通知方連牌都寫不了。
    StoreUnderLock,
    /// b#1 實際解:store 在鎖外、notify 在鎖內。牌隨時可寫,但「按鈴」要排隊——
    /// waiter 檢查→睡著期間按不了鈴;醒來重拿鎖時,mutex 的 release/acquire
    /// 順便把鎖外那筆 store 發布過來,re-check 必看到 true。
    NotifyUnderLock,
}

fn model(side: ShutdownSide) {
    loom::model(move || {
        let jobs = Arc::new(Mutex::new(()));
        let cv = Arc::new(Condvar::new());
        let stop = Arc::new(AtomicBool::new(false));

        let worker = {
            let (jobs, cv, stop) = (Arc::clone(&jobs), Arc::clone(&cv), Arc::clone(&stop));
            thread::spawn(move || {
                // 對應 pool 的 worker:拿鎖 → 檢查 predicate → 不成立就 wait。
                let mut guard = jobs.lock().unwrap();
                while !stop.load(Ordering::Acquire) {
                    guard = cv.wait(guard).unwrap();
                }
                drop(guard);
            })
        };

        match side {
            ShutdownSide::NoLock => {
                stop.store(true, Ordering::Release);
                cv.notify_all();
            }
            ShutdownSide::StoreUnderLock => {
                {
                    let _guard = jobs.lock().unwrap();
                    stop.store(true, Ordering::Release);
                }
                cv.notify_all();
            }
            ShutdownSide::NotifyUnderLock => {
                stop.store(true, Ordering::Release);
                {
                    let _guard = jobs.lock().unwrap();
                    cv.notify_all();
                }
            }
        }

        worker.join().unwrap();
    });
}

/// 壞版:loom 必然找到「notify 落空 → worker 睡死 → join 永遠等不到」的交錯,
/// 訊息長這樣:`deadlock; threads = [(Id(0), Blocked), (Id(1), Blocked)]`。
/// loom 的死鎖 panic 會在清理期二次 panic 而 SIGABRT(炸掉整個測試行程),
/// 無法用 `#[should_panic]` 收編——所以掛 ignore,想看它抓人就手動跑:
/// `cargo test -p reference --test loom_lost_wakeup -- --ignored`(預期 abort = bug 被抓到)。
#[test]
#[ignore = "示範用:loom 抓到死鎖會 abort 整個行程,手動跑觀賞"]
fn lost_wakeup_when_no_lock_at_all() {
    model(ShutdownSide::NoLock);
}

/// 教科書解:store 搬進鎖裡,loom 窮舉所有交錯全數存活。
#[test]
fn no_lost_wakeup_when_store_under_lock() {
    model(ShutdownSide::StoreUnderLock);
}

/// b#1 實際採用的解:notify 拿鎖發。loom 同樣全數存活——
/// 關窗的本質是「檢查→睡的那段,通知方的關鍵動作進不來」,
/// 擋 store 或擋 notify 都成立。
#[test]
fn no_lost_wakeup_when_notify_under_lock() {
    model(ShutdownSide::NotifyUnderLock);
}
