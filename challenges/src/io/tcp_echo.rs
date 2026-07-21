//! challenge:nonblocking TCP echo server
//!
//! 【題目】在 event loop 上寫一個單執行緒 echo server:
//! 收到什麼吐回什麼,同時伺服多個連線。
//! **epoll 綁定與 event loop 已提供**(`reference::io::event_loop`,
//! mio 形狀:register/poll/token dispatch)——你只從頭寫 server 本體:
//! accept 迴圈、read/write 迴圈、連線生命週期。
//!
//! 【constraints】
//! - 單執行緒(run 之內不准 spawn);所有 fd nonblocking
//! - write 塞住(WouldBlock)不可丟資料、也不可阻塞迴圈——
//!   想清楚資料放哪、什麼時候再寫
//! - LT 模式下不可 busy loop(interest 的掛法是考點)
//! - 對端斷線要清乾淨,server 要能繼續服務其他連線
//!
//! 【clarify points——動手前先自答】
//! - 一次 readable 事件,accept/read 要做幾次?停在什麼條件上?
//! - 常駐監聽 WRITABLE 會發生什麼?那什麼時候才監聽?
//! - EOF(read 回 0)時,如果還有沒寫完的資料怎麼辦?
//!
//! 【要實作】下方簽名。struct 內部自己設計。
//! 【驗收】tests/tcp_echo.rs 轉綠(含 1MB 灌流逼出部分寫路徑)。

use std::io;
use std::net::SocketAddr;

pub struct EchoServer {
    // ↓ 佔位:動手時整個換成你的設計。
    _todo: (),
}

/// 可跨執行緒叫停 run() 的把手(提示:event loop 有 wake_handle)。
#[derive(Clone)]
pub struct ShutdownHandle {
    _todo: (),
}

impl ShutdownHandle {
    pub fn shutdown(&self) {
        todo!("challenge")
    }
}

impl EchoServer {
    pub fn bind(addr: &str) -> io::Result<Self> {
        todo!("challenge: 從空白開始")
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        todo!("challenge")
    }

    pub fn shutdown_handle(&self) -> ShutdownHandle {
        todo!("challenge")
    }

    /// 事件迴圈:直到 shutdown 才返回。
    pub fn run(&mut self) -> io::Result<()> {
        todo!("challenge")
    }
}
