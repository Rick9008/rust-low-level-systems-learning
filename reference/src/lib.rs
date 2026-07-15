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

// 語言慣用法(std-only,面試高頻):邊迭代邊修改 Vec / slice
pub mod iter_mutate;
// 上面 pattern 的高頻 LeetCode in-place 題示範
pub mod inplace_leetcode;

// stage 4:atomic / lock-free(loom 驗證見 tests/loom_*.rs)
pub mod arena_lockfree;
pub mod spsc_ring;
pub(crate) mod sync_shim;

// stage 5:async internals
pub mod executor;

// stage 6:event loop / IO(Linux-only:epoll)
pub mod epoll_sys;
pub mod event_loop;
pub mod file_io_offload;
pub mod tcp_echo;

// stage 7:橋接軟硬體(binary protocol + framing + 雙 server)
pub mod hw_bridge;
