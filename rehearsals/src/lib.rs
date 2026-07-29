//! rehearsals —— 計時彩排題(模擬 CoderPad 條件)。
//!
//! 規則與題目敘述見本 crate 的 README.md。每題:
//! - `src/<name>.rs`:只有 API 簽名,實作與**你自己的測試**都寫在那個檔案裡(單檔模擬)。
//! - `tests/<name>_test.rs`:參考測試(`#[ignore]`),彩排完才開,對照你漏了哪些邊界。

// 骨架期參數必然未使用(todo!());與 challenges crate 同款 allow。
#![allow(unused_variables, dead_code)]

pub mod bounded_channel;
pub mod event_registry;
pub mod fd_registry;
pub mod frame_parser_heartbeat;
pub mod pool_graceful_shutdown;
pub mod ring_drop_oldest;
pub mod telemetry_aggregator;
pub mod timer_queue;
pub mod tokio_frame_server;

// sim 系列:onsite spec-heavy 模擬題(題幹 docs/interviews/sim-problems.md;
// mock/oracle 直接在各檔內,Phase 2 由面試官手冊控制)。
pub mod sim_i_dma;
pub mod sim_j_isr;
pub mod sim_k_fanin;
pub mod sim_l_mmio;
pub mod sim_m_watchdog;
