//! solution:題 e event_registry——**寫完彩排才開**。
//! canonical 設計:`HashMap<id, Vec<Handler>>` + `retain_mut` 一趟完成
//! 「執行 + 依回傳值移除」——handler 用回傳值自我移除,繞開 dispatch 中途
//! unregister 的借用打架。id 密集且有界時換 `Vec<Option<...>>` 直接 index
//! (worst-case O(1)、零 hashing,代價是 max_id 個 slot)。
//! 驗證:rehearsals/tests/event_registry_test.rs 全綠。

use std::collections::HashMap;

pub enum After {
    Keep,
    Remove,
}

pub type Handler = Box<dyn FnMut(u64) -> After>;

pub struct Registry {
    map: HashMap<u32, Vec<Handler>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: u32, handler: Handler) {
        self.map.entry(id).or_default().push(handler);
    }

    pub fn dispatch(&mut self, id: u32, payload: u64) -> usize {
        let Some(list) = self.map.get_mut(&id) else {
            return 0;
        };
        let mut ran = 0;
        // retain_mut:依序執行,回 Remove 的當場拔掉,順序保持
        list.retain_mut(|h| {
            ran += 1;
            matches!(h(payload), After::Keep)
        });
        if list.is_empty() {
            self.map.remove(&id);
        }
        ran
    }

    pub fn handler_count(&self, id: u32) -> usize {
        self.map.get(&id).map_or(0, |v| v.len())
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let mut r = Registry::new();
    let hits = Rc::new(RefCell::new(0u32));
    let h = Rc::clone(&hits);
    r.register(
        7,
        Box::new(move |_| {
            *h.borrow_mut() += 1;
            After::Remove // 跑一次就自我移除
        }),
    );
    assert_eq!(r.dispatch(7, 0), 1);
    assert_eq!(r.dispatch(7, 0), 0); // 已移除
    assert_eq!(*hits.borrow(), 1);
    assert_eq!(r.dispatch(999, 0), 0); // 未知 id no-op
    println!("sol_event_registry: ok");
}
