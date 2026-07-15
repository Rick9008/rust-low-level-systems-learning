//! rehearsal e:event_registry —— 題目見 rehearsals/README.md。
//!
//! 只給 API 簽名;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`。

/// handler 執行完後的去留(handler 自己決定,解掉「dispatch 中途 unregister」的借用問題)。
pub enum After {
    Keep,
    Remove,
}

pub type Handler = Box<dyn FnMut(u64) -> After>;

pub struct Registry {
    // ↓ 佔位:動手時整個換成你的設計。
    _todo: (),
}

impl Registry {
    pub fn new() -> Self {
        todo!("rehearsal")
    }

    /// 同一個 id 可掛多個 handler;dispatch 依註冊順序執行。
    pub fn register(&mut self, id: u32, handler: Handler) {
        todo!("rehearsal")
    }

    /// 執行 id 的所有 handler(依註冊順序),回傳執行了幾個。
    /// handler 回 `After::Remove` → 從此不再被呼叫。未知 id → 0。
    /// (dispatch 進行中不支援 register;由 caller 保證。)
    pub fn dispatch(&mut self, id: u32, payload: u64) -> usize {
        todo!("rehearsal")
    }

    /// 目前掛在 id 上的 handler 數。
    pub fn handler_count(&self, id: u32) -> usize {
        todo!("rehearsal")
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}
