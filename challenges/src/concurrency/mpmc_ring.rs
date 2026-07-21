//! ★ challenge:MPMC lock-free bounded queue
//!
//! 【題目】實作多生產者多消費者的無鎖固定容量佇列:任意執行緒 push、
//! 任意執行緒 pop,不使用任何鎖,只用 atomic 操作。
//! (這是 spsc_ring 的後續題:面試官在你寫完 SPSC 後說
//! "now make it work with multiple producers and consumers"。)
//!
//! 【constraints】
//! - std-only;try_push/try_pop 均攤 O(1)、無 syscall、無全域鎖
//! - 容量固定(建構時決定,可向上調整到方便的數字)
//! - 滿時 push 回 Err 歸還元素;空時 pop 回 None(不阻塞)
//! - 任意端點數:同一個佇列物件經 `Arc` 共享即可,兩端方法都拿 `&self`
//!
//! 【clarify points——動手前先自答】
//! - SPSC 的「先寫槽位、再 Release store tail」在多 producer 下哪一步先壞?
//! - 「搶到寫的位置」和「資料寫完可讀」還是同一個事件嗎?不是的話,
//!   consumer 憑什麼知道一個槽位可以讀?
//! - 你的滿/空判斷在兩個 counter 都被多執行緒推進時還成立嗎?
//! - FIFO 對 MPMC 來說是什麼意思?(誰跟誰之間保序?)
//! - pop 回 None 的時刻,佇列裡可能有元素嗎?(想想一個 producer
//!   正好被 OS deschedule 在哪裡。)
//!
//! 【要實作】下方簽名。struct 內部完全自己設計。
//! 【驗收】tests/mpmc_ring.rs 轉綠(含 2P2C 壓力測試),
//! 然後跑 `cargo test -p reference --test loom_mpmc` 對照 ordering 選擇。

use std::marker::PhantomData;

pub struct MpmcRing<T> {
    // ↓ 佔位:讓空殼能編譯。動手時整個換成你的設計。
    _todo: PhantomData<T>,
}

impl<T: Send> MpmcRing<T> {
    /// 建立容量至少為 `cap` 的 MPMC queue(可向上調整)。
    pub fn new(cap: usize) -> Self {
        let _ = cap;
        todo!("challenge: 從空白開始")
    }

    /// 無鎖 push;滿時 Err(item) 歸還。
    pub fn try_push(&self, item: T) -> Result<(), T> {
        let _ = item;
        todo!("challenge")
    }

    /// 無鎖 pop;空時 None。
    pub fn try_pop(&self) -> Option<T> {
        todo!("challenge")
    }

    /// 實際容量(若有向上調整,回傳調整後的值)。
    pub fn capacity(&self) -> usize {
        todo!("challenge")
    }
}
