//! # event_loop —— register / epoll_wait / dispatch 的 readiness 事件迴圈
//!
//! ## [Clarify]
//! 解決:單執行緒服務上千個連線。thread-per-connection 在 C10K 下
//! stack(每條 ~2-8MB 位址空間)與 context switch(~1-10μs)都不 scale;
//! event loop 用一條執行緒 + epoll,把「等 N 個 fd」變成一次 syscall。
//! 本模組是薄的 poller(mio 的形狀):註冊(fd, token, interest, LT/ET)、
//! poll 出事件、dispatch 由 caller 的 match token 完成——
//! 應用(tcp_echo、hw_bridge)自己擁有連線狀態。
//!
//! ## [Abstract]
//! timer(超時管理)與跨執行緒任務佇列不做——wake 機制(eventfd)給了,
//! 上面掛什麼 caller 決定;面試聲明「timer wheel 之後補」往前走。
//!
//! ## [Trade-offs]
//! - **事件緩衝由 caller 持有(`Events`)**:poll(&mut self, &mut Events)
//!   把「迴圈本體」與「事件批次」的借用分開——處理事件時還能呼叫
//!   register/deregister(mio 同款設計;若 poll 回傳 &[Event] 會借用衝突)。
//! - 緩衝固定 64 事件:一次 wait 拿不完下次再拿(epoll 會繼續報),
//!   O(1) 空間換 syscall 次數,量大時攤銷掉。
//! - **eventfd self-wake**:epoll_wait 阻塞中的迴圈,別的執行緒要它醒
//!   (提交任務/要求停機)只能透過一個「它有在聽的 fd」。
//!   wake 走 `Arc<EventFd>`:handle 不因迴圈先 drop 而變 dangling fd。
//! - LT vs ET 都支援(`Trigger`):LT 好寫(漏處理下次還報);
//!   ET 少假醒但**必須讀寫到 EAGAIN**,漏了就是永久 stall——見 epoll_sys 測試。
//!
//! ## [Dry-Run]
//! 測試:LT 重複上報/ET 一次、跨執行緒 wake、timeout、
//! socket 可寫事件、dispatch 迴圈中改註冊。
//!
//! Production 對照:mio(本模組的工業版)、tokio 的 reactor 層。

use crate::epoll_sys::{
    EPOLLERR, EPOLLET, EPOLLHUP, EPOLLIN, EPOLLOUT, EPOLLPRI, EPOLLRDHUP, Epoll, EpollEvent,
    EventFd,
};
use std::io;
use std::os::fd::RawFd;
use std::sync::Arc;
use std::time::Duration;

/// 使用者自訂識別:事件回來時用它找回「是哪個連線」。
/// u64::MAX 保留給內部 wake。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token(pub u64);

const WAKE_TOKEN: u64 = u64::MAX;

/// 感興趣的就緒方向。
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

    fn to_epoll(self, trigger: Trigger) -> u32 {
        let mut ev = EPOLLRDHUP; // 半關閉一律想知道(免費,還早 read()==0 一步)
        if self.readable {
            ev |= EPOLLIN;
        }
        if self.writable {
            ev |= EPOLLOUT;
        }
        if let Trigger::Edge = trigger {
            ev |= EPOLLET;
        }
        ev
    }
}

/// LT(預設,狀態持續就報)vs ET(狀態變化才報)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Trigger {
    #[default]
    Level,
    Edge,
}

/// 翻譯後的就緒事件(kernel 遮罩 → 具名 bool,caller 不用記位元)。
#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub token: Token,
    pub readable: bool,
    pub writable: bool,
    /// 對端關了寫端(EPOLLRDHUP)或整條斷了(EPOLLHUP)。
    pub peer_closed: bool,
    /// fd 出錯(EPOLLERR)——處理方式幾乎總是:關掉這條連線。
    pub error: bool,
}

/// 一批事件的緩衝,caller 持有(借用分離,見模組 doc)。
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

    /// 這輪 poll 是否被 `WakeHandle::wake` 叫醒(檢查外部佇列的訊號)。
    pub fn woken(&self) -> bool {
        self.woken
    }
}

/// 跨執行緒喚醒把手:Clone + Send。
#[derive(Clone)]
pub struct WakeHandle {
    efd: Arc<EventFd>,
}

impl WakeHandle {
    /// 叫醒(或即將進入)epoll_wait 的迴圈。多次 wake 合併(eventfd 計數器)。
    /// 迴圈那端還沒睡也不丟:計數已 +1,下次 wait 立即返回——
    /// 與 executor 的 park permit 同一形狀的「帶狀態訊號」。
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
        // wake fd 用 LT:poll 的翻譯層一定會 drain,不需要 ET 的紀律。
        epoll.add(wake.as_raw_fd(), EPOLLIN, WAKE_TOKEN)?;
        Ok(Self { epoll, wake })
    }

    pub fn wake_handle(&self) -> WakeHandle {
        WakeHandle {
            efd: Arc::clone(&self.wake),
        }
    }

    /// 註冊 fd。caller 保證 fd 已設 nonblocking(event loop 裡任何
    /// 阻塞 IO 都會凍住全部連線——這是 readiness model 的鐵律)。
    pub fn register(
        &self,
        fd: RawFd,
        token: Token,
        interest: Interest,
        trigger: Trigger,
    ) -> io::Result<()> {
        assert!(token.0 != WAKE_TOKEN, "u64::MAX is reserved");
        self.epoll.add(fd, interest.to_epoll(trigger), token.0)
    }

    /// 改 interest(如 write 緩衝清空後拿掉 WRITABLE——LT 下常掛 WRITABLE
    /// 會每輪都報「可寫」,busy loop)。
    pub fn reregister(
        &self,
        fd: RawFd,
        token: Token,
        interest: Interest,
        trigger: Trigger,
    ) -> io::Result<()> {
        assert!(token.0 != WAKE_TOKEN, "u64::MAX is reserved");
        self.epoll.modify(fd, interest.to_epoll(trigger), token.0)
    }

    pub fn deregister(&self, fd: RawFd) -> io::Result<()> {
        self.epoll.delete(fd)
    }

    /// 等一批事件。timeout None = 等到有事件或被 wake。
    /// wake 事件在此翻譯層消化(drain 計數、設 woken 旗標),不進事件列表。
    pub fn poll(&mut self, events: &mut Events, timeout: Option<Duration>) -> io::Result<()> {
        events.list.clear();
        events.woken = false;
        let timeout_ms = timeout.map(|d| i32::try_from(d.as_millis()).unwrap_or(i32::MAX));
        let n = self.epoll.wait(&mut events.raw, timeout_ms)?;
        for raw in &events.raw[..n] {
            let raw = *raw; // packed struct:整值 copy 再讀欄位
            if raw.data == WAKE_TOKEN {
                self.wake.drain()?; // LT:不 drain 下輪還報
                events.woken = true;
                continue;
            }
            events.list.push(Event {
                token: Token(raw.data),
                readable: raw.events & (EPOLLIN | EPOLLPRI) != 0,
                writable: raw.events & EPOLLOUT != 0,
                peer_closed: raw.events & (EPOLLRDHUP | EPOLLHUP) != 0,
                error: raw.events & EPOLLERR != 0,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::os::fd::AsRawFd;
    use std::os::unix::net::UnixStream;
    use std::time::Instant;

    fn nonblocking_pair() -> (UnixStream, UnixStream) {
        let (a, b) = UnixStream::pair().unwrap();
        a.set_nonblocking(true).unwrap();
        b.set_nonblocking(true).unwrap();
        (a, b)
    }

    /// [Dry-Run] register → 寫入對端 → poll 報 readable(token 對上)→
    /// 讀走資料 → poll(0) 無事件。LT 全流程。
    #[test]
    fn register_poll_dispatch_roundtrip() {
        let mut el = EventLoop::new().unwrap();
        let mut events = Events::with_capacity(8);
        let (mut a, b) = nonblocking_pair();
        el.register(b.as_raw_fd(), Token(5), Interest::READABLE, Trigger::Level)
            .unwrap();
        a.write_all(b"hi").unwrap();
        el.poll(&mut events, Some(Duration::from_millis(500)))
            .unwrap();
        assert_eq!(events.len(), 1);
        let ev = events.iter().next().unwrap();
        assert_eq!(ev.token, Token(5));
        assert!(ev.readable && !ev.writable);
        // dispatch(caller 的事):讀走
        let mut buf = [0u8; 8];
        let mut b_read = &b;
        let n = b_read.read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"hi");
        el.poll(&mut events, Some(Duration::from_millis(0)))
            .unwrap();
        assert!(events.is_empty()); // 資料讀完,LT 不再報
    }

    /// boundary:LT 沒讀走 → 下一輪**還報**;ET → 下一輪**不報**。
    /// (與 epoll_sys 的測試互補:這裡驗的是 wrapper 的 Trigger 參數。)
    #[test]
    fn boundary_lt_rereports_et_does_not() {
        for (trigger, second_round) in [(Trigger::Level, 1usize), (Trigger::Edge, 0usize)] {
            let mut el = EventLoop::new().unwrap();
            let mut events = Events::with_capacity(8);
            let (mut a, b) = nonblocking_pair();
            el.register(b.as_raw_fd(), Token(1), Interest::READABLE, trigger)
                .unwrap();
            a.write_all(b"x").unwrap();
            el.poll(&mut events, Some(Duration::from_millis(500)))
                .unwrap();
            assert_eq!(events.len(), 1, "{trigger:?} 第一輪必報");
            // 故意不讀 b
            el.poll(&mut events, Some(Duration::from_millis(50)))
                .unwrap();
            assert_eq!(events.len(), second_round, "{trigger:?} 第二輪");
        }
    }

    /// boundary:跨執行緒 wake——poll 無限等,另一執行緒 30ms 後 wake,
    /// poll 返回、woken=true、無 fd 事件。這是 shutdown/任務注入的機制。
    #[test]
    fn boundary_wake_from_other_thread_interrupts_infinite_wait() {
        let mut el = EventLoop::new().unwrap();
        let mut events = Events::with_capacity(8);
        let handle = el.wake_handle();
        let t = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(30));
            handle.wake().unwrap();
        });
        let start = Instant::now();
        el.poll(&mut events, None).unwrap(); // 無 timeout:只有 wake 能救
        assert!(events.woken());
        assert!(events.is_empty());
        assert!(start.elapsed() >= Duration::from_millis(25));
        t.join().unwrap();
        // wake 已被 drain:下一輪不會又醒
        el.poll(&mut events, Some(Duration::from_millis(0)))
            .unwrap();
        assert!(!events.woken());
    }

    /// boundary:wake 先於 poll(迴圈還沒睡)——計數已掛,poll 立即返回。
    /// 與 executor 的「unpark 先於 park」同一課。
    #[test]
    fn boundary_wake_before_poll_not_lost() {
        let mut el = EventLoop::new().unwrap();
        let mut events = Events::with_capacity(8);
        el.wake_handle().wake().unwrap(); // 先叫
        let start = Instant::now();
        el.poll(&mut events, None).unwrap(); // 後睡:必須立即醒
        assert!(events.woken());
        assert!(start.elapsed() < Duration::from_millis(100));
    }

    /// 可寫事件:剛建立的 socket 送緩衝有空位 → WRITABLE 立即就緒。
    /// (LT 下持續掛 WRITABLE 會每輪都報——tcp_echo 的 interest 狀態機
    /// 就是為了避免這個 busy loop。)
    #[test]
    fn writable_reported_for_fresh_socket() {
        let mut el = EventLoop::new().unwrap();
        let mut events = Events::with_capacity(8);
        let (a, _b) = nonblocking_pair();
        el.register(a.as_raw_fd(), Token(3), Interest::BOTH, Trigger::Level)
            .unwrap();
        el.poll(&mut events, Some(Duration::from_millis(500)))
            .unwrap();
        let ev = events.iter().next().unwrap();
        assert!(ev.writable && !ev.readable);
    }

    /// dispatch 中改註冊(borrow 分離的存在理由):
    /// 處理 a 的事件時把 b 註冊進來——Events 與 EventLoop 是兩個值,不衝突。
    #[test]
    fn reregister_inside_dispatch_loop() {
        let mut el = EventLoop::new().unwrap();
        let mut events = Events::with_capacity(8);
        let (mut a, b) = nonblocking_pair();
        let (mut c, d) = nonblocking_pair();
        el.register(b.as_raw_fd(), Token(1), Interest::READABLE, Trigger::Level)
            .unwrap();
        a.write_all(b"x").unwrap();
        c.write_all(b"y").unwrap();
        el.poll(&mut events, Some(Duration::from_millis(500)))
            .unwrap();
        for ev in events.iter() {
            if ev.token == Token(1) {
                // dispatch 中註冊新 fd:不會 borrow 衝突
                el.register(d.as_raw_fd(), Token(2), Interest::READABLE, Trigger::Level)
                    .unwrap();
            }
        }
        el.poll(&mut events, Some(Duration::from_millis(500)))
            .unwrap();
        assert!(events.iter().any(|e| e.token == Token(2)));
    }

    /// boundary:對端關閉 → peer_closed(EPOLLRDHUP/HUP)上報。
    #[test]
    fn boundary_peer_close_reported() {
        let mut el = EventLoop::new().unwrap();
        let mut events = Events::with_capacity(8);
        let (a, b) = nonblocking_pair();
        el.register(b.as_raw_fd(), Token(9), Interest::READABLE, Trigger::Level)
            .unwrap();
        drop(a); // 對端整條關掉
        el.poll(&mut events, Some(Duration::from_millis(500)))
            .unwrap();
        let ev = events.iter().next().unwrap();
        assert!(ev.peer_closed);
    }
}
