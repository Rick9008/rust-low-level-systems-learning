//! 可執行的 hw_bridge server:把 reference 的兩種並發模型跑起來,讓你親手打進去。
//!
//! ```sh
//! cargo run -p reference --example hw_bridge_server              # thread-per-conn(預設)
//! cargo run -p reference --example hw_bridge_server -- --evented # event loop
//! cargo run -p reference --example hw_bridge_server -- --evented 0.0.0.0:9000
//! ```
//!
//! Ctrl-C 結束(std-only:不裝 signal handler crate,由 OS 直接收掉 process)。
//!
//! 值得親眼看的三件事(配合 `--example hw_bridge_client` 的子命令):
//! 1. `drip` —— 一次一 byte 送半個 frame,server 這邊**安靜等待**,
//!    直到最後一 byte 到齊才吐出一行 log。FrameReader 的狀態機是活的。
//! 2. `badop` —— 未知 opcode 回 Error frame,**連線不死**(語意錯可恢復)。
//! 3. `badlen` —— len 欄位胡說八道,server **直接斷線**(framing 錯無法 resync)。
//!
//! 兩種 mode 對同一組操作的外顯行為完全相同——差別只在 `top -H` 看得到的
//! thread 數(threaded 每連線一條;evented 固定 2 條:event loop + command worker)。

use reference::hw_bridge::handler::{CommandHandler, MockHardware};
use reference::hw_bridge::protocol::{Command, Response};
use reference::hw_bridge::server_evented::EventedServer;
use reference::hw_bridge::server_threaded::ThreadedServer;
use std::env;
use std::io;
use std::process;

/// 會說話的假硬體:decorator 包住任何 `CommandHandler`,把命令流印出來。
///
/// library 一行不動——這正是 `CommandHandler` trait 存在的理由:
/// server 只認 trait,不認硬體是真的、假的、還是多嘴的。
struct LoggingHardware<H> {
    inner: H,
    seq: u64,
}

impl<H: CommandHandler> CommandHandler for LoggingHardware<H> {
    fn handle(&mut self, cmd: Command) -> Response {
        self.seq += 1;
        let resp = self.inner.handle(cmd); // Command 是 Copy,先轉發再印不用 clone
        println!("  #{:<4} {:?}  ->  {:?}", self.seq, cmd, resp);
        resp
    }
}

enum Mode {
    Threaded,
    Evented,
}

fn main() {
    let mut mode = Mode::Threaded;
    let mut addr = String::from("127.0.0.1:9000");

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--threaded" => mode = Mode::Threaded,
            "--evented" => mode = Mode::Evented,
            "-h" | "--help" => {
                usage();
                return;
            }
            other if other.starts_with('-') => {
                eprintln!("unknown flag: {other}");
                usage();
                process::exit(2);
            }
            other => addr = other.to_string(),
        }
    }

    let hw = LoggingHardware {
        inner: MockHardware::default(),
        seq: 0,
    };

    if let Err(e) = serve(mode, &addr, hw) {
        eprintln!("server error: {e}");
        process::exit(1);
    }
}

fn serve<H: CommandHandler + 'static>(mode: Mode, addr: &str, hw: H) -> io::Result<()> {
    match mode {
        Mode::Threaded => {
            let mut server = ThreadedServer::bind(addr, hw)?;
            banner("threaded (thread-per-connection)", server.local_addr()?);
            server.run()
        }
        Mode::Evented => {
            let mut server = EventedServer::bind(addr, hw)?;
            banner("evented (epoll + command worker)", server.local_addr()?);
            server.run()
        }
    }
}

fn banner(mode: &str, addr: std::net::SocketAddr) {
    println!("hw_bridge server  mode={mode}  listening on {addr}");
    println!("wire format: [u32 len(BE)][u8 opcode][payload]");
    println!();
    println!("打進來:");
    println!("  cargo run -p reference --example hw_bridge_client -- --addr {addr} demo");
    println!("  cargo run -p reference --example hw_bridge_client -- --addr {addr} drip");
    println!();
    println!("命令流(Ctrl-C 結束):");
}

fn usage() {
    eprintln!("usage: hw_bridge_server [--threaded|--evented] [ADDR]");
    eprintln!("  --threaded   thread-per-connection(預設)");
    eprintln!("  --evented    epoll event loop + 單一 command worker");
    eprintln!("  ADDR         預設 127.0.0.1:9000");
}
