//! sharded offload server:跨連線平行,同連線保序。
//!
//! [`super::server_evented`] 用 **1 條** command worker——因為協定沒有
//! request-id,client 靠 FIFO 對應回應,多 worker 會讓**同連線**的命令
//! 亂序完成。代價:worker 是全域瓶頸,連線 A 的慢命令會讓連線 B 的命令
//! 在 worker 佇列裡陪排(IO thread 沒凍,但延遲照樣傳染)。
//!
//! 觀察:保序約束其實只有**同連線**才需要——跨連線本來就沒有順序語意。
//! 所以正確的平行單位是連線:**shard by connection**。
//!
//! ```text
//! conn token % N  →  shard i:ThreadPool(1) + 自己的 handler 實例
//! ```
//!
//! - 同連線 → 永遠同 shard → 單 worker FIFO,保序不變;
//! - 不同連線 → 大概率不同 shard → 慢命令不再傳染(可執行證據:
//!   mod.rs 的 `slow_handler_latency_contrast`)。
//!
//! **前提(誠實邊界)**:每 shard 要有**自己的下游通道**(bind 收
//! `Vec<H>`,一 shard 一實例)。如果下游只有一顆硬體(單一序列設備),
//! shard 毫無意義——所有 worker 還是在同一把 Mutex 上排隊。這正是
//! clarify 五問裡 Q3(幾個 producer)要連著「下游長什麼樣」一起問的原因;
//! 對單顆硬體,server_evented 的單 worker 就是正解,不是偷懶。
//!
//! 這是 clarify-playbook「per-producer shard」在 server 側的鏡像:
//! 那邊 shard 的是 queue,這邊 shard 的是 worker + 下游通道。

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
    in_flight: usize,
}

/// 一個 shard = 一條 worker + 一個獨立的下游通道(handler 實例)。
/// pool 固定 1 條 thread:shard 內 FIFO 就是保序機制本身。
struct Shard<H> {
    pool: ThreadPool,
    handler: Arc<Mutex<H>>,
}

type Outbox = Arc<Mutex<Vec<(u64, Vec<u8>)>>>;

#[derive(Clone)]
pub struct ShardedShutdown {
    stop: Arc<AtomicBool>,
    wake: WakeHandle,
}

impl ShardedShutdown {
    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::Release);
        let _ = self.wake.wake();
    }
}

pub struct ShardedServer<H: CommandHandler + 'static> {
    el: EventLoop,
    events: Events,
    listener: TcpListener,
    conns: HashMap<u64, Conn>,
    next_token: u64,
    shards: Vec<Shard<H>>,
    outbox: Outbox,
    stop: Arc<AtomicBool>,
}

impl<H: CommandHandler + 'static> ShardedServer<H> {
    /// `handlers.len()` = shard 數;一 shard 一個獨立 handler 實例
    /// (= 一條獨立的下游通道)。
    pub fn bind(addr: &str, handlers: Vec<H>) -> io::Result<Self> {
        assert!(!handlers.is_empty(), "至少一個 shard");
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let el = EventLoop::new()?;
        el.register(
            listener.as_raw_fd(),
            LISTENER_TOKEN,
            Interest::READABLE,
            Trigger::Level,
        )?;
        let shards = handlers
            .into_iter()
            .map(|h| Shard {
                pool: ThreadPool::new(1),
                handler: Arc::new(Mutex::new(h)),
            })
            .collect();
        Ok(Self {
            el,
            events: Events::with_capacity(64),
            listener,
            conns: HashMap::new(),
            next_token: 1,
            shards,
            outbox: Arc::new(Mutex::new(Vec::new())),
            stop: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn shutdown_handle(&self) -> ShardedShutdown {
        ShardedShutdown {
            stop: Arc::clone(&self.stop),
            wake: self.el.wake_handle(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        while !self.stop.load(Ordering::Acquire) {
            self.el.poll(&mut self.events, None)?;
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

    fn route_outbox(&mut self) {
        let batch: Vec<(u64, Vec<u8>)> = std::mem::take(&mut *self.outbox.lock().unwrap());
        for (token, bytes) in batch {
            let Some(conn) = self.conns.get_mut(&token) else {
                continue; // 連線已死:回應無處可去,丟棄
            };
            conn.in_flight -= 1;
            conn.out.extend(bytes);
            let dead = flush(conn);
            self.finish_conn_state(token, dead);
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
                        conn.in_flight += 1;
                        let token = ev.token.0;
                        // 平行的關鍵一行:同連線永遠落在同一 shard(保序),
                        // 不同連線分散到不同 shard(隔離)。
                        let shard = &self.shards[(token as usize) % self.shards.len()];
                        let handler = Arc::clone(&shard.handler);
                        let outbox = Arc::clone(&self.outbox);
                        let wake = self.el.wake_handle();
                        shard.pool.execute(move || {
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
                            let _ = wake.wake();
                        });
                    }
                    Ok(None) => break,
                    Err(_) => {
                        dead = true;
                    }
                }
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
