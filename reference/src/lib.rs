//! # reference —— 完整解答層
//!
//! 每個模組都是可跑、有測試、有完整教學註解的實作。
//! 用途:建 mental model、查 API、寫完 challenges 之後 diff 對答案。
//!
//! 閱讀順序見 repo 根目錄 README 的「學習路徑」。
//! 每個模組頂端的 `//!` doc 依 5 pillars 結構撰寫:
//! [Clarify] → [Abstract] → [Iterate] → [Trade-offs] → [Dry-Run]。
//!
//! 模組樹鏡射 `docs/` 的四分類:
//! `ds`(單執行緒資料結構)→ `concurrency`(鎖與 lock-free)
//! → `runtime`(async internals)→ `io`(event loop 與橋接)。

// 單執行緒資料結構(stage 3,index-based 優先)
pub mod ds;

// 執行緒、鎖、atomic / lock-free(stage 2 + 4 + 4.5)
pub mod concurrency;

// async internals(stage 5)
pub mod runtime;

// event loop / IO / 軟硬體橋接(stage 6–7)
pub mod io;

// 語言慣用法(std-only,面試高頻):邊迭代邊修改 Vec / slice
pub mod iter_mutate;
// 上面 pattern 的高頻 LeetCode in-place 題示範
pub mod inplace_leetcode;

// loom 機關:lib 端 re-export std 同步原語;loom 測試以 #[path] include
// 同一份 core_impl.rs 並自帶 loom 版 sync_shim(見模組 doc)。留在 crate 根,
// core_impl.rs 的 `crate::sync_shim` 路徑才不隨分類搬動而變。
pub(crate) mod sync_shim;
