// ═══ Warmup 8/4 — 題 1(建議 4m)═══
//
// Write a TCP echo server using tokio: bind `127.0.0.1:9000`, accept
// connections forever, handle each connection concurrently, echo back
// whatever you read, and stop serving a connection cleanly when the
// peer closes it. Errors on a single connection must not take down
// the server.
//
// (從這行以下全部自己來:imports、entry point、一切。)

use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9000").await?;
    loop {
        let (mut stream, _peer) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0u8;4096];
            loop {
                match stream.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if stream.write_all(&buf[0..n]).await.is_err() {
                            break;
                        }
                    },
                    Err(_) => break,
                }
            }
        });
    }
    Ok(())
}

// ═══ 批改(8/4;tokio 單檔不可編,eye-grade;Claude)═══
//
// 合約 ✓:spawn per-conn、Ok(0)=>break(EOF)、write_all 有 await、單連線錯誤不殺 server。
//
// 1. 混皮洞(唯一值得想 10 秒的):while let Some(...) 是 std incoming() 的皮
//    (iterator 才有 Some);tokio accept() 回 Result,? 之後直接是 tuple。
//    ✓ loop { let (mut stream, _peer) = listener.accept().await?; ... }
// 2. ✗ use tokio::io::{AsyncReadExt, AsyncWriteExt}; 沒寫——昨錯①二連 → 8/5 taper 名單。
//    (E0599 會直接建議 trait 名 = 名字洞;但「唯一死記項」連掉兩天,值得標。)
// 3. ✗ stream.read(&mut buf) 少 .await——昨天 bind 行點名三次,今天搬到 read 行;
//    write_all 有寫 = 注意力洞非概念洞,列追蹤。
// 4. ✗ let mut buf = &[0u8; 4096];  → ✓ let mut buf = [0u8; 4096];(owned array,不是 &)
// 5. typos:TcpLisnter / accpet;main 結尾少 Ok(())。
// 6. 設計註(不算錯):accept 用 ? = accept 錯誤殺整台 server;
//    checklist §2b 的 ⚠ 是「accept 失敗多半暫時 → log-and-continue」,面試講一句即可。
