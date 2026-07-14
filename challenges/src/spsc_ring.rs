//! ★ challenge:SPSC lock-free ring buffer
//!
//! 【題目】實作單生產者單消費者的無鎖環形佇列:一條執行緒 push、
//! 另一條 pop,不使用任何鎖(Mutex/Condvar/channel 都不行),
//! 只用 atomic 操作。
//!
//! 【constraints】
//! - std-only;push/pop 均攤 O(1)、無 syscall
//! - 容量固定(建構時決定,可向上調整到方便的數字)
//! - 滿時 push 回 Err 歸還元素;空時 pop 回 None(不阻塞)
//! - 「單生產者單消費者」必須由**型別系統**強制,不是靠註解
//!
//! 【clarify points——動手前先自答】
//! - 兩個 index 各由誰寫?讀對方的 index 時需要什麼 memory ordering?為什麼?
//! - 滿與空怎麼區分?你的 index 表示法在 usize 溢位時還對嗎?
//! - 元素跨執行緒移交,槽位用什麼型別存?`Vec<T>` 直接存為什麼不行?
//! - 帶著未消費元素被 drop 時,誰負責 drop 它們?
//!
//! 【要實作】下方三個簽名。struct 內部完全自己設計。
//! 【驗收】tests/spsc_ring.rs 轉綠(含 10 萬元素的雙執行緒順序測試),
//! 然後跑 reference 的 loom 測試對照你的 ordering 選擇。

use std::marker::PhantomData;

pub struct Producer<T> {
    // ↓ 佔位:讓空殼能編譯。動手時整個換成你的設計。
    _todo: PhantomData<T>,
}

pub struct Consumer<T> {
    _todo: PhantomData<T>,
}

/// 建立容量至少為 `cap` 的 SPSC channel。
pub fn channel<T: Send>(cap: usize) -> (Producer<T>, Consumer<T>) {
    todo!("challenge: 從空白開始")
}

impl<T: Send> Producer<T> {
    /// 無鎖 push;滿時 Err(item) 歸還。
    pub fn push(&mut self, item: T) -> Result<(), T> {
        todo!("challenge")
    }
}

impl<T: Send> Consumer<T> {
    /// 無鎖 pop;空時 None。
    pub fn pop(&mut self) -> Option<T> {
        todo!("challenge")
    }
}
