//! rehearsal g:bounded_channel —— 題目見 rehearsals/README.md。
//!
//! std-only(`Mutex` / `Condvar` / `Arc`)。
//! 只給 API 簽名;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`。

use std::marker::PhantomData;

/// receiver 已 drop;把值原封還給你。
#[derive(Debug)]
pub struct SendError<T>(pub T);

pub struct Sender<T> {
    // ↓ 佔位:動手時整個換成你的設計。
    _todo: PhantomData<T>,
}

pub struct Receiver<T> {
    _todo: PhantomData<T>,
}

/// `capacity >= 1`。多生產者(`Sender: Clone`)、單消費者。
pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    todo!("rehearsal")
}

impl<T> Sender<T> {
    /// 滿 → block 到有空位;receiver 已 drop → `Err(SendError(v))`。
    pub fn send(&self, v: T) -> Result<(), SendError<T>> {
        todo!("rehearsal")
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        todo!("rehearsal")
    }
}

impl<T> Receiver<T> {
    /// 空 → block;所有 sender 都 drop 且 buffer 已清空 → None。
    pub fn recv(&self) -> Option<T> {
        todo!("rehearsal")
    }
}
