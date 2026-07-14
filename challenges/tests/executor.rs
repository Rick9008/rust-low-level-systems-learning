//! 驗收:challenges::executor。完成後移除 #[ignore]。

use challenges::executor::{Delay, block_on};
use std::future::Future;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

/// boundary:立即 Ready 的 future(不該有任何等待)。
#[test]
#[ignore = "完成 challenge 後移除"]
fn ready_immediately() {
    assert_eq!(block_on(async { 42 }), 42);
}

/// 核心驗收:Delay 真的等到(跨執行緒喚醒)。
#[test]
#[ignore = "完成 challenge 後移除"]
fn delay_waits_full_duration() {
    let start = Instant::now();
    block_on(Delay::for_duration(Duration::from_millis(40)));
    assert!(start.elapsed() >= Duration::from_millis(40));
}

/// boundary:已過期的 Delay 立即 Ready。
#[test]
#[ignore = "完成 challenge 後移除"]
fn expired_delay_instant() {
    let start = Instant::now();
    block_on(Delay::until(Instant::now() - Duration::from_millis(5)));
    assert!(start.elapsed() < Duration::from_millis(20));
}

/// boundary:wake 先於「睡下」——自我喚醒的 future 不得讓 block_on 永眠。
#[test]
#[ignore = "完成 challenge 後移除"]
fn wake_before_sleep_not_lost() {
    struct YieldN(u32);
    impl Future for YieldN {
        type Output = ();
        fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            if self.0 == 0 {
                Poll::Ready(())
            } else {
                self.0 -= 1;
                cx.waker().wake_by_ref(); // 還沒睡就先叫
                Poll::Pending
            }
        }
    }
    block_on(YieldN(3)); // 不 hang 就是過
}

/// 組合:順序 await 兩個 Delay,時長相加;回傳值穿過狀態機。
#[test]
#[ignore = "完成 challenge 後移除"]
fn sequential_composition() {
    let start = Instant::now();
    let out = block_on(async {
        Delay::for_duration(Duration::from_millis(15)).await;
        Delay::for_duration(Duration::from_millis(15)).await;
        "done"
    });
    assert_eq!(out, "done");
    assert!(start.elapsed() >= Duration::from_millis(30));
}
