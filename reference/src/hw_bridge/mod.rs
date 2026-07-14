//! # hw_bridge —— 橋接軟硬體:binary protocol server + client
//!
//! ## [Clarify]
//! 解決:上位軟體(監控/管理程式)透過 TCP 對硬體控制器下命令、收回應。
//! 模擬的控制器 opcode 表:Ping(健康檢查)、ReadSensor(讀溫度)、
//! SetFan(設轉速)。Constraints:std-only、binary wire format(嵌入式
//! 對端沒有 JSON parser 的餘裕)、多 client 並發、命令對硬體必須序列化
//! (一台設備一次一條命令)。規模:連線數 10⁰–10²,命令頻率 ~kHz。
//!
//! ## [Abstract]
//! 真硬體 stub 成 `CommandHandler` trait + `MockHardware`(handler.rs)——
//! 面試時聲明「硬體行為我 mock 掉,回確定性假值,先把通訊打通」。
//!
//! ## [Iterate] 模組內的分層即演進順序
//! 1. `protocol.rs`——紙上定 wire format,encode / **try_decode**
//! 2. `framer.rs`——`FrameReader`:byte stream 上重建 message 邊界(**最易錯**)
//! 3. `server_threaded.rs`——thread-per-conn:45 分鐘能跑的端到端
//! 4. `server_evented.rs`——event loop + command worker:scale 版
//! 5. `client.rs`——sync 請求/回應(async 版的 request-id 設計見其 doc)
//!
//! ## [Trade-offs] thread-per-conn vs event loop(§45 分鐘的核心問答)
//! | | threaded | evented |
//! |---|---|---|
//! | code 複雜度 | 直線邏輯,阻塞 IO | interest 狀態機 + 欠帳緩衝 + 回程信箱 |
//! | 每連線成本 | 1 thread(~8MB 位址空間 + 排程) | ~數百 bytes 狀態 |
//! | 連線上限 | ~10² 舒適 | ~10⁴⁺(C10K) |
//! | 硬體序列化 | Arc\<Mutex\> 在 handler 上 | 單一 command worker 天然序列 |
//! | 適用 | 硬體控制器的真實連線數 | 閘道器/雲端側 |
//!
//! 面試順序:**先 threaded 打通端到端,有時間才上 evented**——
//! 兩版共用 protocol/framer/handler,重寫的只有 IO 骨架。
//!
//! ## [Dry-Run]
//! protocol/framer 的 boundary 測試在各自檔案(partial 窮舉切割點、
//! 多 frame、malformed、byte-by-byte)。本檔的整合測試把**同一套
//! client 測試組**跑在兩個 server 上:基本命令、雙 client 交錯、
//! 未知 opcode → Error frame(連線活著)、framing 損毀 → 斷線、
//! 送半個 frame 就斷線(server 不倒)。
//!
//! 對應 docs/hw_bridge.md:45 分鐘增量順序、binary vs text、
//! length-prefix vs delimiter 的取捨。

pub mod client;
pub mod framer;
pub mod handler;
pub mod protocol;
pub mod server_evented;
pub mod server_threaded;

#[cfg(test)]
mod tests {
    use super::client::{ClientError, HwClient};
    use super::handler::MockHardware;
    use super::protocol::{Command, ERR_UNKNOWN_OPCODE, Response, encode_frame};
    use super::server_evented::EventedServer;
    use super::server_threaded::ThreadedServer;
    use std::net::SocketAddr;
    use std::thread;
    use std::time::Duration;

    /// 兩個 server 共用的整套 client 行為測試。
    /// 任何 server 實作只要能過這套,協定層面就等價。
    fn exercise_server(addr: SocketAddr) {
        // 1. 基本命令 roundtrip
        let mut c = HwClient::connect(addr).unwrap();
        c.ping().unwrap();
        assert_eq!(
            c.read_sensor(25).unwrap(),
            MockHardware::sensor_value(25) // 確定性 mock:可重算
        );
        c.set_fan(4200).unwrap();
        // SetFan 的副作用透過協定觀察不到,但 Ack 已證明命令走完全程。

        // 2. 雙 client 交錯:server 同時伺服、回應不串線
        let mut c2 = HwClient::connect(addr).unwrap();
        c2.ping().unwrap();
        assert_eq!(c.read_sensor(7).unwrap(), MockHardware::sensor_value(7));
        assert_eq!(c2.read_sensor(9).unwrap(), MockHardware::sensor_value(9));

        // 3. 未知 opcode:語意層錯誤 → Error frame,連線活著
        c.send_raw(&encode_frame(0x7E, &[])).unwrap();
        match c.recv_response() {
            Ok(Response::Error { code }) => assert_eq!(code, ERR_UNKNOWN_OPCODE),
            other => panic!("expected Error frame, got {other:?}"),
        }
        c.ping().unwrap(); // 同一條連線繼續可用

        // 4. 命令拆兩段送(partial frame 穿過真 TCP):server 的 framer 要黏回來
        let bytes = Command::ReadSensor { sensor_id: 3 }.encode();
        c.send_raw(&bytes[..3]).unwrap();
        thread::sleep(Duration::from_millis(30)); // 逼出兩次獨立的 read
        c.send_raw(&bytes[3..]).unwrap();
        match c.recv_response() {
            Ok(Response::SensorValue {
                sensor_id,
                millicelsius,
            }) => {
                assert_eq!(sensor_id, 3);
                assert_eq!(millicelsius, MockHardware::sensor_value(3));
            }
            other => panic!("expected SensorValue, got {other:?}"),
        }

        // 5. framing 損毀(len 超大):server 應斷線(後續 IO 得到 EOF/錯誤)
        let mut bad = HwClient::connect(addr).unwrap();
        bad.send_raw(&u32::MAX.to_be_bytes()).unwrap();
        match bad.ping() {
            Err(ClientError::ServerClosed | ClientError::Io(_)) => {}
            other => panic!("expected connection death, got {other:?}"),
        }

        // 6. 半個 frame 就斷線:server 不能倒(下一個 client 還連得上)
        let mut half = HwClient::connect(addr).unwrap();
        half.send_raw(&bytes[..2]).unwrap();
        drop(half);
        thread::sleep(Duration::from_millis(30));
        let mut again = HwClient::connect(addr).unwrap();
        again.ping().unwrap();
    }

    #[test]
    fn threaded_server_passes_full_client_suite() {
        let mut server = ThreadedServer::bind("127.0.0.1:0", MockHardware::default()).unwrap();
        let addr = server.local_addr().unwrap();
        let shutdown = server.shutdown_handle().unwrap();
        let join = thread::spawn(move || server.run());
        exercise_server(addr);
        shutdown.shutdown();
        join.join().unwrap().unwrap();
    }

    #[test]
    fn evented_server_passes_full_client_suite() {
        let mut server = EventedServer::bind("127.0.0.1:0", MockHardware::default()).unwrap();
        let addr = server.local_addr().unwrap();
        let shutdown = server.shutdown_handle();
        let join = thread::spawn(move || server.run());
        exercise_server(addr);
        shutdown.shutdown();
        join.join().unwrap().unwrap();
    }

    /// 順序保證:同一連線連發 20 條不同命令(不等回應),回應必須
    /// 按請求順序回來(sync 協定的 FIFO 對應;evented server 的單 worker
    /// 設計就是為了這條性質)。
    #[test]
    fn pipelined_commands_come_back_in_order() {
        let mut server = EventedServer::bind("127.0.0.1:0", MockHardware::default()).unwrap();
        let addr = server.local_addr().unwrap();
        let shutdown = server.shutdown_handle();
        let join = thread::spawn(move || server.run());

        let mut c = HwClient::connect(addr).unwrap();
        // 一口氣送 20 條(pipeline),再依序收 20 個回應
        for id in 0..20u16 {
            c.send_raw(&Command::ReadSensor { sensor_id: id }.encode())
                .unwrap();
        }
        for id in 0..20u16 {
            match c.recv_response().unwrap() {
                Response::SensorValue { sensor_id, .. } => assert_eq!(sensor_id, id),
                other => panic!("expected SensorValue, got {other:?}"),
            }
        }
        shutdown.shutdown();
        join.join().unwrap().unwrap();
    }
}
