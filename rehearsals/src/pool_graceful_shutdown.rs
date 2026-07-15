//! rehearsal b:pool_graceful_shutdown —— 題目見 rehearsals/README.md。
//!
//! 只給 API 簽名。std-only;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`。

pub struct Pool {
    // ↓ 佔位:動手時整個換成你的設計。
    _todo: (),
}

/// submit 被拒(pool 已 shutdown)。
#[derive(Debug, PartialEq, Eq)]
pub struct Rejected;

impl Pool {
    /// 起 `workers` 條 worker 執行緒;`workers >= 1`。
    pub fn new(workers: usize) -> Self {
        todo!("rehearsal")
    }

    /// 已 shutdown → `Err(Rejected)`;回 `Ok` 代表任務保證會被執行。
    pub fn submit<F>(&self, job: F) -> Result<(), Rejected>
    where
        F: FnOnce() + Send + 'static,
    {
        todo!("rehearsal")
    }

    /// 阻塞到所有已接受的任務執行完;之後的 submit 一律拒絕;可重複呼叫。
    pub fn shutdown(&self) {
        todo!("rehearsal")
    }
}
