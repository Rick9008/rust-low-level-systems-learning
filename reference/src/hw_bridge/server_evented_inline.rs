//! ⚠️ 反面教材:event loop 裡 **inline 執行阻塞 handler**——故意寫壞的版本。
//!
//! event loop 的合約是「回呼絕不阻塞」:單一 IO thread 服務所有連線,
//! `handler.handle()` 裡的 sleep / 磁碟 / 下游 RPC 會讓**所有**連線的
//! read / write / accept 一起停擺整段延遲——cost-model 第五節的轉折點
//! 「CPU-bound / 阻塞任務會凍住 loop → offload」,這個檔案就是凍住的樣子。
//!
//! 可執行的病徵:mod.rs 的 `slow_handler_latency_contrast` 測試——
//! client B 的 Ping 陪 client A 的慢 ReadSensor 等整段 delay。
//!
//! 解法(依序):[`super::server_evented`](offload 到 command worker +
//! eventfd 回程)→ [`super::server_evented_sharded`](offload + 依連線
//! shard,跨連線平行)。
//!
//! 結構上它是 [`super::server_evented`] 砍掉 worker / outbox / in_flight
//! 的簡化版——正因為少了那三樣,它才會壞。

use super::framer::FrameReader;
use super::handler::CommandHandler;
use super::protocol::{Command, ERR_BAD_PAYLOAD, ERR_UNKNOWN_OPCODE, Response, WireError};
use crate::event_loop::{Event, EventLoop, Events, Interest, Token, Trigger, WakeHandle};
use std::collections::{HashMap, VecDeque};
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const LISTENER_TOKEN: Token = Token(0);

struct Conn {
    stream: TcpStream,
    framer: FrameReader,
    out: VecDeque<u8>,
    registered: Interest,
    read_eof: bool,
}

#[derive(Clone)]
pub struct InlineShutdown {
    stop: Arc<AtomicBool>,
    wake: WakeHandle,
}

impl InlineShutdown {
    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::Release);
        let _ = self.wake.wake();
    }
}

pub struct InlineServer<H: CommandHandler> {
    el: EventLoop,
    events: Events,
    listener: TcpListener,
    conns: HashMap<u64, Conn>,
    next_token: u64,
    /// 單執行緒,不需要 Arc/Mutex——這是唯一比 offload 版「乾淨」的地方,
    /// 也是陷阱的一部分:code 看起來更簡單,行為卻是全域凍結。
    handler: H,
    stop: Arc<AtomicBool>,
}

impl<H: CommandHandler> InlineServer<H> {
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
            handler,
            stop: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn shutdown_handle(&self) -> InlineShutdown {
        InlineShutdown {
            stop: Arc::clone(&self.stop),
            wake: self.el.wake_handle(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        while !self.stop.load(Ordering::Acquire) {
            self.el.poll(&mut self.events, None)?;
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
                        },
                    );
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
    }

    fn drive_conn(&mut self, ev: Event) {
        let Some(conn) = self.conns.get_mut(&ev.token.0) else {
            return;
        };
        let mut dead = ev.error;

        if !dead && (ev.readable || ev.peer_closed) && !conn.read_eof {
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
            while !dead {
                match conn.framer.next_frame() {
                    Ok(Some(frame)) => {
                        // ⚠️ 病灶在這一行:handler 在 IO thread 上同步執行。
                        // 它 sleep 300ms,這個 loop(= 所有連線)就凍 300ms。
                        let resp = match Command::try_from_frame(&frame) {
                            Ok(cmd) => self.handler.handle(cmd),
                            Err(WireError::UnknownOpcode(_)) => Response::Error {
                                code: ERR_UNKNOWN_OPCODE,
                            },
                            Err(WireError::BadPayloadLen { .. }) => Response::Error {
                                code: ERR_BAD_PAYLOAD,
                            },
                        };
                        conn.out.extend(resp.encode());
                    }
                    Ok(None) => break,
                    Err(_) => {
                        dead = true;
                    }
                }
            }
            if !dead && !conn.out.is_empty() {
                dead = flush(conn);
            }
        }

        if !dead && ev.writable && !conn.out.is_empty() {
            dead = flush(conn);
        }
        self.finish_conn_state(ev.token.0, dead);
    }

    fn finish_conn_state(&mut self, token: u64, dead: bool) {
        let Some(conn) = self.conns.get_mut(&token) else {
            return;
        };
        let finished = conn.read_eof && conn.out.is_empty();
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

/// 與 server_evented::flush 同款。
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
