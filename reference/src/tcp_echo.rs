//! # tcp_echo —— event loop 上的 nonblocking TCP echo server
//!
//! ## [Clarify]
//! 解決:單執行緒、event_loop 驅動的 echo 服務——「readiness model 下
//! 一條連線的完整生命週期」的最小完整範例:accept → 讀 → 寫(可能寫不完)
//! → 緩衝 + EPOLLOUT → 對端關閉 → 清理。
//! Constraints:std::net(nonblocking)+ 自家 event_loop;LT 模式
//! (ET 變體的紀律差異見 docs);echo 語意 = 收什麼吐什麼、順序不變。
//!
//! ## [Abstract]
//! 「協定」在這裡就是 echo(零解析)——framing/協定處理是 hw_bridge 的主題,
//! 面試時先 echo 打通 IO 路徑再疊協定,是正確的增量順序。
//!
//! ## [Trade-offs]——本模組的三個核心決策
//! 1. **write 塞住 → 緩存 + EPOLLOUT**:kernel 送緩衝滿時 write 回 WouldBlock,
//!    剩餘 bytes 進 per-conn `VecDeque<u8>`,並把 interest 加上 WRITABLE;
//!    可寫事件來了再 flush。天真的「迴圈寫到完」會阻塞整個 event loop。
//! 2. **WRITABLE 用完即拆**:LT 下 socket 幾乎永遠可寫,常掛 WRITABLE
//!    ⇒ 每輪 poll 都報 ⇒ busy loop 空轉 100% CPU。interest 是狀態機:
//!    「有欠帳才聽可寫」。
//! 3. **out 緩衝無上限**(教學從簡):慢消費者會把 server 記憶體吃爆。
//!    production 要有高水位:超過即暫停讀該連線(拿掉 READABLE)——
//!    backpressure 沿 TCP 往回傳。doc 註明,不實作。
//!
//! ## [Dry-Run]
//! 測試:單客戶端 roundtrip、多客戶端交錯、1MB 大傳輸(逼出部分寫 +
//! EPOLLOUT 路徑)、客戶端斷線清理、shutdown。
//!
//! Production 對照:tokio + TcpStream(readiness 藏進 async)、mio 範例集。

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
    /// 欠帳:還沒送出去的 echo bytes。空間 O(欠帳量),見 Trade-off 3。
    out: VecDeque<u8>,
    /// 對端已送 EOF(read 回 0):不再讀,flush 完欠帳就關。
    read_eof: bool,
    /// 目前掛在 epoll 上的 interest(避免每輪無腦 reregister 的 syscall)。
    registered: Interest,
}

/// 讓 `run()` 停下來的把手(Clone + Send):設旗標 + wake。
#[derive(Clone)]
pub struct ShutdownHandle {
    stop: Arc<AtomicBool>,
    wake: WakeHandle,
}

impl ShutdownHandle {
    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::Release);
        let _ = self.wake.wake(); // 迴圈可能睡在 epoll_wait,叫醒它看旗標
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
        // 鐵律:event loop 裡的每個 fd 都必須 nonblocking——
        // 阻塞的 accept/read/write 會凍住所有其他連線。
        listener.set_nonblocking(true)?;
        let el = EventLoop::new()?;
        el.register(
            listener.as_raw_fd(),
            LISTENER_TOKEN,
            Interest::READABLE, // listener 的「可讀」= 有連線可 accept
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

    /// 事件迴圈本體:poll → dispatch(match token)→ 更新每條連線的狀態機。
    pub fn run(&mut self) -> io::Result<()> {
        while !self.stop.load(Ordering::Acquire) {
            self.el.poll(&mut self.events, None)?;
            // dispatch:事件只帶 token,狀態在 conns map——O(1) 找回連線。
            // 先收集 token 再處理:處理中會 &mut self(關連線、改註冊)。
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

    /// accept 到 WouldBlock 為止:一次 readable 事件可能積壓多個連線
    /// (LT 下漏掉也會再報,但一次收完省 poll 輪次;ET 下不收完就是 bug)。
    fn accept_all(&mut self) -> io::Result<()> {
        loop {
            match self.listener.accept() {
                Ok((stream, _peer)) => {
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
                // EMFILE(fd 用完)等:真 server 要有對策(保留 fd、拒接、退避);
                // 教學實作如實上拋。
                Err(e) => return Err(e),
            }
        }
    }

    /// 單一連線的狀態推進。任何 IO 錯誤 → 關連線(echo 沒有可恢復錯誤)。
    fn drive_conn(&mut self, ev: Event) {
        let Some(conn) = self.conns.get_mut(&ev.token.0) else {
            return; // 同批事件裡已被關掉(例:error+readable 兩事件)
        };
        let mut dead = ev.error;

        // 讀路徑:readable 或 peer_closed 都值得去讀
        // (peer 半關閉時緩衝裡可能還有最後一批資料)。
        if !dead && (ev.readable || ev.peer_closed) && !conn.read_eof {
            dead = Self::read_and_echo(conn);
        }
        // 寫路徑:flush 欠帳。
        if !dead && ev.writable && !conn.out.is_empty() {
            dead = Self::flush(conn);
        }
        // 生命週期:EOF 且無欠帳 → 優雅關閉。
        if dead || (conn.read_eof && conn.out.is_empty()) {
            let conn = self.conns.remove(&ev.token.0).unwrap();
            let _ = self.el.deregister(conn.stream.as_raw_fd());
            return; // conn drop → fd close
        }
        // interest 狀態機:讀著(除非 EOF)+ 只在有欠帳時聽可寫。
        let want = Interest {
            readable: !conn.read_eof,
            writable: !conn.out.is_empty(),
        };
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

    /// 讀到 WouldBlock / EOF 為止,立即嘗試回寫,寫不完掛帳。
    /// 回傳 true = 連線該關(IO 錯誤)。
    fn read_and_echo(conn: &mut Conn) -> bool {
        let mut buf = [0u8; 4096];
        loop {
            match conn.stream.read(&mut buf) {
                Ok(0) => {
                    conn.read_eof = true; // EOF:對端不再送(可能還欠它 echo)
                    return false;
                }
                Ok(n) => {
                    // 快路徑:沒有欠帳時直接寫,避免一次 memcpy;
                    // 寫剩的才進 out(慢路徑)。順序保證:有欠帳一律排隊。
                    let chunk = &buf[..n];
                    if conn.out.is_empty() {
                        match Self::write_some(&mut conn.stream, chunk) {
                            Ok(written) => conn.out.extend(&chunk[written..]),
                            Err(_) => return true,
                        }
                    } else {
                        conn.out.extend(chunk);
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return false, // 讀完了
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => return true, // ECONNRESET 等:關
            }
        }
    }

    /// flush 欠帳到 WouldBlock 為止。回傳 true = 連線該關。
    fn flush(conn: &mut Conn) -> bool {
        while !conn.out.is_empty() {
            // VecDeque 環形儲存:一次拿連續的前段 slice(最多兩段)。
            let (front, _) = conn.out.as_slices();
            match Self::write_some(&mut conn.stream, front) {
                Ok(0) => return false, // 沒進展(WouldBlock):等下一次可寫事件
                Ok(n) => {
                    conn.out.drain(..n);
                }
                Err(_) => return true,
            }
        }
        false
    }

    /// 盡力寫:回 Ok(寫出的 bytes;WouldBlock 算 0),Err = 致命錯誤。
    fn write_some(stream: &mut TcpStream, mut data: &[u8]) -> io::Result<usize> {
        let mut total = 0;
        while !data.is_empty() {
            match stream.write(data) {
                Ok(0) => break, // 理論上 TCP 不回 0;防禦性跳出
                Ok(n) => {
                    total += n;
                    data = &data[n..];
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break, // 送緩衝滿
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;
    use std::thread;
    use std::time::Duration;

    /// 起 server 於背景執行緒,回 (addr, shutdown, join)。
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

    /// [Dry-Run] 單客戶端 roundtrip:
    ///   connect → 事件(listener readable)→ accept、註冊 READABLE
    ///   write "hello" → 事件(conn readable)→ read 5 bytes → 立即回寫成功
    ///   client read → "hello"
    #[test]
    fn echo_roundtrip_single_client() {
        let (addr, shutdown, join) = spawn_server();
        let mut c = TcpStream::connect(addr).unwrap();
        c.write_all(b"hello").unwrap();
        let mut buf = [0u8; 5];
        c.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"hello");
        shutdown.shutdown();
        join.join().unwrap().unwrap();
    }

    /// 多客戶端交錯:3 條連線都開著,亂序互動——單執行緒 server
    /// 靠 token dispatch 同時伺服(這正是 event loop 的存在理由)。
    #[test]
    fn multiple_clients_interleaved() {
        let (addr, shutdown, join) = spawn_server();
        let mut clients: Vec<TcpStream> =
            (0..3).map(|_| TcpStream::connect(addr).unwrap()).collect();
        // 亂序寫
        for (i, c) in clients.iter_mut().enumerate().rev() {
            c.write_all(format!("msg-{i}").as_bytes()).unwrap();
        }
        // 亂序讀
        for (i, c) in clients.iter_mut().enumerate() {
            let mut buf = [0u8; 5];
            c.read_exact(&mut buf).unwrap();
            assert_eq!(buf, format!("msg-{i}").as_bytes());
        }
        shutdown.shutdown();
        join.join().unwrap().unwrap();
    }

    /// boundary:1MB 單向猛灌(客戶端先不讀)→ kernel 送緩衝必滿 →
    /// server write 遇 WouldBlock → 欠帳 + EPOLLOUT 路徑被逼出來。
    /// 之後客戶端把 1MB 讀回,逐 byte 驗序(部分寫的切割不能亂序/丟失)。
    #[test]
    fn boundary_large_transfer_exercises_partial_writes() {
        let (addr, shutdown, join) = spawn_server();
        const N: usize = 1 << 20; // 1 MiB
        let data: Vec<u8> = (0..N).map(|i| (i % 251) as u8).collect(); // 質數週期好抓錯位
        let mut c = TcpStream::connect(addr).unwrap();
        let mut c_read = c.try_clone().unwrap();
        // 寫和讀分兩條執行緒:全塞完才讀會讓 client 的 write 也可能塞住
        // (server 欠帳無上限,但 client 自己的送緩衝有限)。
        let writer = thread::spawn(move || {
            c.write_all(&data).unwrap();
            c.flush().unwrap();
            data
        });
        let mut got = vec![0u8; N];
        c_read.read_exact(&mut got).unwrap();
        let data = writer.join().unwrap();
        assert_eq!(got, data); // 內容與順序完全一致
        shutdown.shutdown();
        join.join().unwrap().unwrap();
    }

    /// boundary:客戶端斷線 → server 清掉連線,其他/後續客戶端不受影響。
    #[test]
    fn boundary_disconnect_cleans_up_and_server_lives_on() {
        let (addr, shutdown, join) = spawn_server();
        {
            let mut c1 = TcpStream::connect(addr).unwrap();
            c1.write_all(b"bye").unwrap();
            let mut buf = [0u8; 3];
            c1.read_exact(&mut buf).unwrap();
        } // c1 drop → FIN → server 走 EOF 清理路徑
        thread::sleep(Duration::from_millis(50)); // 讓清理跑完
        let mut c2 = TcpStream::connect(addr).unwrap(); // server 還活著
        c2.write_all(b"again").unwrap();
        let mut buf = [0u8; 5];
        c2.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"again");
        shutdown.shutdown();
        join.join().unwrap().unwrap();
    }

    /// boundary:shutdown 喚醒睡在 epoll_wait(無 timeout)的 run()。
    #[test]
    fn boundary_shutdown_wakes_idle_server() {
        let (_addr, shutdown, join) = spawn_server();
        thread::sleep(Duration::from_millis(30)); // server 已睡在 poll(None)
        shutdown.shutdown();
        join.join().unwrap().unwrap(); // 不 hang 就是過
    }
}
