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

// R2 sim 系列教學版:bus 驅動的 event-loop state machine
// (彩排 harness 在 rehearsals/src/sim_*.rs;⚠ 對應計時場跑完前不要開 j–n 的檔)
pub mod dma_dispatcher;
// SPSC ring 的硬體邊界版:barrier→doorbell 鐵律 + 序號差滿判定(⚠ 8/2 場後開)
pub mod mmio_cmdq;
// dma_dispatcher + 第三種 state「時間」:隔離、zombie 免疫、retry budget(⚠ 8/8 場後開)
pub mod engine_watchdog;
