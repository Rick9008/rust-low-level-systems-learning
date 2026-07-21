//! thread-per-connection server:45 分鐘面試的首選形狀。
//!
//! 結構:main thread 阻塞 accept;每條連線一條 thread(阻塞 read/write,
//! 各自一個 FrameReader);硬體 handler 用 `Arc<Mutex>` 共享——
//! 硬體是一台序列設備,命令天然要序列化,鎖不是妥協,是語意。
//!
//! 「1 thread/conn 不 scale」:每條 thread 預設 8MB stack 位址空間 +
//! ~10μs context switch;萬條連線 = 萬條 thread,排程器先跪。
//! 但在連線數 ≤ 數百的硬體控制場景,這是**正確的**選擇:
//! code 直線、錯誤處理直觀、無 interest 狀態機。scale 的答案見
//! server_evented.rs。

use super::framer::FrameReader;
use super::handler::CommandHandler;
use super::protocol::{Command, ERR_BAD_PAYLOAD, ERR_UNKNOWN_OPCODE, Response, WireError};
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct ThreadedServer<H: CommandHandler + 'static> {
    listener: TcpListener,
    handler: Arc<Mutex<H>>,
    stop: Arc<AtomicBool>,
}

/// 停機把手:設旗標 + 對自己 connect 一下,把阻塞中的 accept 戳醒。
/// (std-only 下沒有「可中斷的阻塞 accept」;這是老實而常見的解法。
/// evented server 的 eventfd wake 是這件事的正規版。)
#[derive(Clone)]
pub struct ThreadedShutdown {
    stop: Arc<AtomicBool>,
    addr: SocketAddr,
}

impl ThreadedShutdown {
    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::Release);
        let _ = TcpStream::connect(self.addr); // 戳醒 accept;連上即棄
    }
}

impl<H: CommandHandler + 'static> ThreadedServer<H> {
    pub fn bind(addr: &str, handler: H) -> io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(addr)?,
            handler: Arc::new(Mutex::new(handler)),
            stop: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn shutdown_handle(&self) -> io::Result<ThreadedShutdown> {
        Ok(ThreadedShutdown {
            stop: Arc::clone(&self.stop),
            addr: self.listener.local_addr()?,
        })
    }

    /// accept 迴圈。返回時所有連線 thread 已 join(不遺留殭屍)。
    pub fn run(&mut self) -> io::Result<()> {
        let mut conn_threads = Vec::new();
        loop {
            let (stream, _) = self.listener.accept()?; // 阻塞:被 shutdown 的 poke 戳醒
            if self.stop.load(Ordering::Acquire) {
                break; // poke 連線本身直接丟棄
            }
            let handler = Arc::clone(&self.handler);
            conn_threads.push(thread::spawn(move || {
                // 連線級錯誤(斷線、framing 損毀)只影響這條線:吞掉、thread 結束。
                let _ = serve_conn(stream, &handler);
            }));
        }
        for t in conn_threads {
            let _ = t.join();
        }
        Ok(())
    }
}

/// 單一連線的完整生命週期:read → feed → 切 frame → 執行 → 回寫。
/// 阻塞 IO 在 per-conn thread 裡是合法的——這正是此模型的簡單所在。
fn serve_conn<H: CommandHandler>(mut stream: TcpStream, handler: &Arc<Mutex<H>>) -> io::Result<()> {
    let mut reader = FrameReader::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            return Ok(()); // EOF:對端說完了
        }
        reader.feed(&buf[..n]);
        // loop 到切不動為止:一次 read 可能含多個 frame(見 framer.rs)
        loop {
            match reader.next_frame() {
                Ok(Some(frame)) => {
                    let resp = match Command::try_from_frame(&frame) {
                        // 鎖區間 = 一條命令:硬體序列化的最小粒度
                        Ok(cmd) => handler.lock().unwrap().handle(cmd),
                        // 語意錯誤可恢復:回 Error frame,連線續命
                        Err(WireError::UnknownOpcode(_)) => Response::Error {
                            code: ERR_UNKNOWN_OPCODE,
                        },
                        Err(WireError::BadPayloadLen { .. }) => Response::Error {
                            code: ERR_BAD_PAYLOAD,
                        },
                    };
                    stream.write_all(&resp.encode())?;
                }
                Ok(None) => break, // 殘料不足一個 frame:回去等下一次 read
                // framing 損毀:byte 流失去同步,唯一正解是關線
                Err(_) => return Ok(()),
            }
        }
    }
}
