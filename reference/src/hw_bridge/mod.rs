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
//! **handler-IO 對照組**(handler 內部要做阻塞 IO 時,三種下場):
//! - `server_evented_inline.rs`——⚠️ 反面教材:handler 在 IO thread 上跑,
//!   一條慢命令凍住所有連線
//! - `server_evented.rs`——offload 到單 worker:loop 不凍,但延遲跨連線傳染
//!   (worker 佇列陪排)
//! - `server_evented_sharded.rs`——shard by connection:同連線保序、
//!   跨連線隔離(前提:每 shard 有自己的下游通道)
//!
//! tokio 對照:`rehearsals/examples/sol_tokio_frame_server.rs`(async handler
//! 天然不凍 loop——`.await` 就是讓位點)。
//!
//! **佇列對照**(同架構、換佇列):`server_evented_spsc.rs`——IO thread ×
//! 單 worker 天然一產一消,兩條 `Mutex` 佇列換成兩條 SPSC ring + eventfd,
//! handler 免鎖(worker 獨占),買 p99.9。與 `crate::signal_pipeline`
//! 同一套掛牌握手。
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
//! 對應 docs/io/hw_bridge.md:45 分鐘增量順序、binary vs text、
//! length-prefix vs delimiter 的取捨。

pub mod client;
pub mod framer;
pub mod handler;
pub mod protocol;
pub mod server_evented;
pub mod server_evented_inline;
pub mod server_evented_sharded;
pub mod server_evented_spsc;
pub mod server_threaded;

#[cfg(test)]
mod tests {
    use super::client::{ClientError, HwClient};
    use super::handler::{MockHardware, SlowHardware};
    use super::protocol::{Command, ERR_UNKNOWN_OPCODE, Response, encode_frame};
    use super::server_evented::EventedServer;
    use super::server_evented_inline::InlineServer;
    use super::server_evented_sharded::ShardedServer;
    use super::server_threaded::ThreadedServer;
    use std::net::SocketAddr;
    use std::thread;
    use std::time::{Duration, Instant};

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

    /// SPSC evented server 跑同一套 client 行為測試 + pipeline 保序:
    /// 換佇列不換行為。
    #[test]
    fn spsc_evented_server_passes_full_client_suite() {
        use super::server_evented_spsc::SpscEventedServer;
        let mut server = SpscEventedServer::bind("127.0.0.1:0", MockHardware::default()).unwrap();
        let addr = server.local_addr().unwrap();
        let shutdown = server.shutdown_handle();
        let join = thread::spawn(move || {
            let r = server.run();
            drop(server); // Drop:worker drain + join
            r
        });
        exercise_server(addr);

        let mut c = HwClient::connect(addr).unwrap();
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
        drop(c);
        shutdown.shutdown();
        join.join().unwrap().unwrap();
    }

    /// sharded server 跑同一套 client 行為測試:協定層面與其他 server 等價。
    #[test]
    fn sharded_server_passes_full_client_suite() {
        let handlers = (0..4).map(|_| MockHardware::default()).collect();
        let mut server = ShardedServer::bind("127.0.0.1:0", handlers).unwrap();
        let addr = server.local_addr().unwrap();
        let shutdown = server.shutdown_handle();
        let join = thread::spawn(move || server.run());
        exercise_server(addr);
        shutdown.shutdown();
        join.join().unwrap().unwrap();
    }

    /// 保序不因 shard 而破:同一連線 → 同一 shard → 單 worker FIFO。
    #[test]
    fn sharded_pipelined_commands_stay_in_order() {
        let handlers = (0..4).map(|_| MockHardware::default()).collect();
        let mut server = ShardedServer::bind("127.0.0.1:0", handlers).unwrap();
        let addr = server.local_addr().unwrap();
        let shutdown = server.shutdown_handle();
        let join = thread::spawn(move || server.run());

        let mut c = HwClient::connect(addr).unwrap();
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

    /// handler-IO 對照組的可執行證據(模組 doc 的三種下場,量兩端):
    ///
    /// 場景:client A 發慢 ReadSensor(SlowHardware sleep 300ms),
    /// 50ms 後 client B 發 Ping(快路徑,handler 不 sleep)。
    ///
    /// - inline 版:handler 在 IO thread 上 sleep,B 的 Ping 讀都讀不進來
    ///   → B 延遲 ≈ 整段 delay(斷言 ≥150ms)。
    /// - sharded 版:A、B token 不同 → 不同 shard → B 不陪等
    ///   (斷言 <150ms)。閾值取 delay 的一半,留 CI 抖動空間。
    #[test]
    fn slow_handler_latency_contrast() {
        let delay = Duration::from_millis(300);

        // ── inline(反面教材):B 陪 A 凍整段 ──
        let mut server = InlineServer::bind("127.0.0.1:0", SlowHardware::new(delay)).unwrap();
        let addr = server.local_addr().unwrap();
        let shutdown = server.shutdown_handle();
        let join = thread::spawn(move || server.run());

        let mut a = HwClient::connect(addr).unwrap();
        let mut b = HwClient::connect(addr).unwrap();
        b.ping().unwrap(); // 先確認兩條連線都 accept 完成
        let slow = thread::spawn(move || {
            a.read_sensor(1).unwrap();
        });
        thread::sleep(Duration::from_millis(50)); // 讓 A 的慢命令先進 loop
        let t0 = Instant::now();
        b.ping().unwrap();
        let inline_ping = t0.elapsed();
        slow.join().unwrap();
        shutdown.shutdown();
        join.join().unwrap().unwrap();
        assert!(
            inline_ping >= Duration::from_millis(150),
            "inline 版 B 的 Ping 應被 A 的慢命令拖住,實測 {inline_ping:?}"
        );

        // ── sharded:B 在自己的 shard,不陪等 ──
        let handlers = (0..4).map(|_| SlowHardware::new(delay)).collect();
        let mut server = ShardedServer::bind("127.0.0.1:0", handlers).unwrap();
        let addr = server.local_addr().unwrap();
        let shutdown = server.shutdown_handle();
        let join = thread::spawn(move || server.run());

        let mut a = HwClient::connect(addr).unwrap();
        let mut b = HwClient::connect(addr).unwrap();
        b.ping().unwrap();
        let slow = thread::spawn(move || {
            a.read_sensor(1).unwrap();
        });
        thread::sleep(Duration::from_millis(50));
        let t0 = Instant::now();
        b.ping().unwrap();
        let sharded_ping = t0.elapsed();
        slow.join().unwrap();
        shutdown.shutdown();
        join.join().unwrap().unwrap();
        assert!(
            sharded_ping < Duration::from_millis(150),
            "sharded 版 B 的 Ping 不該陪等,實測 {sharded_ping:?}"
        );
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
