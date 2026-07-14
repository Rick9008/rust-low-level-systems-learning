//! 可執行的 epoll echo server:一條 thread、多條連線。
//!
//! ```sh
//! cargo run -p reference --example tcp_echo_server            # 127.0.0.1:9001
//! cargo run -p reference --example tcp_echo_server -- 0.0.0.0:7000
//! ```
//!
//! 開幾個 terminal 打進來:
//! ```sh
//! nc 127.0.0.1 9001            # 打字 → 立刻 echo 回來
//! ```
//!
//! ## 值得親眼看的兩件事
//!
//! **1. 單執行緒服務多連線。** 開三四個 `nc`,每個都通。`top -H -p $(pgrep tcp_echo)`
//!    看到的 thread 數不會跟著連線數長——這就是 readiness model 的全部意義:
//!    不是「一條 thread 等一條連線」,而是「一條 thread 等**任何一條**連線有事」。
//!
//! **2. backpressure。** 灌一大坨資料進去、同時讓自己讀得很慢:
//!    ```sh
//!    yes "$(head -c 4000 /dev/zero | tr '\0' 'x')" | nc 127.0.0.1 9001 | pv -qL 1000 > /dev/null
//!    ```
//!    server 的 write 會塞住(`EWOULDBLOCK`),此時它**不能忙等、也不能丟資料**:
//!    把寫不完的 bytes 緩存起來、把該連線的 interest 改成 `EPOLLOUT`,
//!    等 kernel 說「可以寫了」再回來把欠帳寫完。這段狀態機是 tcp_echo.rs 的重點,
//!    也是所有 async runtime 內部都在做的事。
//!
//! Ctrl-C 結束。

use reference::tcp_echo::EchoServer;
use std::env;
use std::process;

fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:9001".into());

    let mut server = match EchoServer::bind(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("bind {addr} failed: {e}");
            process::exit(1);
        }
    };

    let bound = server.local_addr().unwrap_or_else(|e| {
        eprintln!("local_addr failed: {e}");
        process::exit(1);
    });

    println!("epoll echo server listening on {bound}  (單執行緒,Ctrl-C 結束)");
    println!();
    println!("  nc {} {}", bound.ip(), bound.port());
    println!();
    println!("開多個 nc 同時連——thread 數不會跟著長。");

    if let Err(e) = server.run() {
        eprintln!("run error: {e}");
        process::exit(1);
    }
}
