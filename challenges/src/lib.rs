//! # challenges —— 從頭寫層(模擬面試 live coding)
//!
//! 只給 public API 簽名 + 測試檔。沒有骨架、沒有 helper。
//! 每個模組的 `//!` doc 是面試 prompt:constraints、clarify points、
//! 要實作什麼——但不透露怎麼做。
//!
//! 使用法:
//! 1. 讀模組 doc,當作面試官唸題目。先回答 clarify points 再動手。
//! 2. 把 struct 裡的佔位欄位(`_todo`)整個換成你的設計,填掉 todo!()。
//! 3. `cargo test -p challenges -- --include-ignored` 轉綠
//!    (轉綠後把測試檔的 `#[ignore]` 移除)。
//! 4. `diff` 對照 `reference/` 的完整解。
//!
//! 建議順序(★ = 先做):★ spsc_ring → ★ signal_pipeline → ★ executor
//! → ★ lru → ★ hw_bridge → dsu → sharded_map → tcp_echo。

// 空殼狀態下簽名的參數/佔位欄位還沒被使用;動手寫時建議先拿掉這行自查。
#![allow(unused_variables, dead_code)]

pub mod ds;
pub mod executor;
pub mod hw_bridge;
pub mod sharded_map;
pub mod signal_pipeline;
pub mod spsc_ring;
pub mod tcp_echo;
