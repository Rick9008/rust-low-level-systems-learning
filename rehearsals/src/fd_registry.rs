//! rehearsal e2:fd_registry —— 題目見 rehearsals/README.md。
//!
//! 只給 API 簽名;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`。

// use std::marker::PhantomData;

/// 要能塞進 kernel 的 u64 座位(`epoll_event.data`)往返。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token(u64);

impl Token {
    pub fn to_raw(self) -> u64 {
        self.0
    }

    pub fn pack(generation: u32, fd: u32) -> u64 {
        ((generation as u64) << 32) | fd as u64
    }

    // first is generation, second is fd
    pub fn unpack(self) -> (u32, u32) {
        let gene = (self.0 >> 32) as u32;
        let fd = (self.0 & ((1 << 32) - 1)) as u32;
        (gene, fd)
    }

    pub fn from_raw(raw: u64) -> Self {
        Token(raw)
    }
}

pub struct FdRegistry<T> {
    // ↓ 佔位:動手時整個換成你的設計。
    // _todo: PhantomData<T>,
    genes: Vec<u32>,
    fd_table: Vec<Option<T>>,
    len: usize,
}

impl<T> FdRegistry<T> {
    pub fn new() -> Self {
        // todo!("rehearsal")
        Self {
            genes: Vec::new(),
            fd_table: Vec::new(),
            len: 0,
        }
    }

    /// 登記 fd 的狀態,回一個可進出 u64 的 token。
    /// 對活著的 fd 重複 register 是 caller bug(panic 即可)。
    pub fn register(&mut self, fd: usize, value: T) -> Token {
        // todo!("rehearsal")
        assert!(
            fd < u32::MAX as usize,
            "fd should not larger or equal to u32::MAX."
        );
        if self.fd_table.len() <= fd {
            self.fd_table.resize_with(fd + 1, || None);
            self.genes.resize(fd + 1, 0);
        }

        if self.fd_table[fd].is_some() {
            panic!("You re-regsiter in a fd registry.");
        }

        let gene = self.genes[fd];
        self.fd_table[fd] = Some(value);
        self.len += 1;
        Token::from_raw(Token::pack(gene, fd as u32))
    }

    /// 移除並取回。過期 token(fd 已回收再登記)→ None,且不影響現任。
    pub fn unregister(&mut self, token: Token) -> Option<T> {
        // todo!("rehearsal")
        let (generation, fd) = token.unpack();
        if fd >= self.fd_table.len() as u32 {
            return None;
        }

        if self.genes[fd as usize] != generation {
            return None;
        }

        if self.fd_table[fd as usize].is_some() {
            self.genes[fd as usize] = self.genes[fd as usize].wrapping_add(1);
            self.len -= 1;
        }

        self.fd_table[fd as usize].take()
    }

    /// O(1) 找回狀態。**過期 token 必須回 None**——這是整題的核心需求。
    pub fn get(&self, token: Token) -> Option<&T> {
        // todo!("rehearsal")
        let (generation, fd) = token.unpack();

        if fd >= self.fd_table.len() as u32 {
            return None;
        }
        if self.genes[fd as usize] != generation {
            return None;
        }

        self.fd_table[fd as usize].as_ref()
    }

    pub fn get_mut(&mut self, token: Token) -> Option<&mut T> {
        // todo!("rehearsal")

        let (generation, fd) = token.unpack();

        if fd >= self.fd_table.len() as u32 {
            return None;
        }

        if self.genes[fd as usize] != generation {
            return None;
        }

        self.fd_table[fd as usize].as_mut()
    }

    /// 活著的登記數。
    pub fn len(&self) -> usize {
        // todo!("rehearsal")
        self.len
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

#[cfg(test)]
mod dry_run {
    use crate::fd_registry::{FdRegistry, Token};
    #[test]
    fn boundary_test() {
        // dry run:
        // genes: []
        // fd_table: []
        // len: 0
        let mut fd_reg = FdRegistry::<u32>::new();
        assert_eq!(fd_reg.len(), 0);
        assert!(fd_reg.is_empty());

        // [0, 0, 0, 0, 0, 0]
        // [x, x, x, x, x, 7]
        assert_eq!(fd_reg.register(6, 7), Token::from_raw(Token::pack(0, 6)));
        assert_eq!(fd_reg.len(), 1);
        assert_eq!(fd_reg.unregister(Token::from_raw(Token::pack(1, 6))), None);
        assert_eq!(fd_reg.get(Token::from_raw(Token::pack(0, 6))), Some(&7));
        assert_eq!(
            fd_reg.unregister(Token::from_raw(Token::pack(0, 6))),
            Some(7)
        );
        assert_eq!(fd_reg.register(6, 10), Token::from_raw(Token::pack(1, 6)));
        assert_eq!(fd_reg.get(Token::from_raw(Token::pack(0, 6))), None);
    }
}
