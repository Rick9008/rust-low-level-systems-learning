//! event loop / IO / 軟硬體橋接的填空題(對應 `docs/io/`)。

pub mod epoll_sys;
pub mod event_loop;
pub mod fd_registry;
pub mod file_io_offload;
pub mod hw_bridge;
pub mod tcp_echo;
