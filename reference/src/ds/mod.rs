//! # ds —— 單執行緒基礎資料結構(index-based 優先)
//!
//! stage 3 的六個模組集中於此:皆為單執行緒、std-only,
//! 偏好「索引進 `Vec` arena」而非 `Rc<RefCell>` 指標圖
//! (兩種寫法的完整對照見 [`tree`] 的兩版並列)。
//!
//! 並發資料結構(`spsc_ring`、`arena_lockfree`)刻意不收進來:
//! 它們的重點是 memory ordering 與 loom 驗證,留在 crate 根與 stage 4 對應。

pub mod aggregation_tree;
pub mod boot_planner;
pub mod dsu;
pub mod graph;
pub mod lru;
pub mod ring_buffer;
pub mod route_planner;
pub mod tree;
pub mod trie;
