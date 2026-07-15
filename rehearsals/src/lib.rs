//! rehearsals —— 計時彩排題(模擬 CoderPad 條件)。
//!
//! 規則與題目敘述見本 crate 的 README.md。每題:
//! - `src/<name>.rs`:只有 API 簽名,實作與**你自己的測試**都寫在那個檔案裡(單檔模擬)。
//! - `tests/<name>_test.rs`:參考測試(`#[ignore]`),彩排完才開,對照你漏了哪些邊界。

// 骨架期參數必然未使用(todo!());與 challenges crate 同款 allow。
#![allow(unused_variables, dead_code)]

pub mod frame_parser_heartbeat;
pub mod pool_graceful_shutdown;
pub mod ring_drop_oldest;
pub mod tokio_frame_server;
