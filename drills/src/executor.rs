//! drill:executor —— 填 block_on 的 poll 迴圈與 Delay 的 poll。
//!
//! 已給:ThreadWaker(Wake impl)、Delay 結構與 timer 啟動 helper。
//! 要填:`block_on` / `Delay::poll`。
//! 填之前紙上回答:wake 發生在「poll 回 Pending 之後、park 之前」,
//! 為什麼不會永眠?(關鍵詞:park 的 permit/token 語意)

use std::future::Future;
use std::pin::pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};
use std::time::{Duration, Instant};

struct ThreadWaker {
    thread: Thread,
}

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.thread.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.thread.unpark();
    }
}

/// spec:把 future 跑到完成。
/// 1. `pin!(fut)` 釘在 stack(future 是自引用狀態機,poll 後不准搬家)
/// 2. 用 `Waker::from(Arc::new(ThreadWaker{ thread: thread::current() }))`
///    造 waker、包成 Context
/// 3. loop:poll → Ready 就回傳;Pending 就 `thread::park()`
/// 4. 醒來**一律 re-poll**(park 允許虛假喚醒;完成與否由 future 說)
pub fn block_on<F: Future>(fut: F) -> F::Output {
    todo!("spec: pin! + waker + loop {{ poll / park }}")
}

/// async sleep。
pub struct Delay {
    deadline: Instant,
    waker_slot: Option<Arc<Mutex<Option<Waker>>>>,
}

impl Delay {
    pub fn until(deadline: Instant) -> Self {
        Self {
            deadline,
            waker_slot: None,
        }
    }

    pub fn for_duration(d: Duration) -> Self {
        Self::until(Instant::now() + d)
    }

    /// helper(已給):spawn timer thread,睡到 deadline 後把 slot 裡的
    /// waker 拿出來(take)叫醒。
    fn spawn_timer(deadline: Instant, slot: Arc<Mutex<Option<Waker>>>) {
        thread::spawn(move || {
            let now = Instant::now();
            if deadline > now {
                thread::sleep(deadline - now);
            }
            if let Some(w) = slot.lock().unwrap().take() {
                w.wake();
            }
        });
    }
}

impl Future for Delay {
    type Output = ();

    /// spec:
    /// 1. 已到 deadline → Ready(())(第一次 poll 就過期也走這裡,不 spawn)
    /// 2. 第一次 poll(waker_slot 是 None)→ 建 slot 放入 cx.waker().clone(),
    ///    spawn_timer,記下 slot → Pending
    /// 3. 之後的 poll → 把 slot 裡的 waker **換成最新的**(poll 契約:
    ///    只保證最後一次 poll 的 waker 會被叫)→ Pending
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let this = self.get_mut(); // Delay 無自引用 ⇒ Unpin,安全取 &mut
        todo!("spec: 到期 Ready;首次 spawn timer;之後更新 waker")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// boundary:立即 Ready(零 park)。
    #[test]
    #[ignore = "填完 block_on 後移除"]
    fn ready_immediately() {
        assert_eq!(block_on(async { 42 }), 42);
    }

    /// boundary:Delay 真的等到(跨執行緒喚醒)。
    #[test]
    #[ignore = "填完 block_on/Delay::poll 後移除"]
    fn delay_waits() {
        let start = Instant::now();
        block_on(Delay::for_duration(Duration::from_millis(30)));
        assert!(start.elapsed() >= Duration::from_millis(30));
    }

    /// boundary:wake 先於 park——自我喚醒的 future 不得讓 block_on 永眠。
    #[test]
    #[ignore = "填完 block_on 後移除"]
    fn wake_before_park_not_lost() {
        struct YieldOnce(bool);
        impl Future for YieldOnce {
            type Output = ();
            fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                if self.0 {
                    Poll::Ready(())
                } else {
                    self.0 = true;
                    cx.waker().wake_by_ref(); // 還沒 park 就先叫
                    Poll::Pending
                }
            }
        }
        block_on(YieldOnce(false)); // 不 hang 就是過
    }

    /// boundary:已過期的 Delay 第一次 poll 就 Ready(不 spawn timer)。
    #[test]
    #[ignore = "填完 block_on/Delay::poll 後移除"]
    fn expired_delay_is_instant() {
        let start = Instant::now();
        block_on(Delay::until(Instant::now() - Duration::from_millis(5)));
        assert!(start.elapsed() < Duration::from_millis(20));
    }
}
