//! # executor —— mini block_on:async internals from scratch
//!
//! ## [Clarify]
//! 解決:不依賴 tokio,把 `Future` 從「語法糖」還原成「狀態機 + 輪詢協定」:
//! - `block_on(fut)`:在當前執行緒把一個 future 跑到完成
//! - `Delay`:一個真的會 `Pending` 再被喚醒的 leaf future(async 版 sleep)
//! - `YieldNow`:喚醒發生在 park **之前**的極端路徑(token 語意的試金石)
//!
//! Constraints:std-only;單執行緒 executor(不做任務佇列/多工,
//! 那是 production runtime 的事);Waker 用 `std::task::Wake` trait + `Arc`。
//!
//! ## [Abstract]
//! timer 用「每個 Delay 一條 thread」stub 掉(面試聲明:production 是
//! timer wheel / 時間堆,一條 thread 管全部 deadline——先 spawn 往前走)。
//!
//! ## [Trade-offs]
//! - **Waker = Thread + unpark**:`std::task::Wake` 要求 `Arc<Self>`,
//!   因為 Waker 是 `Clone + Send` 的——它會被塞進任意執行緒(timer、IO)
//!   在未來某刻呼叫;Arc 是最便宜的共享所有權。
//! - **park token 語意(整個 executor 正確性的支點)**:`unpark` 存一個
//!   permit(飽和,不累積);`park` 有 permit 就消耗掉立刻返回,否則睡。
//!   所以「poll 回 Pending → 還沒 park,wake 先到了」不會丟:
//!   permit 已放著,隨後的 park 直接穿過。若用 condvar 裸寫(沒有 predicate)
//!   這裡就是經典 lost-wakeup 死鎖。
//! - **虛假喚醒是協定的一部分**:park 允許 spurious return,所以 loop 裡
//!   醒來一律 re-poll,由 future 自己判斷「真的好了嗎」——與 condvar 的
//!   predicate-wait 同構(見 bounded_queue)。
//! - poll 迴圈時間 O(喚醒次數 × 單次 poll);空間 O(1)(future 釘在 stack 上,
//!   `std::pin::pin!` 零配置——對照 `Box::pin` 的 heap 版)。
//!
//! ## [Dry-Run]
//! 測試:立即 Ready(0 次 park)、Delay 真的等到(跨執行緒喚醒)、
//! wake-先於-park(YieldNow 與「spawn 立刻 wake」兩種)、順序組合。
//!
//! Production 對照:tokio(多執行緒 + IO/timer driver)、
//! futures::executor::block_on(同思路的工業版)、smol。

use std::future::Future;
use std::pin::pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};
use std::time::{Duration, Instant};

/// Waker 的實體:喚醒 = unpark 目標執行緒。
struct ThreadWaker {
    /// `Thread` 是廉價的 handle(內部 Arc),可跨執行緒 unpark。
    thread: Thread,
}

impl Wake for ThreadWaker {
    /// 任何執行緒都可能呼叫(timer thread、IO thread……)。
    /// unpark 是飽和的 permit:目標未 park → 存起來;已 park → 叫醒。
    fn wake(self: Arc<Self>) {
        self.thread.unpark();
    }

    // wake_by_ref 用預設實作(clone + wake)也對;覆寫省一次 Arc bump。
    fn wake_by_ref(self: &Arc<Self>) {
        self.thread.unpark();
    }
}

/// 把 future 跑到完成:async 世界與同步世界的橋。
///
/// 協定回顧:poll 回 `Pending` 的 future **保證**已把 `cx.waker()` 留給
/// 某個之後會叫它的人(timer、IO reactor…)——所以我們敢睡;
/// `Ready` 前的最後一次喚醒一定會把我們從 park 拉起來 re-poll。
pub fn block_on<F: Future>(fut: F) -> F::Output {
    // pin!:future 是自引用狀態機(await 點跨越的借用指向自身),
    // 一旦開始 poll 就不准再搬家;釘在本函式的 stack frame 上,零 heap。
    let mut fut = pin!(fut);
    let waker = Waker::from(Arc::new(ThreadWaker {
        thread: thread::current(),
    }));
    let mut cx = Context::from_waker(&waker);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(out) => return out,
            // 時序陷阱:走到 park 前 wake 可能已發生。
            // park 的 token 語意救了我們:unpark 先到 ⇒ permit 已放,
            // park 立刻返回。沒有這個語意(裸 condvar wait)就是 lost wakeup。
            Poll::Pending => thread::park(),
        }
        // 醒來不代表完成(park 允許虛假喚醒;或有人多叫了一次)——
        // 一律 re-poll,讓 future 自己說話。與 condvar 的 while-predicate 同構。
    }
}

/// async 版 sleep:第一個「真的會 Pending」的 leaf future。
pub struct Delay {
    deadline: Instant,
    /// 首次 poll 時才 spawn timer thread(lazy:future 不 poll 不做事)。
    /// Mutex<Option<Waker>> 讓後續 poll 能換上最新的 waker
    /// (poll 契約:只保證「最後一次 poll 的 waker」會被叫)。
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
}

impl Future for Delay {
    type Output = ();

    /// Delay 沒有自引用欄位 ⇒ 自動 `Unpin`,`get_mut` 安全取 &mut。
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let this = self.get_mut();
        if Instant::now() >= this.deadline {
            return Poll::Ready(());
        }
        match &this.waker_slot {
            Some(slot) => {
                // 不是第一次:更新成最新的 waker。
                // 場景:future 被搬去別的 task/executor,舊 waker 叫錯人。
                *slot.lock().unwrap() = Some(cx.waker().clone());
            }
            None => {
                let slot = Arc::new(Mutex::new(Some(cx.waker().clone())));
                let timer_slot = Arc::clone(&slot);
                let deadline = this.deadline;
                // [Abstract] production 是 timer wheel(一條 thread 管所有
                // deadline,O(1) 插入);這裡 thread-per-delay stub 掉往前走。
                thread::spawn(move || {
                    let now = Instant::now();
                    if deadline > now {
                        thread::sleep(deadline - now);
                    }
                    // take:恰好喚醒一次;之後的 poll 已 Ready 不會再放 waker。
                    if let Some(w) = timer_slot.lock().unwrap().take() {
                        w.wake();
                    }
                });
                this.waker_slot = Some(slot);
            }
        }
        Poll::Pending
    }
}

/// 讓出一次:第一次 poll 先 wake 再回 Pending,第二次 Ready。
/// 這是「wake 發生在 park 之前」的最純測試——executor 的 loop 若寫成
/// 「Pending 就睡、相信一定有人晚點叫」以外的任何花樣都會在這裡死鎖或空轉。
pub struct YieldNow {
    yielded: bool,
}

impl YieldNow {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { yielded: false }
    }
}

impl Future for YieldNow {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let this = self.get_mut();
        if this.yielded {
            Poll::Ready(())
        } else {
            this.yielded = true;
            // 先自我喚醒、再回 Pending:executor 還沒 park,permit 先掛上。
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [Dry-Run] 立即 Ready:poll 一次直接返回,park 次數 0。
    /// boundary:最短路徑(無 Pending、無喚醒)。
    #[test]
    fn boundary_ready_immediately_no_park() {
        assert_eq!(block_on(async { 42 }), 42);
    }

    /// Delay 真的等:trace——
    ///   poll#1:now < deadline → spawn timer、放 waker → Pending → park
    ///   (timer thread)sleep 到期 → take waker → wake → unpark
    ///   poll#2:now >= deadline → Ready
    /// boundary:跨執行緒喚醒(waker 被送到另一條 thread 上呼叫)。
    #[test]
    fn delay_waits_and_wakes_across_threads() {
        let start = Instant::now();
        block_on(Delay::for_duration(Duration::from_millis(30)));
        assert!(start.elapsed() >= Duration::from_millis(30));
    }

    /// boundary:deadline 已過的 Delay——第一次 poll 就 Ready,不 spawn timer。
    #[test]
    fn boundary_delay_already_expired() {
        let start = Instant::now();
        block_on(Delay::until(Instant::now() - Duration::from_millis(10)));
        assert!(start.elapsed() < Duration::from_millis(20)); // 沒有真的睡
    }

    /// boundary:wake 先於 park(YieldNow 在返回 Pending 前就 wake)。
    /// park 的 permit 語意讓 block_on 不死鎖;連續 yield 3 次都要活著回來。
    #[test]
    fn boundary_wake_before_park_via_yield() {
        block_on(async {
            YieldNow::new().await;
            YieldNow::new().await;
            YieldNow::new().await;
        });
    }

    /// boundary:wake 先於 park 的另一形態——poll 期間另一條執行緒
    /// 「立刻」wake(可能趕在 main park 之前)。兩種時序都必須正確:
    ///   wake 先到 → permit 掛上,park 穿過;park 先睡 → unpark 叫醒。
    #[test]
    fn boundary_racing_wake_from_other_thread() {
        struct WakeAsap {
            spawned: bool,
        }
        impl Future for WakeAsap {
            type Output = ();
            fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                let this = self.get_mut();
                if this.spawned {
                    return Poll::Ready(());
                }
                this.spawned = true;
                let waker = cx.waker().clone();
                thread::spawn(move || waker.wake()); // 不睡,馬上叫
                Poll::Pending
            }
        }
        block_on(WakeAsap { spawned: false });
    }

    /// 組合:兩個 Delay 順序 await,總時長 ≈ 相加(狀態機串接正確)。
    #[test]
    fn sequential_delays_compose() {
        let start = Instant::now();
        let out = block_on(async {
            Delay::for_duration(Duration::from_millis(15)).await;
            Delay::for_duration(Duration::from_millis(15)).await;
            "done"
        });
        assert_eq!(out, "done");
        assert!(start.elapsed() >= Duration::from_millis(30));
    }
}
