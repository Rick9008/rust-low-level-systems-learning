//! solution:題 e2 fd_registry——**寫完彩排才開**。
//! canonical 設計:`Vec<Option<T>>` slots(index = fd,密集所以直接 index)
//! + `Vec<u32>` generation(unregister 成功時 +1),token 打包
//! `(gen << 32) | fd` 恰好塞進 epoll_event.data 的 u64。
//! 過期 token 的 gen 對不上 → get/unregister 自然 None——一個欄位擋掉
//! 「fd 回收後舊 event dispatch 到新住戶」的經典 bug。
//! 驗證:rehearsals/tests/fd_registry_test.rs 全綠。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token(u64);

impl Token {
    pub fn to_raw(self) -> u64 {
        self.0
    }

    pub fn from_raw(raw: u64) -> Self {
        Token(raw)
    }

    fn fd_index(self) -> usize {
        (self.0 & 0xFFFF_FFFF) as usize
    }

    fn generation(self) -> u32 {
        (self.0 >> 32) as u32
    }
}

pub struct FdRegistry<T> {
    slots: Vec<Option<T>>,
    gens: Vec<u32>,
    len: usize,
}

impl<T> FdRegistry<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            gens: Vec::new(),
            len: 0,
        }
    }

    pub fn register(&mut self, fd: usize, value: T) -> Token {
        assert!(fd <= u32::MAX as usize);
        if fd >= self.slots.len() {
            self.slots.resize_with(fd + 1, || None);
            self.gens.resize(fd + 1, 0);
        }
        assert!(self.slots[fd].is_none(), "fd {fd} already registered");
        self.slots[fd] = Some(value);
        self.len += 1;
        Token(((self.gens[fd] as u64) << 32) | fd as u64)
    }

    pub fn unregister(&mut self, token: Token) -> Option<T> {
        let fd = token.fd_index();
        if self.gens.get(fd).copied() != Some(token.generation()) {
            return None;
        }
        let value = self.slots[fd].take();
        if value.is_some() {
            self.gens[fd] = self.gens[fd].wrapping_add(1); // 換代:舊 token 從此對不上
            self.len -= 1;
        }
        value
    }

    pub fn get(&self, token: Token) -> Option<&T> {
        let fd = token.fd_index();
        if self.gens.get(fd).copied() != Some(token.generation()) {
            return None;
        }
        self.slots[fd].as_ref()
    }

    pub fn get_mut(&mut self, token: Token) -> Option<&mut T> {
        let fd = token.fd_index();
        if self.gens.get(fd).copied() != Some(token.generation()) {
            return None;
        }
        self.slots[fd].as_mut()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

fn main() {
    // smoke:fd 重用 trace——題幹那個 bug 的最小重現。
    let mut r = FdRegistry::new();
    let t1 = r.register(5, "conn-A");
    assert_eq!(r.unregister(t1), Some("conn-A"));
    let t2 = r.register(5, "conn-B");
    assert_eq!(r.get(t1), None, "stale token 已死");
    assert_eq!(r.get(Token::from_raw(t2.to_raw())), Some(&"conn-B"));
    assert!(!r.is_empty());
    println!("sol_fd_registry: stale token 擋下,ok");
}
