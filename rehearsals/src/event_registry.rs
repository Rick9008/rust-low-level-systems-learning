#![allow(unused_imports)]
//! rehearsal e:event_registry —— 題目見 rehearsals/README.md。
//!
//! 只給 API 簽名;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`。

use std::collections::HashMap;

/// handler 執行完後的去留(handler 自己決定,解掉「dispatch 中途 unregister」的借用問題)。
pub enum After {
    Keep,
    Remove,
}

pub type Handler = Box<dyn FnMut(u64) -> After>;

pub struct Registry {
    // ↓ 佔位:動手時整個換成你的設計。
    // _todo: (),
    map: HashMap<u32, Vec<Handler>>,
    cnt: usize,
}

impl Registry {
    pub fn new() -> Self {
        // todo!("rehearsal")
        Self {
            map: HashMap::new(),
            cnt: 0,
        }
    }

    /// 同一個 id 可掛多個 handler;dispatch 依註冊順序執行。
    /// TC: O(1)
    pub fn register(&mut self, id: u32, handler: Handler) {
        self.map.entry(id).or_default().push(handler);
        self.cnt += 1;
    }

    /// 執行 id 的所有 handler(依註冊順序),回傳執行了幾個。
    /// handler 回 `After::Remove` → 從此不再被呼叫。未知 id → 0。
    /// (dispatch 進行中不支援 register;由 caller 保證。)
    /// TC: O(k), k is the handlers count
    pub fn dispatch(&mut self, id: u32, payload: u64) -> usize {
        let handlers = match self.map.get_mut(&id) {
            Some(handlers) => handlers,
            None => return 0,
        };
        let cnt = handlers.len();
        self.cnt -= cnt;
        handlers.retain_mut(|handle| match handle(payload) {
            After::Keep => true,
            After::Remove => false,
        });
        self.cnt += handlers.len();
        cnt
    }

    /// 目前掛在 id 上的 handler 數。
    /// TC: O(1)
    pub fn handler_count(&self, id: u32) -> usize {
        if !self.map.contains_key(&id) {
            return 0;
        }
        self.map[&id].len()
    }

    pub fn total_handlers(&self) -> usize {
        self.cnt
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

#[test]
fn smoke() {
    let smoking = Arc::new(AtomicI64::new(0));
    let smoking_arc = smoking.clone();
    let cnt = Arc::new(AtomicI64::new(0));
    let cnt_1 = cnt.clone();
    let cnt_2 = cnt.clone();
    let mut reg = Registry::new();
    reg.register(
        0,
        Box::new(move |payload| {
            smoking.fetch_add(1, Ordering::Relaxed);
            cnt_1.fetch_add(1, Ordering::Relaxed);
            if smoking.load(Ordering::Relaxed) >= 1 {
                After::Remove
            } else {
                After::Keep
            }
        }),
    );
    reg.register(
        0,
        Box::new(move |payload| {
            smoking_arc.fetch_add(1, Ordering::Relaxed);
            cnt_2.fetch_add(1, Ordering::Relaxed);
            if smoking_arc.load(Ordering::Relaxed) >= 5 {
                After::Remove
            } else {
                After::Keep
            }
        }),
    );
    assert_eq!(reg.handler_count(1), 0);
    assert_eq!(reg.handler_count(0), 2);
    assert_eq!(reg.dispatch(0, 0), 2);
    assert_eq!(reg.handler_count(0), 1);
    assert_eq!(reg.dispatch(0, 0), 1);
    assert_eq!(cnt.load(Ordering::Relaxed), 3);
}
