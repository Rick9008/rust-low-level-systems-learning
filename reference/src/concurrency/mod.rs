//! # concurrency —— 執行緒、鎖、atomic / lock-free
//!
//! 對應 `docs/concurrency/`。由淺入深:
//! mutex/condvar(stage 2)→ atomic / lock-free(stage 4)→ 同步策略對照組(stage 4.5)。

// stage 2:mutex/condvar 基礎
pub mod bounded_queue;
pub mod sharded_map;
pub mod thread_pool;

// stage 4:atomic / lock-free(loom 驗證見 tests/loom_*.rs)
pub mod arena_lockfree;
// JD 本尊圖:訊號源 → SPSC → spin-then-park 消費(掛牌握手 = SeqCst 的實戰位)
pub mod signal_pipeline;
pub mod spsc_ring;
// spsc 的兩刀升級:CAS 取號 + per-slot seq(佔位/發布分家)
pub mod mpmc_ring;
// 多生產單消費的 unbounded 連結串列(tokio 遠端 wake queue 同款;含「縫」的顯式 API)
pub mod mpsc_list;
// mpmc_ring 的單消費退化(退化表實體):pop 免 CAS、head 連 atomic 都不是
pub mod mpsc_ring;
// Michael–Scott unbounded MPMC(佔位=發布合一 ⇒ 正式 lock-free;reclamation 攤開講)
pub mod mpmc_list;
// Chase–Lev work-stealing deque(SeqCst fence 的第二個實戰位;tokio/rayon per-worker queue)
pub mod ws_deque;

// stage 4.5:同步策略對照組——同一個資料結構,鎖版/無鎖版並排
// (誰值得無鎖、誰上鎖就夠、無鎖版為何有時不存在:docs/concurrency/ds_sync.md)
pub mod ds_sync;
