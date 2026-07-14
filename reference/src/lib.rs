//! # reference —— 完整解答層
//!
//! 每個模組都是可跑、有測試、有完整教學註解的實作。
//! 用途:建 mental model、查 API、寫完 challenges 之後 diff 對答案。
//!
//! 閱讀順序見 repo 根目錄 README 的「學習路徑」。
//! 每個模組頂端的 `//!` doc 依 5 pillars 結構撰寫:
//! [Clarify] → [Abstract] → [Iterate] → [Trade-offs] → [Dry-Run]。

// stage 2:mutex/condvar 基礎
pub mod bounded_queue;
pub mod sharded_map;
pub mod thread_pool;

// stage 3:單執行緒資料結構(index-based 優先)
pub mod dsu;
pub mod graph;
pub mod lru;
pub mod ring_buffer;
pub mod tree;
pub mod trie;
