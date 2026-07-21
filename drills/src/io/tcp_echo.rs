//! drill:tcp_echo —— 填 nonblocking IO 的三個核心決策。
//!
//! 已給:server 骨架(accept、dispatch、生命週期管理,用 reference 的
//! event_loop 當底)。要填:
//! - `write_some`:盡力寫,WouldBlock 不算錯
//! - `flush`:清欠帳
//! - `desired_interest`:interest 狀態機(WRITABLE 用完即拆,否則 busy loop)
//!
//! 填之前紙上回答:LT 模式下常駐 WRITABLE 會發生什麼?

use reference::io::event_loop::{Event, EventLoop, Events, Interest, Token, Trigger, WakeHandle};
use std::collections::{HashMap, VecDeque};
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const LISTENER_TOKEN: Token = Token(0);

struct Conn {
    stream: TcpStream,
    out: VecDeque<u8>,
    read_eof: bool,
    registered: Interest,
}

/// spec:盡力寫 data,回 Ok(實際寫出的 bytes)。
/// - `stream.write` 回 Ok(n):累計,繼續寫剩的
/// - Ok(0):防禦性跳出(TCP 理論上不回 0)
/// - WouldBlock:跳出,回 Ok(目前累計)——**不是錯誤**,是「送緩衝滿了」
/// - Interrupted:continue 重試
/// - 其他錯誤:Err 上拋(連線該關)
fn write_some(stream: &mut TcpStream, data: &[u8]) -> io::Result<usize> {
    todo!("spec: 迴圈寫;WouldBlock 停;回寫出總量")
}

/// spec:flush 欠帳到 WouldBlock。回 true = 連線該關(致命錯誤)。
/// 提示:`conn.out.as_slices().0` 拿環形佇列的連續前段;
/// 寫出 n bytes 就 `conn.out.drain(..n)`;write_some 回 Ok(0) 表示塞住,停。
fn flush(conn: &mut Conn) -> bool {
    todo!("spec: while 有欠帳 {{ write_some 前段; 0 → 停; Err → true }}; false")
}

/// spec:根據連線狀態算出應註冊的 interest。
/// - readable:還沒收到 EOF 就聽
/// - writable:**只在有欠帳時**聽(常駐 WRITABLE = LT 每輪都報 = busy loop)
fn desired_interest(conn: &Conn) -> Interest {
    todo!("spec: readable = !read_eof; writable = !out.is_empty()")
}

// ---------- 以下骨架已給,讀懂即可 ----------

#[derive(Clone)]
pub struct ShutdownHandle {
    stop: Arc<AtomicBool>,
    wake: WakeHandle,
}

impl ShutdownHandle {
    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::Release);
        let _ = self.wake.wake();
    }
}

pub struct EchoServer {
    el: EventLoop,
    events: Events,
    listener: TcpListener,
    conns: HashMap<u64, Conn>,
    next_token: u64,
    stop: Arc<AtomicBool>,
}

impl EchoServer {
    pub fn bind(addr: &str) -> io::Result<Self> {
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
            stop: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn shutdown_handle(&self) -> ShutdownHandle {
        ShutdownHandle {
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
                            out: VecDeque::new(),
                            read_eof: false,
                            registered: Interest::READABLE,
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
                    Ok(n) => {
                        let chunk = &buf[..n];
                        if conn.out.is_empty() {
                            match write_some(&mut conn.stream, chunk) {
                                Ok(written) => conn.out.extend(&chunk[written..]),
                                Err(_) => {
                                    dead = true;
                                    break;
                                }
                            }
                        } else {
                            conn.out.extend(chunk);
                        }
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(_) => {
                        dead = true;
                        break;
                    }
                }
            }
        }
        if !dead && ev.writable && !conn.out.is_empty() {
            dead = flush(conn);
        }
        if dead || (conn.read_eof && conn.out.is_empty()) {
            let conn = self.conns.remove(&ev.token.0).unwrap();
            let _ = self.el.deregister(conn.stream.as_raw_fd());
            return;
        }
        let want = desired_interest(conn);
        if want != conn.registered {
            if self
                .el
                .reregister(conn.stream.as_raw_fd(), ev.token, want, Trigger::Level)
                .is_ok()
            {
                conn.registered = want;
            } else {
                let conn = self.conns.remove(&ev.token.0).unwrap();
                let _ = self.el.deregister(conn.stream.as_raw_fd());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    fn spawn_server() -> (
        SocketAddr,
        ShutdownHandle,
        thread::JoinHandle<io::Result<()>>,
    ) {
        let mut server = EchoServer::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();
        let handle = server.shutdown_handle();
        let join = thread::spawn(move || server.run());
        (addr, handle, join)
    }

    /// boundary:單客戶端 echo roundtrip。
    #[test]
    #[ignore = "填完三個函式後移除"]
    fn echo_roundtrip() {
        let (addr, shutdown, join) = spawn_server();
        let mut c = TcpStream::connect(addr).unwrap();
        c.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        c.write_all(b"hello").unwrap();
        let mut buf = [0u8; 5];
        c.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"hello");
        shutdown.shutdown();
        join.join().unwrap().unwrap();
    }

    /// boundary:1MB 灌流(先不讀)→ 逼出 WouldBlock + 欠帳 + EPOLLOUT 路徑。
    #[test]
    #[ignore = "填完三個函式後移除"]
    fn large_transfer_partial_writes() {
        let (addr, shutdown, join) = spawn_server();
        const N: usize = 1 << 20;
        let data: Vec<u8> = (0..N).map(|i| (i % 251) as u8).collect();
        let mut c = TcpStream::connect(addr).unwrap();
        c.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut c_read = c.try_clone().unwrap();
        let writer = thread::spawn(move || {
            c.write_all(&data).unwrap();
            data
        });
        let mut got = vec![0u8; N];
        c_read.read_exact(&mut got).unwrap();
        assert_eq!(got, writer.join().unwrap());
        shutdown.shutdown();
        join.join().unwrap().unwrap();
    }
}
