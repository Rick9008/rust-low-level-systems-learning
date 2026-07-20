//! # ds_sync —— 同步策略對照組:同一個資料結構,鎖版 / 無鎖版並排
//!
//! `ds/` 回答「這個結構怎麼寫」;本模組回答「多執行緒要共享它時,
//! 你有哪幾檔選擇、各付什麼」。完整取捨帳(塌縮原理、Mutex vs lock-free
//! 的成本真相、升級階梯 global lock → shard → per-thread → lock-free)
//! 見 docs/concurrency/ds_sync.md。
//!
//! 全 repo 的配對地圖:
//!
//! | 結構 | 單執行緒 | 鎖版 | 無鎖版 |
//! |------|----------|------|--------|
//! | bounded queue | [`crate::ds::ring_buffer`] | [`crate::bounded_queue`] (+condvar) | [`crate::spsc_ring`] |
//! | arena / slab | — | [`arena_locked`] | [`crate::arena_lockfree`] |
//! | union-find | [`crate::ds::dsu`] | `Mutex<Dsu>`(不值得開檔,見下) | [`dsu_lockfree`] |
//! | LRU cache | [`crate::ds::lru`] | [`lru_locked`] (sharded) | **不存在**(get=寫,見 docs) |
//! | sorted list/set | — | [`list_fine`] (hand-over-hand) | Harris list(mark bit,研究級,不實作) |
//! | tree / trie / graph | [`crate::ds`] 各模組 | `Mutex<T>` 或 shard | 研究級(Ctrie 等),docs 講為什麼 |
//!
//! 兩條讀法:
//! - **塌縮方向**(無鎖 → 鎖):對照 `arena_lockfree` 讀 [`arena_locked`]——
//!   gen tag、atomic next、MaybeUninit 整組蒸發,證明複雜度住在同步策略裡。
//! - **解放方向**(鎖 → 無鎖):對照 `Mutex<Dsu>` 讀 [`dsu_lockfree`]——
//!   什麼樣的結構值得無鎖化(寫入單調、熱點會被壓縮攤平),
//!   以及付出的語意稅(connected 變快照、components 變統計值)。
//!   `Mutex<Dsu>` 本身一行包裝就完事,無獨立教學價值,故不開檔。

pub mod arena_locked;
pub mod dsu_lockfree;
pub mod list_fine;
pub mod lru_locked;
