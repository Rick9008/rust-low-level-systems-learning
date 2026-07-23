//! ★ challenge:mini executor(block_on + Delay)
//!
//! 【題目】不用任何 async runtime,實作:
//! 1. `block_on(fut)`——在當前執行緒把一個 future 跑到完成
//! 2. `Delay`——async 版 sleep(到期前 Pending、到期後被喚醒並 Ready)
//!
//! 【constraints】
//! - std-only;Waker 用 `std::task::Wake` trait(需要 Arc)
//! - block_on 等待期間不可 busy-spin(CPU 要能睡)
//! - Delay 允許為每個實例 spawn 一條 timer thread(production 的
//!   timer wheel 不要求)
//!
//! 【clarify points——動手前先自答】
//! - poll 回 Pending 後你打算讓執行緒睡在哪?誰、憑什麼把它叫醒?
//! - 喚醒發生在「Pending 返回之後、睡下去之前」——會不會永眠?為什麼?
//! - 醒來就代表 future 完成了嗎?該做什麼?
//! - future 為什麼要 Pin?你打算把它釘在哪(stack / heap)?
//! - Delay 被 poll 多次、每次拿到的 waker 不同——存哪一個?
//!
//! 【要實作】下方簽名。【驗收】tests/executor.rs 轉綠。

use std::future::Future;
use std::sync::Arc;
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

/// 把 future 跑到完成,回傳其輸出。
pub fn block_on<F: Future>(fut: F) -> F::Output {
    // todo!("challenge: 從空白開始")

    let mut pin_fut = std::pin::pin!(fut);
    let waker = Waker::from(Arc::new(ThreadWaker {
        thread: thread::current(),
    }));
    let mut cx = Context::from_waker(&waker);
    loop {
        let res = pin_fut.as_mut().poll(&mut cx);

        match res {
            Poll::Ready(output) => break output,
            Poll::Pending => {
                thread::park();
                continue;
            }
        };
    }
}

/// async 版 sleep。
pub struct Delay {
    // ↓ 佔位:動手時整個換成你的設計。
    // _todo: (),
    deadline: Instant,
}

impl Delay {
    pub fn until(deadline: Instant) -> Self {
        // todo!("challenge")
        Self { deadline }
    }

    pub fn for_duration(d: Duration) -> Self {
        // todo!("challenge")
        Self {
            deadline: Instant::now() + d,
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<()> {
        // todo!("challenge")
        if Instant::now() < self.deadline {
            let waker = cx.waker().clone();
            let sleep_time = self.deadline - Instant::now();
            thread::spawn(move || {
                thread::sleep(sleep_time);
                waker.wake();
            });
            return Poll::Pending;
        }
        Poll::Ready(())
    }
}
