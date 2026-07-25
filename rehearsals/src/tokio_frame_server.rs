//! rehearsal d:tokio_frame_server —— 題目見 rehearsals/README.md。
//!
//! 唯一用 crate 的一題(tokio;pad 實測清單有)。
//! wire format 同題目 c:`[u32 len(BE)][payload]`,`len == 0` 是 heartbeat。
//! 只給 API 簽名;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`
//! (`#[tokio::test]` 照用)。

/*
> **用途**:「面試官說可用 crate」那條分支的**保險**,只跑一遍(7/24);預設路線仍 std-only + 陳述假設。
> c 題 framer 的延伸(黏包邏輯直接重用)+ idle timeout / heartbeat 保活。

A device gateway: many devices connect over TCP, speaking the protocol from
problem c — `[u32 len (BE)][payload]`, `len == 0` is a heartbeat.

Requirements:

- Write the server with tokio: `serve(listener, idle_timeout)`, runs until
  the listener errors out.
- Connections are served concurrently and independently.
- Data frame → echo it back unchanged (same wire format).
- Heartbeat → no response.
- If a connection goes more than `idle_timeout` without **any** bytes
  arriving, close it. Heartbeats count as traffic — that's what keeps an
  idle device's connection alive; that's why they exist.
- TCP still has no message boundaries — reuse your problem-c homework.
*/

#[cfg(test)]
use crate::frame_parser_heartbeat::FrameParser;

use super::frame_parser_heartbeat;
use std::io;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
#[cfg(test)]
use tokio::net::TcpStream;

/// 服務到 listener 出錯為止;每個連線並發服務、互相獨立。
///
/// - data frame(`len > 0`)→ 原封不動 echo 回去(同 wire format)。
/// - heartbeat(`len == 0`)→ 不回應,但算流量。
/// - 一條連線超過 `idle_timeout` 沒有任何 bytes 進來 → 關閉該連線。
pub async fn serve(listener: TcpListener, idle_timeout: Duration) -> io::Result<()> {
    // todo!("rehearsal")
    // start in 11:21 after clarify
    loop {
        let (mut stream, _addr) = listener.accept().await?;
        tokio::spawn(async move {
            let mut parser = frame_parser_heartbeat::FrameParser::new();
            let mut buf = [0; 4096];
            'parsing: loop {
                let res = tokio::time::timeout(idle_timeout, stream.read(&mut buf)).await;
                let read_res = match res {
                    Ok(read_res) => read_res,
                    Err(_timeout) => break 'parsing,
                };
                match read_res {
                    Ok(0) => break 'parsing,
                    Ok(n) => {
                        let datas = parser.feed(&buf[0..n]);
                        for data in datas {
                            match data {
                                frame_parser_heartbeat::Frame::Heartbeat => continue,
                                frame_parser_heartbeat::Frame::Data(data) => {
                                    let mut throwback = (data.len() as u32).to_be_bytes().to_vec();
                                    throwback.extend(&data);
                                    let res = stream.write_all(&throwback).await;
                                    if res.is_err() {
                                        break 'parsing;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => break 'parsing,
                }
            }
        });
    }
}

#[tokio::test]
async fn dryrun_happy_path() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    tokio::spawn(async move { serve(listener, Duration::from_secs(10000)).await });
    let sender = TcpStream::connect(local_addr).await;
    assert!(sender.is_ok());
    let mut stream = sender.unwrap();
    assert!(stream.write_all(&[0, 0, 0, 1, 3, 0, 0, 0]).await.is_ok());
    let mut parser = FrameParser::new();
    let mut buf = [0; 4096];
    let res = stream.read(&mut buf).await;
    assert!(res.is_ok());
    let mut len = res.unwrap();
    let res = parser.feed(&buf[0..len]);
    assert!(res.len() == 1);
}

#[tokio::test]
async fn dryrun_boundary() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let res = serve(listener, Duration::from_millis(1)).await;
        assert!(res.is_err());
    });
    let sender = TcpStream::connect(local_addr).await;
    tokio::time::sleep(Duration::from_secs(1)).await;
}
