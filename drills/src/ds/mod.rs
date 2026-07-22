//! # ds —— 單執行緒基礎資料結構(填空層)
//!
//! 與 `reference::ds` 同樹:六個模組皆為單執行緒、std-only、index-based 優先。
//! 挖空的位置與使用法見 crate root doc;卡住時 diff `reference/src/ds/` 同名檔。

pub mod dsu;
pub mod graph;
pub mod lru;
pub mod ring_buffer;
pub mod telemetry_aggregator;
pub mod tree;
pub mod trie;
