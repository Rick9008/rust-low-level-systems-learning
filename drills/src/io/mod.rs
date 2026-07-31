//! event loop / IO / 軟硬體橋接的填空題(對應 `docs/io/`)。

pub mod endian_pack;
pub mod epoll_sys;
pub mod event_loop;
pub mod fd_registry;
pub mod file_io_offload;
pub mod hw_bridge;
pub mod tcp_echo;

// R2 sim 系列填空版(⚠ m=8/2 場後開;l 填空版是 8/2 lite 場材料,開跑即用;i 已開)
pub mod dma_dispatcher;
pub mod engine_watchdog;
pub mod mmio_cmdq;
