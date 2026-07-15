//! solution:題 d tokio_frame_server——**寫完彩排才開**。
//! canonical 設計:accept loop + 每連線一個 task;`tokio::time::timeout` 包 read
//! 做 idle timeout(任何 bytes——含 heartbeat——都重置計時)。
//! 驗證:rehearsals/tests/tokio_frame_server_test.rs 全綠。

use std::io;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn serve(listener: TcpListener, idle_timeout: Duration) -> io::Result<()> {
    loop {
        let (sock, _) = listener.accept().await?;
        tokio::spawn(handle_conn(sock, idle_timeout));
    }
}

async fn handle_conn(mut sock: TcpStream, idle_timeout: Duration) {
    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let n = match tokio::time::timeout(idle_timeout, sock.read(&mut chunk)).await {
            Err(_) => return,    // idle timeout → drop socket = 關閉連線
            Ok(Ok(0)) => return, // peer 關閉
            Ok(Ok(n)) => n,
            Ok(Err(_)) => return,
        };
        buf.extend_from_slice(&chunk[..n]);
        loop {
            if buf.len() < 4 {
                break;
            }
            let len = u32::from_be_bytes(buf[..4].try_into().unwrap()) as usize;
            if buf.len() < 4 + len {
                break;
            }
            let payload = buf[4..4 + len].to_vec();
            buf.drain(..4 + len);
            if len > 0 {
                let mut resp = (payload.len() as u32).to_be_bytes().to_vec();
                resp.extend_from_slice(&payload);
                if sock.write_all(&resp).await.is_err() {
                    return;
                }
            }
            // heartbeat(len == 0):不回應;read 本身已重置 idle 計時
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(serve(listener, Duration::from_millis(400)));

    let mut c = TcpStream::connect(addr).await.unwrap();
    let mut req = 3u32.to_be_bytes().to_vec();
    req.extend_from_slice(b"abc");
    c.write_all(&req).await.unwrap();

    let mut len_buf = [0u8; 4];
    c.read_exact(&mut len_buf).await.unwrap();
    let mut payload = vec![0u8; u32::from_be_bytes(len_buf) as usize];
    c.read_exact(&mut payload).await.unwrap();
    assert_eq!(payload, b"abc");
    println!("sol_tokio_frame_server: ok");
}
