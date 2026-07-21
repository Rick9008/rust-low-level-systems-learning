//! # runtime —— async internals(stage 5)
//!
//! 對應 `docs/async/`(`async` 是 Rust 保留字,模組名用 `runtime`)。
//! executor(手搓 block_on)→ async 同步原語 → executor × reactor 縫成 mini-tokio。

pub mod executor;
// blocking 同步原語的 async 化(rendezvous 三部曲第三章):AsyncMutex + Notify
pub mod async_sync;
// executor × reactor 縫起來:兩階 reactor(V0 O(n) scan → V1 epoll)的 mini-tokio
pub mod mini_runtime;
