//! # drills —— 填空層
//!
//! 與 `reference` 相同的模組樹,但核心函式的函式體被挖空成
//! `todo!("spec: ...")`。骨架、helper、資料結構定義都給了,
//! 只填會被面試考的那幾行核心邏輯。
//!
//! 使用法:
//! 1. `cargo test -p drills -- --include-ignored` 看哪些測試紅。
//! 2. 打開對應檔案,讀函式上方的 spec doc comment。
//! 3. 先在紙上 dry-run 測試裡標注的 boundary(空、滿、wrap、overflow)。
//! 4. 填掉 `todo!()`,移除測試上的 `#[ignore]`,轉綠。
