//! rehearsal e2:fd_registry —— 題目見 rehearsals/README.md。
//!
//! 只給 API 簽名;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`。

use std::marker::PhantomData;

/// 要能塞進 kernel 的 u64 座位(`epoll_event.data`)往返。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token(u64);

impl Token {
    pub fn to_raw(self) -> u64 {
        self.0
    }

    pub fn from_raw(raw: u64) -> Self {
        Token(raw)
    }
}

pub struct FdRegistry<T> {
    // ↓ 佔位:動手時整個換成你的設計。
    _todo: PhantomData<T>,
}

impl<T> FdRegistry<T> {
    pub fn new() -> Self {
        todo!("rehearsal")
    }

    /// 登記 fd 的狀態,回一個可進出 u64 的 token。
    /// 對活著的 fd 重複 register 是 caller bug(panic 即可)。
    pub fn register(&mut self, fd: usize, value: T) -> Token {
        todo!("rehearsal")
    }

    /// 移除並取回。過期 token(fd 已回收再登記)→ None,且不影響現任。
    pub fn unregister(&mut self, token: Token) -> Option<T> {
        todo!("rehearsal")
    }

    /// O(1) 找回狀態。**過期 token 必須回 None**——這是整題的核心需求。
    pub fn get(&self, token: Token) -> Option<&T> {
        todo!("rehearsal")
    }

    pub fn get_mut(&mut self, token: Token) -> Option<&mut T> {
        todo!("rehearsal")
    }

    /// 活著的登記數。
    pub fn len(&self) -> usize {
        todo!("rehearsal")
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Default for FdRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}
