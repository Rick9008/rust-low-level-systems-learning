//! drill:async_sync —— 填 AsyncMutex 的 lock/unlock 與 Notify 的兩端。
//!
//! 已給:結構、new、Deref/DerefMut(含 unsafe,不用你寫)、Send/Sync。
//! 要填:`LockFuture::poll`、`AsyncMutexGuard::drop`、
//! `Notify::notify_one`、`Notified::poll`。
//!
//! 一句話抓住全部:**blocking 原語 = 睡執行緒的佇列 + park/unpark;
//! async 原語 = 存 Waker 的佇列 + Pending/wake**。
//! 「登記」與「查狀態」要在同一把 std Mutex 下原子完成,lost wakeup
//! 才在結構上不可能。設計取捨見 `docs/async/async_sync.md`。

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
    waiters: VecDeque<Waker>,
}

pub struct AsyncMutex<T> {
    state: Mutex<LockState>,
    data: UnsafeCell<T>,
}

// SAFETY:data 的存取由 locked 旗標保證互斥(同時最多一個 guard),
// 語意同 std::sync::Mutex,沿用 T: Send 的邊界。
unsafe impl<T: Send> Send for AsyncMutex<T> {}
// SAFETY:同上——&AsyncMutex 只能經 lock() 拿到 &mut T。
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

    pub fn lock(&self) -> LockFuture<'_, T> {
        LockFuture { mutex: self }
    }
}

pub struct LockFuture<'a, T> {
    mutex: &'a AsyncMutex<T>,
}

impl<'a, T> Future for LockFuture<'a, T> {
    type Output = AsyncMutexGuard<'a, T>;

    /// spec:鎖 self.mutex.state。
    /// - `locked == false` → 設 true,`Ready(AsyncMutexGuard { mutex: self.mutex })`
    /// - `locked == true` → `cx.waker().clone()` push 進 waiters,`Pending`
    ///   (被叫醒 ≠ 拿到鎖——可能被 barging,re-poll 重走判斷)
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!("spec: 查 locked; 沒鎖佔住 Ready(guard); 鎖著登記 waker Pending")
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
        // SAFETY:同 deref。
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for AsyncMutexGuard<'_, T> {
    /// spec:unlock + 交棒。
    /// 鎖 state → `locked = false` → waiters `pop_front()` 有人就 `wake()`
    /// (wake one 即可——醒全部是 thundering herd)。
    fn drop(&mut self) {
        todo!("spec: locked=false; pop_front 有人就 wake")
    }
}

// ─── Notify ───────────────────────────────────────────────────────

struct NotifyState {
    /// 飽和的 permit:最多一張、不累積(park/unpark 的 token 語意)。
    permit: bool,
    waiters: VecDeque<Waker>,
}

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

    /// spec:`permit = true`(飽和,不管原值)+ waiters `pop_front()`
    /// 有人就 wake。沒人在等 → permit 留著,「通知先到」不丟。
    pub fn notify_one(&self) {
        todo!("spec: permit=true; pop_front 有人就 wake")
    }

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

    /// spec:有 permit → 吃掉(設 false)→ `Ready(())`;
    /// 沒有 → 登記 waker → `Pending`。
    /// (被叫醒後 permit 可能已被別人吃掉——re-poll 撲空就重新登記。)
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        todo!("spec: 有 permit 吃掉 Ready; 沒有登記 waker Pending")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reference::runtime::executor::{YieldNow, block_on};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::time::Duration;

    /// guard 跨 await 點——AsyncMutex 存在的唯一理由。
    #[test]
    #[ignore = "填完 LockFuture::poll / Guard::drop 後移除"]
    fn guard_lives_across_await_point() {
        let m = AsyncMutex::new(10);
        block_on(async {
            let mut g = m.lock().await;
            let read = *g;
            YieldNow::new().await;
            *g += read;
            drop(g);
            assert_eq!(*m.lock().await, 20);
        });
    }

    /// 互斥性 stress:交錯的 read-modify-write 不掉更新。
    #[test]
    #[ignore = "填完 LockFuture::poll / Guard::drop 後移除"]
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
                            YieldNow::new().await;
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

    /// boundary:permit 先於 await 不丟;飽和不累積。
    #[test]
    #[ignore = "填完 notify_one / Notified::poll 後移除"]
    fn notify_before_await_is_not_lost() {
        let n = Notify::new();
        n.notify_one();
        n.notify_one();
        block_on(n.notified());
    }

    /// predicate loop:假通知(flag 沒立)不能放行——醒來重查。
    #[test]
    #[ignore = "填完 notify_one / Notified::poll 後移除"]
    fn predicate_loop_survives_spurious_notify() {
        let n = Arc::new(Notify::new());
        let flag = Arc::new(AtomicBool::new(false));
        let (n2, f2) = (Arc::clone(&n), Arc::clone(&flag));
        let producer = thread::spawn(move || {
            thread::sleep(Duration::from_millis(30));
            n2.notify_one(); // 假通知
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
