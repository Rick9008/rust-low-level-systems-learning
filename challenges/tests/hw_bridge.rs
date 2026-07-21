//! 驗收:challenges::io::hw_bridge。完成後移除 #[ignore]。
//! 注意:編碼端刻意用 **reference 的 encode**——你的解碼要與它互通
//! (自己編自己解會把「兩邊都錯」的 bug 藏起來)。

use challenges::io::hw_bridge::{DecodeError, FrameReader, MAX_FRAME_LEN, try_decode};
use reference::io::hw_bridge::protocol as ref_proto;

fn drain(r: &mut FrameReader) -> Vec<challenges::io::hw_bridge::RawFrame> {
    let mut out = Vec::new();
    while let Some(f) = r.next_frame().unwrap() {
        out.push(f);
    }
    out
}

/// 核心驗收:與 reference encoder 的精確互通。
#[test]
#[ignore = "完成 challenge 後移除"]
fn interop_with_reference_encoder() {
    let bytes = ref_proto::encode_frame(0x02, &[0x01, 0x02]);
    assert_eq!(bytes, vec![0, 0, 0, 3, 0x02, 0x01, 0x02]); // 佈局再確認一次
    let (frame, consumed) = try_decode(&bytes).unwrap().unwrap();
    assert_eq!(consumed, 7);
    assert_eq!(frame.opcode, 0x02);
    assert_eq!(frame.payload, vec![0x01, 0x02]);
}

/// boundary:每一個不完整前綴都是 Ok(None)。
#[test]
#[ignore = "完成 challenge 後移除"]
fn every_partial_prefix_is_none() {
    let full = ref_proto::encode_frame(0x01, &[9, 9, 9]);
    for cut in 0..full.len() {
        assert_eq!(try_decode(&full[..cut]).unwrap(), None, "prefix {cut}");
    }
}

/// boundary:malformed len 是 Err 不是 None。
#[test]
#[ignore = "完成 challenge 後移除"]
fn malformed_is_err() {
    assert_eq!(
        try_decode(&0u32.to_be_bytes()),
        Err(DecodeError::EmptyFrame)
    );
    assert_eq!(
        try_decode(&(MAX_FRAME_LEN + 1).to_be_bytes()),
        Err(DecodeError::FrameTooLarge(MAX_FRAME_LEN + 1))
    );
}

/// FrameReader:窮舉切割點 + 多 frame 一次 feed + byte-by-byte。
#[test]
#[ignore = "完成 challenge 後移除"]
fn frame_reader_reassembles_any_split() {
    let f1 = ref_proto::encode_frame(0x01, &[]);
    let f2 = ref_proto::encode_frame(0x02, &[7, 7]);
    let all: Vec<u8> = f1.iter().chain(f2.iter()).copied().collect();
    // 窮舉兩段切割
    for split in 0..all.len() {
        let mut r = FrameReader::new();
        r.feed(&all[..split]);
        let mut got = drain(&mut r);
        r.feed(&all[split..]);
        got.extend(drain(&mut r));
        assert_eq!(got.len(), 2, "split {split}");
        assert_eq!(got[0].opcode, 0x01);
        assert_eq!(got[1].opcode, 0x02);
    }
    // byte-by-byte
    let mut r = FrameReader::new();
    let mut count = 0;
    for b in all {
        r.feed(&[b]);
        count += drain(&mut r).len();
    }
    assert_eq!(count, 2);
}

/// 整合驗收:reference 的 ThreadedServer 當 harness——
/// 你的 FrameReader 在真 TCP 上解析 server 回應(kernel 決定分段)。
#[test]
#[ignore = "完成 challenge 後移除"]
fn end_to_end_against_reference_server() {
    use reference::io::hw_bridge::handler::MockHardware;
    use reference::io::hw_bridge::server_threaded::ThreadedServer;
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let mut server = ThreadedServer::bind("127.0.0.1:0", MockHardware::default()).unwrap();
    let addr = server.local_addr().unwrap();
    let shutdown = server.shutdown_handle().unwrap();
    let join = std::thread::spawn(move || server.run());

    let mut sock = TcpStream::connect(addr).unwrap();
    // pipeline 10 條 ReadSensor(id=0..10),一次全部送出
    for id in 0..10u16 {
        sock.write_all(&ref_proto::Command::ReadSensor { sensor_id: id }.encode())
            .unwrap();
    }
    // 用你的 FrameReader 收 10 個回應 frame,順序與 id 對應
    let mut reader = FrameReader::new();
    let mut frames = Vec::new();
    let mut buf = [0u8; 4096];
    while frames.len() < 10 {
        let n = sock.read(&mut buf).unwrap();
        assert!(n > 0, "server closed early");
        reader.feed(&buf[..n]);
        frames.extend(drain(&mut reader));
    }
    for (i, f) in frames.iter().enumerate() {
        let resp = ref_proto::Response::try_from_frame(&ref_proto::RawFrame {
            opcode: f.opcode,
            payload: f.payload.clone(),
        })
        .unwrap();
        match resp {
            ref_proto::Response::SensorValue { sensor_id, .. } => {
                assert_eq!(sensor_id, i as u16);
            }
            other => panic!("expected SensorValue, got {other:?}"),
        }
    }
    shutdown.shutdown();
    join.join().unwrap().unwrap();
}
