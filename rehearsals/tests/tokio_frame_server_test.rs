//! 參考測試:tokio_frame_server。
//!
//! 彩排時先自己寫測試(寫在 src/tokio_frame_server.rs 底部);轉綠後才跑這組:
//! `cargo test -p rehearsals --test tokio_frame_server_test -- --include-ignored`

use rehearsals::tokio_frame_server::serve;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// 測試用 idle timeout:夠短讓測試快,夠長不被排程抖動誤傷。
const IDLE: Duration = Duration::from_millis(400);

/// 組一個合法 frame:[u32 len(BE)][payload]。
fn frame(payload: &[u8]) -> Vec<u8> {
    let mut v = (payload.len() as u32).to_be_bytes().to_vec();
    v.extend_from_slice(payload);
    v
}

/// 起 server(port 0 由 OS 配),回傳位址;server task 隨 runtime 結束消滅。
async fn start_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(serve(listener, IDLE));
    addr
}

/// 從 socket 讀一個完整 frame,回傳 payload。
async fn read_frame(sock: &mut TcpStream) -> Vec<u8> {
    let mut len_buf = [0u8; 4];
    sock.read_exact(&mut len_buf).await.unwrap();
    let mut payload = vec![0u8; u32::from_be_bytes(len_buf) as usize];
    sock.read_exact(&mut payload).await.unwrap();
    payload
}

/// 基本 echo:一個 data frame 進、同 payload 的 frame 出。
#[tokio::test]
#[ignore = "參考測試:彩排完成後再開"]
async fn echo_single_frame() {
    let addr = start_server().await;
    let mut c = TcpStream::connect(addr).await.unwrap();
    c.write_all(&frame(b"hello")).await.unwrap();
    assert_eq!(read_frame(&mut c).await, b"hello");
}

/// boundary:byte stream 沒有邊界——半個 frame 分兩次寫要能拼回來,
/// 一次寫兩個 frame 要各自 echo、順序不亂(題目 c 的功課在 server 端重用)。
#[tokio::test]
#[ignore = "參考測試:彩排完成後再開"]
async fn partial_and_batched_writes() {
    let addr = start_server().await;
    let mut c = TcpStream::connect(addr).await.unwrap();

    let f = frame(b"abc");
    c.write_all(&f[..3]).await.unwrap(); // len 欄位都還沒到齊
    tokio::time::sleep(Duration::from_millis(50)).await;
    c.write_all(&f[3..]).await.unwrap();
    assert_eq!(read_frame(&mut c).await, b"abc");

    let mut two = frame(b"x");
    two.extend(frame(b"yz"));
    c.write_all(&two).await.unwrap();
    assert_eq!(read_frame(&mut c).await, b"x");
    assert_eq!(read_frame(&mut c).await, b"yz");
}

/// boundary:heartbeat 不回應——夾在 data frame 之間也不能產生任何回覆 bytes
/// (若誤回了 empty frame,下面第一個 read_frame 會讀到 b"" 而非 b"a")。
#[tokio::test]
#[ignore = "參考測試:彩排完成後再開"]
async fn heartbeat_produces_no_response() {
    let addr = start_server().await;
    let mut c = TcpStream::connect(addr).await.unwrap();

    let mut bytes = frame(b""); // 開頭就是 heartbeat
    bytes.extend(frame(b"a"));
    bytes.extend(frame(b"")); // 夾中間
    bytes.extend(frame(b"b"));
    c.write_all(&bytes).await.unwrap();

    assert_eq!(read_frame(&mut c).await, b"a");
    assert_eq!(read_frame(&mut c).await, b"b");
}

/// boundary:連線互相獨立——c1 卡著半個 frame 不能阻塞 c2 的服務
/// (per-connection 狀態隔離,無跨連線 head-of-line blocking)。
#[tokio::test]
#[ignore = "參考測試:彩排完成後再開"]
async fn concurrent_connections_isolated() {
    let addr = start_server().await;
    let mut c1 = TcpStream::connect(addr).await.unwrap();
    let mut c2 = TcpStream::connect(addr).await.unwrap();

    let f1 = frame(b"from-1");
    c1.write_all(&f1[..4]).await.unwrap(); // c1 卡半個 frame

    c2.write_all(&frame(b"from-2")).await.unwrap();
    assert_eq!(read_frame(&mut c2).await, b"from-2"); // c2 不受影響

    c1.write_all(&f1[4..]).await.unwrap(); // c1 補齊
    assert_eq!(read_frame(&mut c1).await, b"from-1");
}

/// boundary:idle timeout——完全沒流量的連線,server 要在 idle_timeout 後
/// 主動關閉(client 讀到 EOF)。watchdog 3 秒:沒關就是明確失敗,不是卡死。
#[tokio::test]
#[ignore = "參考測試:彩排完成後再開"]
async fn idle_connection_closed() {
    let addr = start_server().await;
    let mut c = TcpStream::connect(addr).await.unwrap();

    let mut buf = [0u8; 1];
    let n = tokio::time::timeout(Duration::from_secs(3), c.read(&mut buf))
        .await
        .expect("server 應在 idle_timeout 後關閉連線,不該讓 read 卡住")
        .unwrap();
    assert_eq!(n, 0, "應讀到 EOF(server 端關閉)");
}

/// boundary:heartbeat 保活——這就是 heartbeat 存在的目的。
/// 連續 heartbeat 撐過 2.5 倍 idle_timeout 後,連線必須還活著、echo 照常。
#[tokio::test]
#[ignore = "參考測試:彩排完成後再開"]
async fn heartbeats_keep_idle_connection_alive() {
    let addr = start_server().await;
    let mut c = TcpStream::connect(addr).await.unwrap();

    for _ in 0..10 {
        c.write_all(&frame(b"")).await.unwrap(); // 間隔 100ms << IDLE 400ms
        tokio::time::sleep(Duration::from_millis(100)).await; // 總時長 1s > IDLE
    }
    c.write_all(&frame(b"still-alive")).await.unwrap();
    assert_eq!(read_frame(&mut c).await, b"still-alive");
}
