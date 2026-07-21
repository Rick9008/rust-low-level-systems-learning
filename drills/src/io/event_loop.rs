//! drill:event_loop —— 填 poll 的事件翻譯與 wake 消化。
//!
//! 已給:Token/Interest/Trigger/Event/Events/WakeHandle、register 系列。
//! 要填:`Interest::to_epoll`(旗標翻譯)與 `EventLoop::poll`
//! (epoll_wait → 翻譯事件 → wake 事件在翻譯層消化)。
//! 注意:這個 drill 直接使用 **reference 的 epoll_sys**(已驗證的底層),
//! 你只練 event loop 這一層。

use reference::io::epoll_sys::{
    EPOLLERR, EPOLLET, EPOLLHUP, EPOLLIN, EPOLLOUT, EPOLLPRI, EPOLLRDHUP, Epoll, EpollEvent,
    EventFd,
};
use std::io;
use std::os::fd::RawFd;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token(pub u64);

const WAKE_TOKEN: u64 = u64::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interest {
    pub readable: bool,
    pub writable: bool,
}

impl Interest {
    pub const READABLE: Interest = Interest {
        readable: true,
        writable: false,
    };
    pub const WRITABLE: Interest = Interest {
        readable: false,
        writable: true,
    };
    pub const BOTH: Interest = Interest {
        readable: true,
        writable: true,
    };

    /// spec:翻成 epoll 旗標——EPOLLRDHUP 永遠帶上(半關閉早知道);
    /// readable → EPOLLIN、writable → EPOLLOUT、Edge → EPOLLET。
    fn to_epoll(self, trigger: Trigger) -> u32 {
        todo!("spec: 起手 EPOLLRDHUP,按欄位 OR 上去")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Trigger {
    #[default]
    Level,
    Edge,
}

#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub token: Token,
    pub readable: bool,
    pub writable: bool,
    pub peer_closed: bool,
    pub error: bool,
}

pub struct Events {
    raw: Vec<EpollEvent>,
    list: Vec<Event>,
    woken: bool,
}

impl Events {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            raw: vec![EpollEvent { events: 0, data: 0 }; cap.max(1)],
            list: Vec::with_capacity(cap.max(1)),
            woken: false,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Event> {
        self.list.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn woken(&self) -> bool {
        self.woken
    }
}

#[derive(Clone)]
pub struct WakeHandle {
    efd: Arc<EventFd>,
}

impl WakeHandle {
    pub fn wake(&self) -> io::Result<()> {
        self.efd.notify()
    }
}

pub struct EventLoop {
    epoll: Epoll,
    wake: Arc<EventFd>,
}

impl EventLoop {
    pub fn new() -> io::Result<Self> {
        let epoll = Epoll::new()?;
        let wake = Arc::new(EventFd::new()?);
        epoll.add(wake.as_raw_fd(), EPOLLIN, WAKE_TOKEN)?;
        Ok(Self { epoll, wake })
    }

    pub fn wake_handle(&self) -> WakeHandle {
        WakeHandle {
            efd: Arc::clone(&self.wake),
        }
    }

    pub fn register(
        &self,
        fd: RawFd,
        token: Token,
        interest: Interest,
        trigger: Trigger,
    ) -> io::Result<()> {
        assert!(token.0 != WAKE_TOKEN);
        self.epoll.add(fd, interest.to_epoll(trigger), token.0)
    }

    pub fn reregister(
        &self,
        fd: RawFd,
        token: Token,
        interest: Interest,
        trigger: Trigger,
    ) -> io::Result<()> {
        assert!(token.0 != WAKE_TOKEN);
        self.epoll.modify(fd, interest.to_epoll(trigger), token.0)
    }

    pub fn deregister(&self, fd: RawFd) -> io::Result<()> {
        self.epoll.delete(fd)
    }

    /// spec:等一批事件並翻譯進 `events`。
    /// 1. 清空 events.list、重置 woken
    /// 2. `self.epoll.wait(&mut events.raw, timeout_ms)`(None → 無限等)
    /// 3. 逐一翻譯前 n 個 raw 事件(**packed struct:先整值 copy 再讀欄位**):
    ///    - data == WAKE_TOKEN → `self.wake.drain()?`、events.woken = true、
    ///      **不進 list**(wake 是 loop 的內部機制,不是使用者事件)
    ///    - 其他 → Event { readable: IN|PRI, writable: OUT,
    ///      peer_closed: RDHUP|HUP, error: ERR }
    pub fn poll(&mut self, events: &mut Events, timeout: Option<Duration>) -> io::Result<()> {
        todo!("spec: wait → 翻譯;wake 事件 drain 掉並設旗標")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::os::fd::AsRawFd;
    use std::os::unix::net::UnixStream;

    /// boundary:readable 事件翻譯 + token 對應。
    #[test]
    #[ignore = "填完 to_epoll/poll 後移除"]
    fn readable_event_translated() {
        let mut el = EventLoop::new().unwrap();
        let mut events = Events::with_capacity(8);
        let (mut a, b) = UnixStream::pair().unwrap();
        b.set_nonblocking(true).unwrap();
        el.register(b.as_raw_fd(), Token(5), Interest::READABLE, Trigger::Level)
            .unwrap();
        a.write_all(b"hi").unwrap();
        el.poll(&mut events, Some(Duration::from_millis(500)))
            .unwrap();
        let ev = events.iter().next().unwrap();
        assert_eq!(ev.token, Token(5));
        assert!(ev.readable && !ev.writable);
    }

    /// boundary:wake 先於 poll 不丟;wake 事件不進 list、woken 旗標立起。
    #[test]
    #[ignore = "填完 to_epoll/poll 後移除"]
    fn wake_before_poll_sets_flag_only() {
        let mut el = EventLoop::new().unwrap();
        let mut events = Events::with_capacity(8);
        el.wake_handle().wake().unwrap();
        el.poll(&mut events, None).unwrap(); // 必須立即返回
        assert!(events.woken());
        assert!(events.is_empty());
        // drain 過了:下一輪不會又醒
        el.poll(&mut events, Some(Duration::from_millis(0)))
            .unwrap();
        assert!(!events.woken());
    }
}
