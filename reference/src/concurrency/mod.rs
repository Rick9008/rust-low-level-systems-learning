//! # concurrency —— 執行緒、鎖、atomic / lock-free
//!
//! 對應 `docs/concurrency/`。由淺入深:
//! mutex/condvar(stage 2)→ atomic / lock-free(stage 4)→ 同步策略對照組(stage 4.5)。

// stage 2:mutex/condvar 基礎
pub mod bounded_queue;
// per-key conflation(值層/通知層分離;圖解與 stepper 在 html_p/conflation-slot-stepper.html)
pub mod conflation_slot;
pub mod sharded_map;
pub mod thread_pool;

// stage 4:atomic / lock-free(loom 驗證見 tests/loom_*.rs)
pub mod arena_lockfree;
// 開胃菜:atomics 的最小完整應用——TTAS 自旋鎖 + RAII guard
// (Drop/Deref/unsafe impl Sync 的教學模組;7/31 加,drills/challenges 有練習版)
pub mod spin_lock;
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

// 讀多寫少的快照發布(std 版 poor-man's ArcSwap;零 unsafe,光譜「快照」站實體)
pub mod rcu_snapshot;

// stage 4.5:同步策略對照組——同一個資料結構,鎖版/無鎖版並排
// (誰值得無鎖、誰上鎖就夠、無鎖版為何有時不存在:docs/concurrency/ds_sync.md)
pub mod ds_sync;

// R2 sim 系列教學版(⚠ 對應場次跑完前不要開:j=7/31、k=8/1 lite、n=8/3 lite)
// signal_pipeline 的 spec-heavy 版:ISR 三禁 + sticky-flag 喚醒 + shutdown drain
pub mod isr_pipeline;
// 扇入節的 spec-heavy 版:per-core SPSC + budget 公平 + 全空一輪才睡
pub mod percpu_fanin;
// 兩層閘門:DAG indegree 入場閘 + BinaryHeap 優先權閘(Kahn 的事件驅動版)
pub mod job_scheduler;
