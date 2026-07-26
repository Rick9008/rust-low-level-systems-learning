//! rehearsal c:frame_parser_heartbeat —— 題目見 rehearsals/README.md。
//!
//! wire format:`[u32 len(BE)][payload:len bytes]`,`len == 0` 是 heartbeat。
//! 只給 API 簽名;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`。

/*
Devices send frames over TCP. Wire format:

```text
[u32 len (big-endian)][payload: len bytes]
```

`len` is the payload byte count; a frame with `len == 0` is a **heartbeat**
(no payload). TCP is a byte stream — a single `read` may hand you half a
frame, or several frames at once.

Write an incremental parser: `feed(&[u8])` consumes the newly arrived bytes
and returns **all frames completed by this call**, in stream order.
Heartbeats must be reported too. Assume the stream is well-formed (trusted
peer — no malformed handling needed).
*/

#[derive(Debug, PartialEq, Eq)]
pub enum Frame {
    /// `len == 0`,無 payload。
    Heartbeat,
    /// `len > 0`,帶完整 payload。
    Data(Vec<u8>),
}

pub struct FrameParser {
    // ↓ 佔位:動手時整個換成你的設計。
    // _todo: (),
    buf: Vec<u8>,
    ptr: usize,
}

impl FrameParser {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            ptr: 0,
        }
    }

    /// 吃進這次新到的 bytes,回傳**這次新完成**的所有 frame(依 stream 順序)。
    /// time: O(n)
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Frame> {
        let mut res = Vec::new();
        self.buf.extend_from_slice(bytes);
        while let Some(frame) = self.parse() {
            res.push(frame);
        }
        self.may_compact();
        res
    }
    // time: O(n) (amortized)
    fn may_compact(&mut self) {
        // if self.ptr = 4200
        if self.ptr > 4096 {
            // drain(..4200), kep 4200..
            self.buf.drain(..self.ptr);
            // self.ptr = 0
            self.ptr = 0;
        }
    }

    // O(n) amortized
    fn parse(&mut self) -> Option<Frame> {
        let remain = self.buf.len() - self.ptr;
        if remain < 4 {
            return None;
        }
        let length = u32::from_be_bytes(
            self.buf[self.ptr..self.ptr + 4]
                .try_into()
                .expect("expect 4 bytes"),
        ) as usize;
        let buf_remain = self.buf.len() - self.ptr - 4;
        if length > buf_remain {
            return None;
        }
        let frame = match length {
            0 => Frame::Heartbeat,
            _ => Frame::Data(self.buf[self.ptr + 4..self.ptr + 4 + length].to_vec()),
        };
        self.ptr += 4 + length;
        Some(frame)
    }
}

impl Default for FrameParser {
    fn default() -> Self {
        Self::new()
    }
}

// dry run

#[test]
fn dryrun() {
    let mut parser = FrameParser::new();
    let empty_vec = parser.feed(&[0, 0, 0, 2]);
    assert!(empty_vec.is_empty());
    let mut something = parser.feed(&[3, 4, 0, 0, 0]);
    assert_eq!(something.len(), 1);
    assert_eq!(something.pop().unwrap(), Frame::Data(vec![3, 4]));
    let mut heartbeat = parser.feed(&[0]);
    assert_eq!(heartbeat.len(), 1);
    assert_eq!(heartbeat.pop().unwrap(), Frame::Heartbeat);
}

#[test]
fn boundary_test() {
    let mut parser = FrameParser::new();
    parser.feed(&4096i32.to_be_bytes());
    let frames = parser.feed(&[20; 4096]);
    assert!(!frames.is_empty());
    let mut heartbeat_vec = parser.feed(&[0, 0, 0, 0]);
    assert!(!heartbeat_vec.is_empty());
    assert_eq!(heartbeat_vec.pop().unwrap(), Frame::Heartbeat);
    let mut multiple_feed = parser.feed(&[0, 0, 0, 3, 56, 23, 50, 0, 0, 0, 1, 63]);
    assert_eq!(multiple_feed.len(), 2);
    assert_eq!(multiple_feed.pop().unwrap(), Frame::Data(vec![63]));
    assert_eq!(multiple_feed.pop().unwrap(), Frame::Data(vec![56, 23, 50]));
}
