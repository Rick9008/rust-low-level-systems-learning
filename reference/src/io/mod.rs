//! # io —— event loop 與軟硬體橋接(stage 6–7)
//!
//! 對應 `docs/io/`。epoll 綁定(Linux-only)→ interest table → event loop
//! → offload / echo server → binary protocol 橋接。

// event loop 的 interest table 基底(std-only、平台無關):generational slot map
pub mod fd_registry;

// stage 6:event loop / IO(Linux-only:epoll)
pub mod epoll_sys;
pub mod event_loop;
pub mod file_io_offload;
pub mod tcp_echo;

// stage 7:橋接軟硬體(binary protocol + framing + 雙 server)
pub mod hw_bridge;
