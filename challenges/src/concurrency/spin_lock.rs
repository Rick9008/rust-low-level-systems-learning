//! ★ challenge:SpinLock —— 自旋鎖 + RAII guard
//!
//! 【題目】實作 `SpinLock<T>`:多執行緒共享(`&self` 上鎖)、忙等互斥、
//! 以 RAII guard 表達「持鎖」——guard 活著 = 在臨界區,guard 掉出作用域
//! (包含 panic unwind)= 自動放鎖。不使用 `Mutex`/`Condvar`,只用 atomic。
//!
//! 【constraints】
//! - std-only;`lock()` 無競爭 O(1),等待用自旋(告訴 CPU 你在忙等)
//! - guard 必須實作 `Deref`/`DerefMut`/`Drop`——解鎖不可能忘記、不可能做兩次
//! - `try_lock()` 不等待,佔用中回 `None`
//! - 跨執行緒共享必須過型別系統這關(提示:`UnsafeCell` 一進場,
//!   auto trait 就沒了——哪個 impl 要你自己 unsafe 承諾?bound 寫什麼?)
//!
//! 【clarify points——動手前先自答】
//! - `unsafe impl … Sync` 的 bound:`T: Send` 還是 `T: Sync`?差在哪?
//! - 拿鎖用哪個 Ordering?配對誰的哪個 store?保護的是哪些寫入的可見性?
//! - 等待迴圈:每次都 RMW 硬撞,和「先純 load 等到看似沒鎖再撞」,
//!   對 cache line 的行為差在哪?(這就是 TAS vs TTAS)
//! - guard 可以是 `Send` 嗎?A 執行緒鎖、B 執行緒解,壞在哪裡?
//! - 同執行緒重複 `lock()` 會發生什麼?你要不要處理?
//! - 臨界區 panic:你的鎖之後還能用嗎?資料呢?(std `Mutex` 選了 poisoning,
//!   你選什麼,代價是什麼?)
//!
//! 【要實作】下方簽名。struct 內部完全自己設計(佔位欄位整個換掉)。
//! 【驗收】tests/spin_lock.rs 轉綠(含雙執行緒 10 萬遞增與 panic 不壞死),
//! 然後 diff reference 的 `concurrency/spin_lock`,對每個 Ordering 的理由。

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct SpinLock<T> {
    // ↓ 佔位:讓空殼能編譯。動手時整個換成你的設計。
    _todo: PhantomData<T>,
}

pub struct SpinGuard<'a, T> {
    // ↓ 佔位。想清楚 guard 需要抓著什麼、以及它「不可以是什麼」(auto trait)。
    _todo: PhantomData<&'a mut T>,
}

impl<T: Send> SpinLock<T> {
    pub fn new(value: T) -> Self {
        let _ = value;
        todo!("challenge: 從空白開始")
    }

    /// 忙等拿鎖。
    #[must_use = "guard 一落地鎖就放了"]
    pub fn lock(&self) -> SpinGuard<'_, T> {
        todo!("challenge")
    }

    /// 試拿一次,不等;佔用中回 None。
    #[must_use]
    pub fn try_lock(&self) -> Option<SpinGuard<'_, T>> {
        todo!("challenge")
    }

    /// 消耗鎖取回資料(想想:這裡需要碰 atomic 嗎?為什麼?)。
    pub fn into_inner(self) -> T {
        todo!("challenge")
    }
}

impl<T> Deref for SpinGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        todo!("challenge: 憑什麼能發出 &T ——SAFETY 理由寫在 unsafe 上面")
    }
}

impl<T> DerefMut for SpinGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        todo!("challenge")
    }
}

impl<T> Drop for SpinGuard<'_, T> {
    fn drop(&mut self) {
        // challenge:放鎖動作寫這裡。哪個 Ordering?為什麼不能 Relaxed?
    }
}
