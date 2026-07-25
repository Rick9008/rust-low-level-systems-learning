//! rehearsal e2:fd_registry —— 題目見 rehearsals/README.md。
//!
//! 只給 API 簽名;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`。

// use std::marker::PhantomData;
// 10:40

/// 要能塞進 kernel 的 u64 座位(`epoll_event.data`)往返。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token(u64);

impl Token {
    pub fn to_raw(self) -> u64 {
        self.0
    }

    pub fn pack(generation: u32, fd: u32) -> u64 {
        (generation as u64) << 32 | fd as u64
    }

    // first is generation, second is fd
    pub fn unpack(self) -> (u32, u32) {
        // 0000000000..10........0001
        // 31 x 0 | 1 | 31 x 0 | 1
        (((self.0 >> 32) as u32), (self.0 as u32))
    }

    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

// 1. generation of the slots
// 2. value of the slots
// we dynamic push the value to extend the slots
pub struct FdRegistry<T> {
    generations: Vec<u32>,
    values: Vec<Option<T>>,
    len: usize,
}

impl<T> FdRegistry<T> {
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            values: Vec::new(),
            len: 0,
        }
    }

    /// 登記 fd 的狀態,回一個可進出 u64 的 token。
    /// 對活著的 fd 重複 register 是 caller bug(panic 即可)。
    pub fn register(&mut self, fd: usize, value: T) -> Token {
        assert!(fd < u32::MAX as usize, "fd is too huge.");
        if fd >= self.generations.len() {
            self.generations.resize(fd + 1, 0);
            self.values.resize_with(fd + 1, || None);
        }
        if self.values[fd].is_some() {
            panic!("You should register with a living fd.")
        }
        self.values[fd] = Some(value);
        self.len += 1;
        Token::from_raw(Token::pack(self.generations[fd], fd as u32))
    }

    /// 移除並取回。過期 token(fd 已回收再登記)→ None,且不影響現任。
    pub fn unregister(&mut self, token: Token) -> Option<T> {
        let (generation, fd) = token.unpack();
        // SANITY TEST
        if fd as usize >= self.generations.len() {
            return None;
        }
        if generation != self.generations[fd as usize] {
            return None;
        }
        self.values[fd as usize].as_ref()?;
        self.len -= 1;
        // To handle the boundary case. However, the overflow part still need little thinking on the
        // going back to 0.
        self.generations[fd as usize] = self.generations[fd as usize].wrapping_add(1);
        self.values[fd as usize].take()
    }

    /// O(1) 找回狀態。**過期 token 必須回 None**——這是整題的核心需求。
    pub fn get(&self, token: Token) -> Option<&T> {
        let (generation, fd) = token.unpack();
        if fd as usize >= self.generations.len() {
            return None;
        }
        if self.generations[fd as usize] != generation {
            return None;
        }
        self.values[fd as usize].as_ref()
    }

    pub fn get_mut(&mut self, token: Token) -> Option<&mut T> {
        let (generation, fd) = token.unpack();
        if fd as usize >= self.generations.len() {
            return None;
        }
        if self.generations[fd as usize] != generation {
            return None;
        }
        self.values[fd as usize].as_mut()
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
    #[test]
    fn forged_current_gen_token_on_empty_slot_is_safe() {
        let mut reg = FdRegistry::<u32>::new();
        let t = reg.register(0, 7);
        assert_eq!(reg.unregister(t), Some(7)); // gen 0→1,槽空,len=0
        let forged = Token::from_raw(Token::pack(1, 0)); // gen 剛好對上空槽
        assert_eq!(reg.unregister(forged), None); // mutation 在:len 0-1 → panic → 紅
    }

    /// 洞②:高位 fd 的 pack/unpack roundtrip(不經 registry,這是唯一付得起的網)
    #[test]
    fn token_roundtrip_high_fd_bits() {
        let fd = (1u32 << 31) + 7;
        assert_eq!(Token::from_raw(Token::pack(3, fd)).unpack(), (3, fd));
    }
}
