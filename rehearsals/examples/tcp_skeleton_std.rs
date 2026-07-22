//! TCP 骨架默寫卡(答案版)——**先讀一遍,之後只准默寫再回來對答案**。
//! 排程:7/25 開場讀+默寫 10m(d#1 前置)→ 7/26 d-std 前重默 5m → 7/27 抽查。
//!
//! std 六行肌肉(默寫目標,順序就是故事):
//! 1. `TcpListener::bind(addr)?`(`"127.0.0.1:0"` 的 0 = 要臨時 port,測試用)
//! 2. `for stream in listener.incoming()`(阻塞 accept 的迭代器皮)
//! 3. `thread::spawn(move || ...)`(thread-per-connection;千連線以上才換 event loop)
//! 4. `stream.read(&mut buf)` 迴圈——**要 `use std::io::{Read, Write};`**,
//!    忘了 import trait 是這裡最常見的手滑,錯誤訊息還很難看懂
//! 5. `Ok(0)` = EOF(對端關閉)——**不是錯誤**,是正常收尾
//! 6. `write_all(&buf[..n])`——`write` 可能只寫一半,echo 一律 `write_all`
//!
//! tokio 對照(d#1 的肌肉,逐行同構):bind/connect/accept/read/write 全部
//! 補 `.await`;`thread::spawn` → `tokio::spawn(async move { ... })`;
//! trait import → `use tokio::io::{AsyncReadExt, AsyncWriteExt};`。
//!
//! 自驗跑法:`cargo run -p rehearsals --example tcp_skeleton_std`
//! (std 與 tokio 各做一次 echo roundtrip,assert 通過會各印一行 ✓)。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// std 版 echo server:六行肌肉的本體。`incoming()` 出錯的連線跳過即可
/// (accept 失敗多半是暫時性的,server 不該為它死)。
fn serve_std(listener: TcpListener) {
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match stream.read(&mut buf) {
                    Ok(0) => break, // EOF:對端關了,正常收尾
                    Ok(n) => {
                        if stream.write_all(&buf[..n]).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
}

/// tokio 版:與 std 版逐行對照,差異只有 `.await`、`tokio::spawn`、trait 名。
async fn serve_tokio(listener: tokio::net::TcpListener) {
    loop {
        let Ok((mut stream, _peer)) = listener.accept().await else {
            break;
        };
        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                match stream.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if stream.write_all(&buf[..n]).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
}

fn main() {
    // ---- std 段:bind 臨時 port → server 進背景執行緒 → client roundtrip ----
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    thread::spawn(move || serve_std(listener));

    let mut client = TcpStream::connect(addr).expect("connect");
    client.write_all(b"ping").expect("write");
    let mut back = [0u8; 4];
    client.read_exact(&mut back).expect("read");
    assert_eq!(&back, b"ping");
    println!("std echo roundtrip ✓");

    // ---- tokio 段:同一個故事,補上 .await ----
    let rt = tokio::runtime::Runtime::new().expect("rt");
    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("local_addr");
        tokio::spawn(serve_tokio(listener));

        let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
        client.write_all(b"pong").await.expect("write");
        let mut back = [0u8; 4];
        client.read_exact(&mut back).await.expect("read");
        assert_eq!(&back, b"pong");
        println!("tokio echo roundtrip ✓");
    });
    // main 返回 = process 結束,背景 server 執行緒一併收掉(daemon 語意)。
}
