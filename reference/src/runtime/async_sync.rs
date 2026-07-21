//! # async_sync —— blocking 同步原語的 async 化:AsyncMutex 與 Notify
//!
//! ## [Clarify]
//! 解決:`std::sync::Mutex` 的 `lock()` 會**睡執行緒**——在 async 世界裡
//! 睡執行緒 = 凍住整個 executor worker(所有 task 陪睡,與
//! `server_evented_inline` 同病);`Condvar::wait` 更是天生不相容
//! (它的本質就是 park 執行緒)。要讓「等鎖」「等通知」變成可 `.await`
//! 的操作,睡的單位必須從執行緒改成 task。
//! Constraints:std-only、runtime 無關(純 Waker 協議,不碰 reactor,
//! [`crate::runtime::executor::block_on`] 就能跑)。
//!
//! ## [Abstract]
//! 一個變換打通全部:**blocking 原語 = 睡執行緒的佇列 + park/unpark;
//! async 原語 = 存 `Waker` 的佇列 + `Pending`/`wake`**。
//! 這是 repo「rendezvous 三部曲」的第三章:
//! 1. [`crate::concurrency::thread_pool::JobHandle`]——one-shot,condvar 睡
//! 2. [`crate::io::file_io_offload::JoinFuture`]——one-shot,waker 睡
//! 3. 本模組——可重複使用的原語,waker 睡
//!
//! ## [Iterate]
//! - [`AsyncMutex`]:`Mutex<{locked, waiters}>` 管狀態、`UnsafeCell<T>` 放
//!   資料。lock 的 poll:沒鎖 → 佔住、`Ready(guard)`;鎖著 → 登記 waker、
//!   `Pending`。guard drop:解鎖 + 叫醒隊首。
//! - [`Notify`]:沒有鎖的 condvar。`notify_one` 的 permit 是**飽和的**
//!   (存一張、不累積)——正是 [`crate::runtime::executor`] park/unpark 的 token
//!   語意原封不動搬過來,「notify 先於 await 不丟」。
//! - Condvar 的 predicate-wait 形狀變成:
//!   `loop { if predicate { break } notify.notified().await }`——
//!   「醒來重查」跟 condvar 的 `while` 是同一條契約。
//!
//! ## [Trade-offs]
//! - **內部用 std Mutex 保護 waker 佇列是合法的**:臨界區幾十 ns、絕不跨
//!   `.await`——「async 裡不准用 std Mutex」是訛傳,真正的規則是
//!   **guard 不跨 `.await`**(std guard 掛起時不放,同 worker 的其他 task
//!   要鎖 → 整條 thread 卡死;tokio 的 guard 是 `Send`、std 的不是,
//!   編譯器會擋你一半)。
//! - 什麼時候才需要 AsyncMutex:**guard 要活過 `.await`** 的時候(序列化
//!   一個 IO 資源)。其餘場景 std Mutex(無競爭 ~20ns)更便宜——
//!   async 版每次 contended lock 多付 waker clone + 佇列操作。
//! - 公平性:unlock 叫醒隊首,但新來的 task 可以 **barging**(直接搶到鎖)
//!   ——被叫醒者撲空就重新排隊。吞吐好、可能餓死隊首;tokio 的 Mutex
//!   為此做了 FIFO 交棒。
//! - **誠實邊界(取消)**:`.await` 到一半被 drop 的 LockFuture 不會把
//!   已登記的 waker 撤下——unlock 可能把喚醒「交給」一個已死的等待者,
//!   下一位要等再一次 unlock。production 要在 future 的 Drop 裡轉交喚醒
//!   (tokio 就是);本實作宣告不處理,教學聚焦在 happy path 的協議。
//! - 重複登記:spurious 喚醒後 re-poll 會再 push 一個 waker,佇列裡可能
//!   有同一 task 的多張票——多餘的那張只造成一次 spurious wake,無害。
//!
//! ## [Dry-Run]
//! 測試:guard 跨 await 點、4 執行緒 ×100 次遞增(互斥性 stress)、
//! 交棒(A 持鎖 sleep,B 等到)、permit 先於 await(token 語意)、
//! 跨執行緒 notify、predicate loop。

use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll, Waker};

// ─── AsyncMutex ───────────────────────────────────────────────────

struct LockState {
    locked: bool,
    /// 等鎖的 task 們。「登記」與「查鎖」在同一把 std Mutex 下原子完成
    /// ——lost wakeup(查完沒鎖、登記前對方 unlock)在結構上不可能。
    waiters: VecDeque<Waker>,
}

/// task 級的互斥鎖:`lock().await` 拿 guard,guard 可以活過 `.await`。
pub struct AsyncMutex<T> {
    state: Mutex<LockState>,
    data: UnsafeCell<T>,
}

// SAFETY:AsyncMutex 對 data 的存取由 locked 旗標保證互斥(guard 存在
// ⇔ locked == true,且同時最多一個 guard),語意與 std::sync::Mutex 相同,
// 因此沿用相同的邊界:T: Send 即可跨執行緒共享。
unsafe impl<T: Send> Send for AsyncMutex<T> {}
// SAFETY:同上——&AsyncMutex 只能透過 lock() 拿到 &mut T,互斥已由
// locked 旗標保證。
unsafe impl<T: Send> Sync for AsyncMutex<T> {}

impl<T> AsyncMutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            state: Mutex::new(LockState {
                locked: false,
                waiters: VecDeque::new(),
            }),
            data: UnsafeCell::new(value),
        }
    }

    /// 等到拿到鎖為止。回來的 guard 是 RAII:drop = unlock + 叫醒隊首。
    pub fn lock(&self) -> LockFuture<'_, T> {
        LockFuture { mutex: self }
    }
}

pub struct LockFuture<'a, T> {
    mutex: &'a AsyncMutex<T>,
}

impl<'a, T> Future for LockFuture<'a, T> {
    type Output = AsyncMutexGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut st = self.mutex.state.lock().unwrap();
        if st.locked {
            // 鎖著:登記後睡。被叫醒 ≠ 拿到鎖(可能被 barging),
            // re-poll 時重走這條判斷——「醒來重查」。
            st.waiters.push_back(cx.waker().clone());
            Poll::Pending
        } else {
            st.locked = true;
            Poll::Ready(AsyncMutexGuard { mutex: self.mutex })
        }
    }
}

pub struct AsyncMutexGuard<'a, T> {
    mutex: &'a AsyncMutex<T>,
}

impl<T> Deref for AsyncMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY:guard 存在 ⇔ locked == true,互斥由 lock 協議保證。
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for AsyncMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY:同 deref——同時最多一個 guard,獨占存取成立。
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for AsyncMutexGuard<'_, T> {
    /// unlock + 叫醒隊首(wake one:醒全部是 thundering herd,
    /// 反正只有一個能拿到)。
    fn drop(&mut self) {
        let mut st = self.mutex.state.lock().unwrap();
        st.locked = false;
        if let Some(w) = st.waiters.pop_front() {
            w.wake();
        }
    }
}

// ─── Notify ───────────────────────────────────────────────────────

struct NotifyState {
    /// 飽和的 permit:最多一張、不累積——park/unpark 的 token 語意。
    permit: bool,
    waiters: VecDeque<Waker>,
}

/// 沒有鎖的 condvar:`notified().await` 等一次通知,`notify_one()` 發通知。
/// 「notify 先於 await」不丟(permit 存著);「醒來重查 predicate」由
/// caller 的 loop 負責——與 condvar 的 `while` 同構。
pub struct Notify {
    state: Mutex<NotifyState>,
}

impl Notify {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(NotifyState {
                permit: false,
                waiters: VecDeque::new(),
            }),
        }
    }

    /// 存一張 permit(飽和)並叫醒隊首。沒人在等:permit 留著,
    /// 下一個 `notified().await` 直接穿過——通知不丟。
    pub fn notify_one(&self) {
        let mut st = self.state.lock().unwrap();
        st.permit = true;
        if let Some(w) = st.waiters.pop_front() {
            w.wake();
        }
    }

    /// 等一張 permit。被叫醒後 permit 可能已被別的等待者吃掉
    /// (交給任一等待者的語意)——re-poll 撲空就重新登記。
    pub fn notified(&self) -> Notified<'_> {
        Notified { notify: self }
    }
}

impl Default for Notify {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Notified<'a> {
    notify: &'a Notify,
}

impl Future for Notified<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let mut st = self.notify.state.lock().unwrap();
        if st.permit {
            st.permit = false; // 吃掉這張 permit
            Poll::Ready(())
        } else {
            st.waiters.push_back(cx.waker().clone());
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::executor::{Delay, YieldNow, block_on};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    /// [Dry-Run] guard 跨 await 點——AsyncMutex 存在的唯一理由。
    /// trace:lock → Ready(guard) → 讀值 → YieldNow(Pending、guard 不放)
    /// → 醒來寫回 → drop guard(unlock)→ 再 lock 驗證值。
    /// std Mutex 的 guard 這樣寫直接是 bug(!Send + 掛起不放鎖)。
    #[test]
    fn guard_lives_across_await_point() {
        let m = AsyncMutex::new(10);
        block_on(async {
            let mut g = m.lock().await;
            let read = *g;
            YieldNow::new().await; // guard 活過讓位點
            *g += read;
            drop(g);
            assert_eq!(*m.lock().await, 20);
        });
    }

    /// 互斥性 stress:4 執行緒 × 100 次「讀 → 讓位 → 寫回」遞增。
    /// 若互斥破了,read-modify-write 交錯會丟更新,總數 < 400。
    #[test]
    fn mutual_exclusion_under_contention() {
        let m = Arc::new(AsyncMutex::new(0u64));
        let threads: Vec<_> = (0..4)
            .map(|_| {
                let m = Arc::clone(&m);
                thread::spawn(move || {
                    block_on(async {
                        for _ in 0..100 {
                            let mut g = m.lock().await;
                            let v = *g;
                            YieldNow::new().await; // 把競爭窗撐開
                            *g = v + 1;
                        }
                    })
                })
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
        assert_eq!(*block_on(m.lock()), 400);
    }

    /// 交棒:A 持鎖 80ms,B 的 lock 必須等到 A 放手——量得到的阻塞。
    #[test]
    fn lock_waits_for_holder() {
        let m = Arc::new(AsyncMutex::new(()));
        let m2 = Arc::clone(&m);
        let holder = thread::spawn(move || {
            block_on(async {
                let _g = m2.lock().await;
                Delay::for_duration(Duration::from_millis(80)).await;
            })
        });
        thread::sleep(Duration::from_millis(20)); // 讓 A 先拿到
        let t0 = Instant::now();
        block_on(m.lock());
        assert!(t0.elapsed() >= Duration::from_millis(40), "B 沒等到 A 放鎖");
        holder.join().unwrap();
    }

    /// boundary:permit 先於 await——token 語意,「通知先到」不丟。
    /// (park/unpark 的 wake-before-park 同一張考卷。)
    #[test]
    fn boundary_notify_before_await_is_not_lost() {
        let n = Notify::new();
        n.notify_one();
        n.notify_one(); // 飽和:仍只有一張
        block_on(n.notified()); // 直接穿過
    }

    /// 跨執行緒 notify:等待者真的睡著,80ms 後被另一條執行緒叫醒。
    #[test]
    fn notified_wakes_across_threads() {
        let n = Arc::new(Notify::new());
        let n2 = Arc::clone(&n);
        let t0 = Instant::now();
        let notifier = thread::spawn(move || {
            thread::sleep(Duration::from_millis(80));
            n2.notify_one();
        });
        block_on(n.notified());
        assert!(t0.elapsed() >= Duration::from_millis(40));
        notifier.join().unwrap();
    }

    /// predicate loop:condvar 的 `while (!pred) wait` 換裝——
    /// 醒來重查,flag 沒立起來就繼續等。
    #[test]
    fn predicate_loop_with_notify() {
        let n = Arc::new(Notify::new());
        let flag = Arc::new(AtomicBool::new(false));
        let (n2, f2) = (Arc::clone(&n), Arc::clone(&flag));
        let producer = thread::spawn(move || {
            thread::sleep(Duration::from_millis(30));
            n2.notify_one(); // 假通知:flag 還沒立(考驗醒來重查)
            thread::sleep(Duration::from_millis(30));
            f2.store(true, Ordering::Release);
            n2.notify_one();
        });
        block_on(async {
            while !flag.load(Ordering::Acquire) {
                n.notified().await;
            }
        });
        producer.join().unwrap();
    }
}
