//! event-loop server:scale 的答案,複雜度換 C10K。
//!
//! Thread 切分(三段式):
//! ```text
//! [IO thread]      event_loop:accept + nonblocking read/write + framing
//!       │ dispatch(frame → job)
//! [command worker] ThreadPool(1):執行硬體命令(慢也不卡 IO)
//!       │ outbox.push(token, resp_bytes) + wake()
//! [IO thread]      被 eventfd 喚醒 → 把 response 路由回連線、flush
//! ```
//!
//! 為什麼 worker 是 1 條:協定沒有 request-id,client 靠 **FIFO 順序**
//! 對應回應;多 worker 會讓同連線的兩條命令亂序完成。要多 worker,
//! 協定得先加 request-id(見 client.rs 的 doc)——這是「協定設計決定
//! 並發上限」的實例。
//!
//! IO 骨架(nonblocking、欠帳緩衝、interest 狀態機)與 tcp_echo 同構,
//! 差別只在:read 到的 bytes 進 FrameReader 而不是原樣回寫。

use super::framer::FrameReader;
use super::handler::CommandHandler;
use super::protocol::{Command, ERR_BAD_PAYLOAD, ERR_UNKNOWN_OPCODE, Response, WireError};
use crate::event_loop::{Event, EventLoop, Events, Interest, Token, Trigger, WakeHandle};
use crate::thread_pool::ThreadPool;
use std::collections::{HashMap, VecDeque};
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const LISTENER_TOKEN: Token = Token(0);

struct Conn {
    stream: TcpStream,
    framer: FrameReader,
    out: VecDeque<u8>,
    registered: Interest,
    read_eof: bool,
    /// 已派給 worker、回應還沒路由回來的命令數。
    /// EOF 後要等它歸零才能關線,否則最後幾個回應會被丟掉。
    in_flight: usize,
}

/// worker → IO thread 的回程信箱型別:(token, encoded response)。
type Outbox = Arc<Mutex<Vec<(u64, Vec<u8>)>>>;

#[derive(Clone)]
pub struct EventedShutdown {
    stop: Arc<AtomicBool>,
    wake: WakeHandle,
}

impl EventedShutdown {
    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::Release);
        let _ = self.wake.wake();
    }
}

pub struct EventedServer<H: CommandHandler + 'static> {
    el: EventLoop,
    events: Events,
    listener: TcpListener,
    conns: HashMap<u64, Conn>,
    next_token: u64,
    handler: Arc<Mutex<H>>,
    /// 命令執行緒(見模組 doc:1 條 = 保序)。
    workers: ThreadPool,
    /// worker → IO thread 的回程信箱。
    outbox: Outbox,
    stop: Arc<AtomicBool>,
}

impl<H: CommandHandler + 'static> EventedServer<H> {
    pub fn bind(addr: &str, handler: H) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let el = EventLoop::new()?;
        el.register(
            listener.as_raw_fd(),
            LISTENER_TOKEN,
            Interest::READABLE,
            Trigger::Level,
        )?;
        Ok(Self {
            el,
            events: Events::with_capacity(64),
            listener,
            conns: HashMap::new(),
            next_token: 1,
            handler: Arc::new(Mutex::new(handler)),
            workers: ThreadPool::new(1),
            outbox: Arc::new(Mutex::new(Vec::new())),
            stop: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn shutdown_handle(&self) -> EventedShutdown {
        EventedShutdown {
            stop: Arc::clone(&self.stop),
            wake: self.el.wake_handle(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        while !self.stop.load(Ordering::Acquire) {
            self.el.poll(&mut self.events, None)?;
            // 先路由 worker 的回應(woken 通常意味著信箱有貨)
            if self.events.woken() {
                self.route_outbox();
            }
            let batch: Vec<Event> = self.events.iter().copied().collect();
            for ev in batch {
                if ev.token == LISTENER_TOKEN {
                    self.accept_all()?;
                } else {
                    self.drive_conn(ev);
                }
            }
        }
        Ok(())
    }

    fn accept_all(&mut self) -> io::Result<()> {
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    stream.set_nonblocking(true)?;
                    let token = Token(self.next_token);
                    self.next_token += 1;
                    self.el.register(
                        stream.as_raw_fd(),
                        token,
                        Interest::READABLE,
                        Trigger::Level,
                    )?;
                    self.conns.insert(
                        token.0,
                        Conn {
                            stream,
                            framer: FrameReader::new(),
                            out: VecDeque::new(),
                            registered: Interest::READABLE,
                            read_eof: false,
                            in_flight: 0,
                        },
                    );
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
    }

    /// worker 完成的回應:附掛到對應連線的 out、立即試 flush。
    fn route_outbox(&mut self) {
        let batch: Vec<(u64, Vec<u8>)> = std::mem::take(&mut *self.outbox.lock().unwrap());
        for (token, bytes) in batch {
            let Some(conn) = self.conns.get_mut(&token) else {
                continue; // 連線已死:回應無處可去,丟棄(doc 註明)
            };
            conn.in_flight -= 1;
            conn.out.extend(bytes);
            let dead = flush(conn);
            self.finish_conn_state(token, dead);
        }
    }

    /// 讀事件/寫事件驅動一條連線。
    fn drive_conn(&mut self, ev: Event) {
        let Some(conn) = self.conns.get_mut(&ev.token.0) else {
            return;
        };
        let mut dead = ev.error;

        if !dead && (ev.readable || ev.peer_closed) && !conn.read_eof {
            // 讀到 WouldBlock / EOF,bytes 進 framer
            let mut buf = [0u8; 4096];
            loop {
                match conn.stream.read(&mut buf) {
                    Ok(0) => {
                        conn.read_eof = true;
                        break;
                    }
                    Ok(n) => conn.framer.feed(&buf[..n]),
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(_) => {
                        dead = true;
                        break;
                    }
                }
            }
            // 切 frame → 派工。frame 邊界之外的殘料留在 framer 裡。
            while !dead {
                match conn.framer.next_frame() {
                    Ok(Some(frame)) => {
                        conn.in_flight += 1;
                        let handler = Arc::clone(&self.handler);
                        let outbox = Arc::clone(&self.outbox);
                        let wake = self.el.wake_handle();
                        let token = ev.token.0;
                        self.workers.execute(move || {
                            let resp = match Command::try_from_frame(&frame) {
                                Ok(cmd) => handler.lock().unwrap().handle(cmd),
                                Err(WireError::UnknownOpcode(_)) => Response::Error {
                                    code: ERR_UNKNOWN_OPCODE,
                                },
                                Err(WireError::BadPayloadLen { .. }) => Response::Error {
                                    code: ERR_BAD_PAYLOAD,
                                },
                            };
                            outbox.lock().unwrap().push((token, resp.encode()));
                            let _ = wake.wake(); // IO thread 可能睡在 epoll_wait
                        });
                    }
                    Ok(None) => break,
                    Err(_) => {
                        dead = true; // framing 損毀:關線
                    }
                }
            }
        }

        if !dead && ev.writable && !conn.out.is_empty() {
            dead = flush(conn);
        }
        self.finish_conn_state(ev.token.0, dead);
    }

    /// 統一的收尾:關線判定 + interest 狀態機(與 tcp_echo 同構)。
    fn finish_conn_state(&mut self, token: u64, dead: bool) {
        let Some(conn) = self.conns.get_mut(&token) else {
            return;
        };
        let finished = conn.read_eof && conn.out.is_empty() && conn.in_flight == 0;
        if dead || finished {
            let conn = self.conns.remove(&token).unwrap();
            let _ = self.el.deregister(conn.stream.as_raw_fd());
            return;
        }
        let want = Interest {
            readable: !conn.read_eof,
            writable: !conn.out.is_empty(),
        };
        if want != conn.registered {
            if self
                .el
                .reregister(conn.stream.as_raw_fd(), Token(token), want, Trigger::Level)
                .is_ok()
            {
                conn.registered = want;
            } else {
                let conn = self.conns.remove(&token).unwrap();
                let _ = self.el.deregister(conn.stream.as_raw_fd());
            }
        }
    }
}

/// flush 欠帳到 WouldBlock。true = 連線該關。與 tcp_echo::flush 同款。
fn flush(conn: &mut Conn) -> bool {
    while !conn.out.is_empty() {
        let (front, _) = conn.out.as_slices();
        match conn.stream.write(front) {
            Ok(0) => return false,
            Ok(n) => {
                conn.out.drain(..n);
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return false,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return true,
        }
    }
    false
}
