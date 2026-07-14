//! # challenges —— 從頭寫層(模擬面試 live coding)
//!
//! 只給 public API 簽名 + 測試檔。沒有骨架、沒有 helper。
//! 每個模組的 `//!` doc 是面試 prompt 風格:constraints、clarify points、
//! 要實作什麼——但不透露怎麼做。
//!
//! 使用法:
//! 1. 讀模組 doc,當作面試官唸題目。先問自己 clarify points 的答案。
//! 2. 從 public API 簽名開始,整個自己寫。
//! 3. `cargo test -p challenges -- --include-ignored` 轉綠。
//! 4. 寫完 `diff` 對照 `reference/` 的完整解。
