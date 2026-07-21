//! drill:file_io_offload —— 填 future 端的交棒邏輯。
//!
//! 已給:JoinState、spawn_blocking 的 worker 端(放結果 + 喚醒)。
//! 要填:`JoinFuture::poll`——「等待者先到」與「worker 先到」兩種時序都要對。
//! 底層用 reference 的 thread_pool 與 executor(這裡只練交棒)。

use reference::concurrency::thread_pool::ThreadPool;
use reference::runtime::executor::block_on;
use std::future::Future;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

struct JoinState<T> {
    result: Option<std::thread::Result<T>>,
    waker: Option<Waker>,
}

pub struct JoinFuture<T> {
    state: Arc<Mutex<JoinState<T>>>,
}

impl<T> Future for JoinFuture<T> {
    type Output = T;

    /// spec:
    /// - 鎖住 state,`result.take()`:
    ///   - Some(Ok(v)) → Ready(v)
    ///   - Some(Err(panic)) → `resume_unwind(panic)`(worker 的 panic
    ///     在等待端重拋,不 hang、不靜默)
    ///   - None → 把 `cx.waker().clone()` 存進 state.waker → Pending
    ///     (契約:最後一次 poll 的 waker 才有效——每次 poll 都要更新)
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        todo!("spec: take result 三分支;None 記 waker 回 Pending")
    }
}

/// 已給:worker 端——執行 f(panic 也接住)、放結果、若有人在等就喚醒。
pub fn spawn_blocking<T, F>(pool: &ThreadPool, f: F) -> JoinFuture<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let state = Arc::new(Mutex::new(JoinState {
        result: None,
        waker: None,
    }));
    let worker_state = Arc::clone(&state);
    pool.execute(move || {
        let result = catch_unwind(AssertUnwindSafe(f));
        let mut st = worker_state.lock().unwrap();
        st.result = Some(result);
        if let Some(w) = st.waker.take() {
            w.wake();
        }
    });
    JoinFuture { state }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// boundary:等待者先到(poll 先 Pending、worker 完成後喚醒)。
    #[test]
    #[ignore = "填完 poll 後移除"]
    fn waiter_first() {
        let pool = ThreadPool::new(1);
        let fut = spawn_blocking(&pool, || {
            std::thread::sleep(Duration::from_millis(50));
            21 * 2
        });
        assert_eq!(block_on(fut), 42);
    }

    /// boundary:worker 先完成(第一次 poll 直接 Ready,waker 沒用上)。
    #[test]
    #[ignore = "填完 poll 後移除"]
    fn worker_first() {
        let pool = ThreadPool::new(1);
        let fut = spawn_blocking(&pool, || 7);
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(block_on(fut), 7);
    }

    /// boundary:worker panic 在等待端重拋。
    #[test]
    #[ignore = "填完 poll 後移除"]
    #[should_panic(expected = "worker exploded")]
    fn panic_rethrown() {
        let pool = ThreadPool::new(1);
        block_on(spawn_blocking(&pool, || panic!("worker exploded")));
    }
}
