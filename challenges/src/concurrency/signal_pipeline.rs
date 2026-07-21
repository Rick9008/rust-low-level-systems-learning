//! ★ challenge:signal pipeline(訊號源 → SPSC → 消費執行緒)
//!
//! 【題目】硬體訊號從一條執行緒高速湧入(爆發時每秒百萬級),另一條
//! 執行緒聚合統計(count/sum/min/max)。訊號執行緒**不能被擋**
//! (硬體推不回去);沒訊號時消費執行緒不准燒滿 CPU。
//!
//! 【constraints】
//! - std-only。佇列用現成的 `reference::concurrency::spsc_ring`——佇列本身不是本題
//!   考點(那是 spsc_ring challenge),**接線才是**。
//! - 熱路徑(消費端醒著時)零 syscall、零鎖。
//! - 滿了可以丟,但**丟多少必須可觀測**。
//! - shutdown 語意:殘料處理完才回(drain-then-exit)。
//!
//! 【clarify points——動手前先自答】
//! - 滿了丟哪一筆?在 SPSC ring 上,drop-oldest 為什麼做不到?
//! - 消費端沒訊號時怎麼等?每種等法(spin / park / 每筆 unpark)各付
//!   什麼代價?
//! - 掛牌(宣告自己要睡)之後、真的睡之前,為什麼必須再看一眼佇列?
//!   只用 Release/Acquire 夠嗎?
//! - shutdown 訊號跟「佇列空」怎麼區分「暫時沒貨」與「收工」?
//!
//! 【要實作】下方簽名。內部設計(旗標、fence、spin 策略)完全自己來。
//! 【驗收】tests/signal_pipeline.rs 轉綠(含 lost-wakeup 會卡死的測試),
//! 然後 diff reference::concurrency::signal_pipeline 對答案。

use reference::concurrency::spsc_ring::{Consumer, Producer};
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signal {
    pub sensor_id: u16,
    pub value: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub count: u64,
    pub sum: i64,
    pub min: i64,
    pub max: i64,
}

pub struct SignalSender {
    // ↓ 佔位:動手時整個換成你的設計(會用到 Producer<Signal>)。
    _todo: PhantomData<Producer<Signal>>,
}

pub struct PipelineHandle {
    _todo: PhantomData<Consumer<Signal>>,
}

/// 起消費執行緒,回(producer 把手, 控制把手)。
pub fn start(capacity: usize) -> (SignalSender, PipelineHandle) {
    todo!("challenge: 從空白開始")
}

impl SignalSender {
    /// 訊號執行緒呼叫;不准阻塞。滿了回 false 並計數。
    pub fn send(&mut self, s: Signal) -> bool {
        todo!("challenge")
    }

    /// 至今被丟掉的筆數。
    pub fn dropped(&self) -> u64 {
        todo!("challenge")
    }
}

impl PipelineHandle {
    /// 殘料處理完才回。
    pub fn shutdown(self) -> Stats {
        todo!("challenge")
    }
}
