use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 1. bind  2. accept 迴圈  3. 每連線 spawn  4. read→write 一來一回
    // 考點:.await 放哪、spawn 的 closure 是什麼形狀
    let tcp_listener = TcpListener::bind("127.0.0.1:0").await?;
    loop {
        let (mut stream, _peer) = tcp_listener.accept().await?;
        tokio::spawn(async move {
           let mut buf = [0u8; 4096];
            loop {
                match stream.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {if stream.write_all(&buf[..n]).await.is_err() {
                        break;
                    }},
                    Err(_) => break,
                }
            }
        });
    }
}
