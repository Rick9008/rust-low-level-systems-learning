//! solution:題 c frame_parser_heartbeat——**寫完彩排才開**。
//! canonical 設計:累積 buffer + read cursor,只在需要時 compaction
//! (每個 frame 都 drain 前端是 O(n) per frame → 整體 O(n²);cursor 攤銷 O(n))。
//! 驗證:rehearsals/tests/frame_parser_heartbeat_test.rs 全綠。

#[derive(Debug, PartialEq, Eq)]
pub enum Frame {
    Heartbeat,
    Data(Vec<u8>),
}

pub struct FrameParser {
    buf: Vec<u8>,
    pos: usize, // read cursor:pos 之前是已消費的 bytes
}

impl FrameParser {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            pos: 0,
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Frame> {
        self.buf.extend_from_slice(bytes);
        let mut out = Vec::new();
        loop {
            let rest = &self.buf[self.pos..];
            if rest.len() < 4 {
                break; // len 欄位還沒到齊
            }
            let len = u32::from_be_bytes(rest[..4].try_into().unwrap()) as usize;
            if rest.len() < 4 + len {
                break; // payload 還沒到齊
            }
            out.push(if len == 0 {
                Frame::Heartbeat
            } else {
                Frame::Data(rest[4..4 + len].to_vec())
            });
            self.pos += 4 + len;
        }
        // compaction:全部消費完就歸零;殘留半個 frame 且前綴夠大才搬——攤銷 O(n)
        if self.pos == self.buf.len() {
            self.buf.clear();
            self.pos = 0;
        } else if self.pos > 4096 {
            self.buf.drain(..self.pos);
            self.pos = 0;
        }
        out
    }
}

impl Default for FrameParser {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    fn frame(payload: &[u8]) -> Vec<u8> {
        let mut v = (payload.len() as u32).to_be_bytes().to_vec();
        v.extend_from_slice(payload);
        v
    }
    let mut p = FrameParser::new();
    let mut stream = frame(b"ab");
    stream.extend(frame(b"")); // heartbeat 夾中間
    stream.extend(frame(b"cde"));
    let mut got = Vec::new();
    for b in stream {
        got.extend(p.feed(&[b])); // 逐 byte 餵:每個切斷點都走過
    }
    assert_eq!(
        got,
        vec![
            Frame::Data(b"ab".to_vec()),
            Frame::Heartbeat,
            Frame::Data(b"cde".to_vec()),
        ]
    );
    println!("sol_frame_parser_heartbeat: ok");
}
