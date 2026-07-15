//! 參考測試:frame_parser_heartbeat。
//!
//! 彩排時先自己寫測試(寫在 src/frame_parser_heartbeat.rs 底部);轉綠後才跑這組:
//! `cargo test -p rehearsals --test frame_parser_heartbeat_test -- --include-ignored`

use rehearsals::frame_parser_heartbeat::{Frame, FrameParser};

/// 組一個合法 frame:[u32 len(BE)][payload]。
fn frame(payload: &[u8]) -> Vec<u8> {
    let mut v = (payload.len() as u32).to_be_bytes().to_vec();
    v.extend_from_slice(payload);
    v
}

/// boundary:剛好一個完整 frame——一次 feed 一次吐,buffer 不殘留。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn exactly_one_frame() {
    let mut p = FrameParser::new();
    assert_eq!(p.feed(&frame(b"abc")), vec![Frame::Data(b"abc".to_vec())]);
    assert_eq!(p.feed(&[]), vec![]); // 沒有殘留的半個 frame
}

/// boundary:半個 frame 分兩次 feed——len 到齊、payload 只到一半,
/// 第一次必須回空,第二次補齊才吐 frame。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn half_frame_two_feeds() {
    let mut p = FrameParser::new();
    let f = frame(b"hello"); // 4 + 5 bytes
    assert_eq!(p.feed(&f[..6]), vec![]); // len 完整、payload 只有 2/5
    assert_eq!(p.feed(&f[6..]), vec![Frame::Data(b"hello".to_vec())]);
}

/// boundary:一次 feed 夾兩個 frame——同一次呼叫要吐出兩個、順序不亂。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn two_frames_one_feed() {
    let mut p = FrameParser::new();
    let mut bytes = frame(b"x");
    bytes.extend(frame(b"yz"));
    assert_eq!(
        p.feed(&bytes),
        vec![Frame::Data(b"x".to_vec()), Frame::Data(b"yz".to_vec())]
    );
}

/// boundary:zero-payload heartbeat——單獨出現要如實回報,
/// 夾在兩個 data frame 中間也不能被吞掉。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn heartbeat_zero_payload() {
    let mut p = FrameParser::new();
    assert_eq!(p.feed(&frame(b"")), vec![Frame::Heartbeat]);

    let mut bytes = frame(b"a");
    bytes.extend(frame(b"")); // heartbeat 夾中間
    bytes.extend(frame(b"b"));
    assert_eq!(
        p.feed(&bytes),
        vec![
            Frame::Data(b"a".to_vec()),
            Frame::Heartbeat,
            Frame::Data(b"b".to_vec()),
        ]
    );
}

/// boundary:len 欄位本身被切斷——4 bytes 的 len 只到了 2 bytes,
/// parser 不能把它當 payload 或提前判斷。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn len_field_split_across_feeds() {
    let mut p = FrameParser::new();
    let f = frame(b"hi"); // [0,0,0,2,'h','i']
    assert_eq!(p.feed(&f[..2]), vec![]); // len 只來了一半
    assert_eq!(p.feed(&f[2..]), vec![Frame::Data(b"hi".to_vec())]);
}

/// 最嚴格的增量測試:整條 stream(data + heartbeat + data)逐 byte 餵,
/// 每一個切斷點都被走過一次,結果必須與一次餵完全相同。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn byte_by_byte_feed() {
    let mut stream = frame(b"ab");
    stream.extend(frame(b""));
    stream.extend(frame(b"cde"));

    let mut p = FrameParser::new();
    let mut got = Vec::new();
    for b in stream {
        got.extend(p.feed(&[b]));
    }
    assert_eq!(
        got,
        vec![
            Frame::Data(b"ab".to_vec()),
            Frame::Heartbeat,
            Frame::Data(b"cde".to_vec()),
        ]
    );
}
