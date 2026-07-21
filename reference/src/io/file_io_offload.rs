//! # file_io_offload —— 檔案 IO 丟到 thread pool,以 future 取回
//!
//! ## [Clarify]
//! 解決:event loop / async 環境裡的 regular file IO。
//! **epoll 為什麼不適用 regular file**:epoll 是 readiness model,而 regular
//! file「永遠 ready」——kernel 不會說「檔案還沒準備好」,它直接在 read(2)
//! 裡做磁碟 IO,阻塞你(即使 fd 設了 O_NONBLOCK,對 regular file 也無效)。
//! `epoll_ctl(ADD)` 對 regular file 乾脆回 EPERM。
//! 所以:網路 fd 走 epoll,檔案 IO 走 **offload**——丟給專用 thread pool 阻塞,
//! 完成後喚醒等待者。tokio 的 `spawn_blocking` 就是這條路。
//! (另一條路是 **io_uring**:completion model,把「讀這個檔案」提交給 kernel,
//! 完成後通知你——真 async file IO,本 repo 聲明不實作。)
//!
//! ## [Abstract]
//! 泛化為 `spawn_blocking(pool, f) -> JoinFuture<T>`:任何阻塞工作都能 offload,
//! 檔案讀寫只是特例(`read_file` helper)。與 stage 5 的 executor 直接可組合。
//!
//! ## [Trade-offs]
//! - 完成通知 = future + waker(而非 callback):與 async 生態同形;
//!   `Mutex<(Option<T>, Option<Waker>)>` 是「一次性 rendezvous」的最小實作。
//!   時間 O(1) 每次 poll;空間 O(1) 每個任務。
//! - worker panic:結果永遠不會到,等待者會 hang——所以 worker 端 catch_unwind,
//!   把 panic 當一種結果送回,`JoinFuture` 在等待端 **resume_unwind** 重拋
//!   (tokio JoinError 的簡化版:panic 不該被靜默吃掉,也不該炸錯執行緒)。
//! - offload 的成本:一次 job 排隊 + 兩次鎖 + 一次喚醒(~μs 級)。
//!   小而多的隨機讀在 io_uring 下會顯著更快;順序大檔兩者差距縮小。
//!
//! ## [Dry-Run]
//! 測試:讀真檔案 roundtrip、與 executor 組合(block_on await)、
//! 並發多任務、worker panic 重拋、先完成後 await(waker 從缺的路徑)。

use crate::concurrency::thread_pool::ThreadPool;
use crate::runtime::executor::block_on;
use std::future::Future;
use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::path::Path;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

/// 一次性交棒:worker 放結果 + 叫醒;等待者輪詢 + 登記 waker。
struct JoinState<T> {
    /// Ok(T) 或 worker 的 panic payload(重拋用)。
    result: Option<std::thread::Result<T>>,
    waker: Option<Waker>,
}

pub struct JoinFuture<T> {
    state: Arc<Mutex<JoinState<T>>>,
}

impl<T> Future for JoinFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut st = self.state.lock().unwrap();
        match st.result.take() {
            Some(Ok(v)) => Poll::Ready(v),
            // worker panic:在等待端重拋——錯誤跟著「在乎它的人」走。
            Some(Err(panic)) => resume_unwind(panic),
            None => {
                // 還沒好:留下最新的 waker(契約:最後一次 poll 的 waker 有效)。
                st.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

/// 把阻塞工作丟進 pool,回一個可 await 的 future。
///
/// 完成路徑:worker 放結果 →(若有人已在等)wake → executor re-poll → Ready。
/// 先完成後 await 的路徑:結果已在,第一次 poll 直接 Ready,waker 全程沒用上。
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
        // AssertUnwindSafe:f 與 result 槽只在此處寫入一次,
        // panic 後不會有人觀察到半初始化狀態。
        let result = catch_unwind(AssertUnwindSafe(f));
        let mut st = worker_state.lock().unwrap();
        st.result = Some(result);
        if let Some(w) = st.waker.take() {
            w.wake(); // 有人在等:叫醒。沒人等:結果放著,await 時直接拿。
        }
    });
    JoinFuture { state }
}

/// 檔案讀取的 offload 特例。
pub fn read_file(pool: &ThreadPool, path: impl AsRef<Path>) -> JoinFuture<io::Result<Vec<u8>>> {
    let path = path.as_ref().to_owned();
    spawn_blocking(pool, move || std::fs::read(path))
}

/// 同步入口:block_on + offload 的一站式(給非 async caller)。
pub fn read_file_blocking(pool: &ThreadPool, path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    block_on(read_file(pool, path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_path(name: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "file_io_offload_test_{}_{}",
            std::process::id(),
            name
        ));
        p
    }

    /// [Dry-Run] 讀真檔案:寫入 → offload 讀 → 內容一致。
    /// trace:spawn_blocking 排 job → block_on poll → Pending(登記 waker)
    /// → park → worker fs::read 完成 → 放結果 → wake → re-poll → Ready。
    #[test]
    fn read_real_file_roundtrip() {
        let path = temp_path("roundtrip");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"offloaded bytes")
            .unwrap();
        let pool = ThreadPool::new(2);
        let content = read_file_blocking(&pool, &path).unwrap();
        assert_eq!(content, b"offloaded bytes");
        std::fs::remove_file(&path).unwrap();
    }

    /// boundary:檔案不存在——io::Error 原樣穿過 offload 邊界。
    #[test]
    fn boundary_missing_file_error_passes_through() {
        let pool = ThreadPool::new(1);
        let err = read_file_blocking(&pool, temp_path("nonexistent")).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    /// 並發:8 個任務同時 offload,await 全部(單一 future 逐個等,
    /// 但 worker 端是並行跑的——4 條 worker)。
    #[test]
    fn many_tasks_concurrently() {
        let pool = ThreadPool::new(4);
        let futures: Vec<_> = (0..8)
            .map(|i| spawn_blocking(&pool, move || i * i))
            .collect();
        let results: Vec<i32> = block_on(async {
            let mut out = Vec::new();
            for f in futures {
                out.push(f.await);
            }
            out
        });
        assert_eq!(results, vec![0, 1, 4, 9, 16, 25, 36, 49]);
    }

    /// boundary:先完成、後 await——waker 從未被登記的路徑
    /// (第一次 poll 就 Ready)。
    #[test]
    fn boundary_completed_before_await() {
        let pool = ThreadPool::new(1);
        let fut = spawn_blocking(&pool, || 7);
        std::thread::sleep(std::time::Duration::from_millis(50)); // 讓 worker 先跑完
        assert_eq!(block_on(fut), 7);
    }

    /// boundary:worker panic → 等待端重拋(不是 hang、不是靜默)。
    #[test]
    #[should_panic(expected = "worker exploded")]
    fn boundary_worker_panic_rethrown_at_await() {
        let pool = ThreadPool::new(1);
        let fut = spawn_blocking(&pool, || panic!("worker exploded"));
        block_on(fut);
    }
}
