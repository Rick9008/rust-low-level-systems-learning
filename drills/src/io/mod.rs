//! event loop / IO / 軟硬體橋接的填空題(對應 `docs/io/`)。

pub mod endian_pack;
pub mod epoll_sys;
pub mod event_loop;
pub mod fd_registry;
pub mod file_io_offload;
pub mod hw_bridge;
pub mod tcp_echo;

// R2 sim 系列填空版(⚠ 對應計時場跑完前不要開 j–n 的檔)
pub mod dma_dispatcher;
