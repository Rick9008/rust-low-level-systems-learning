//! 驗收:challenges::io::tcp_echo。完成後移除 #[ignore]。

use challenges::io::tcp_echo::EchoServer;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn spawn_server() -> (
    std::net::SocketAddr,
    challenges::io::tcp_echo::ShutdownHandle,
    thread::JoinHandle<std::io::Result<()>>,
) {
    let mut server = EchoServer::bind("127.0.0.1:0").unwrap();
    let addr = server.local_addr().unwrap();
    let handle = server.shutdown_handle();
    let join = thread::spawn(move || server.run());
    (addr, handle, join)
}

/// 基本 roundtrip。
#[test]
#[ignore = "完成 challenge 後移除"]
fn echo_roundtrip() {
    let (addr, shutdown, join) = spawn_server();
    let mut c = TcpStream::connect(addr).unwrap();
    c.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    c.write_all(b"hello").unwrap();
    let mut buf = [0u8; 5];
    c.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"hello");
    shutdown.shutdown();
    join.join().unwrap().unwrap();
}

/// 多客戶端交錯(單執行緒 server 同時伺服)。
#[test]
#[ignore = "完成 challenge 後移除"]
fn multiple_clients() {
    let (addr, shutdown, join) = spawn_server();
    let mut clients: Vec<TcpStream> = (0..3).map(|_| TcpStream::connect(addr).unwrap()).collect();
    for c in &clients {
        c.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    }
    for (i, c) in clients.iter_mut().enumerate().rev() {
        c.write_all(format!("msg-{i}").as_bytes()).unwrap();
    }
    for (i, c) in clients.iter_mut().enumerate() {
        let mut buf = [0u8; 5];
        c.read_exact(&mut buf).unwrap();
        assert_eq!(buf, format!("msg-{i}").as_bytes());
    }
    shutdown.shutdown();
    join.join().unwrap().unwrap();
}

/// 核心驗收:1MB 灌流(先寫後讀)——逼出 WouldBlock + 欠帳 + 可寫事件路徑。
#[test]
#[ignore = "完成 challenge 後移除"]
fn large_transfer_partial_writes() {
    let (addr, shutdown, join) = spawn_server();
    const N: usize = 1 << 20;
    let data: Vec<u8> = (0..N).map(|i| (i % 251) as u8).collect();
    let mut c = TcpStream::connect(addr).unwrap();
    c.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut c_read = c.try_clone().unwrap();
    let writer = thread::spawn(move || {
        c.write_all(&data).unwrap();
        data
    });
    let mut got = vec![0u8; N];
    c_read.read_exact(&mut got).unwrap();
    assert_eq!(got, writer.join().unwrap());
    shutdown.shutdown();
    join.join().unwrap().unwrap();
}

/// 斷線清理:client 走了 server 要活著。
#[test]
#[ignore = "完成 challenge 後移除"]
fn disconnect_cleanup() {
    let (addr, shutdown, join) = spawn_server();
    {
        let mut c = TcpStream::connect(addr).unwrap();
        c.write_all(b"bye").unwrap();
    } // 立刻斷線
    thread::sleep(Duration::from_millis(50));
    let mut c2 = TcpStream::connect(addr).unwrap();
    c2.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    c2.write_all(b"again").unwrap();
    let mut buf = [0u8; 5];
    c2.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"again");
    shutdown.shutdown();
    join.join().unwrap().unwrap();
}
