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

clarify:
1. is the len contains the len bytes?
    -> no because len == 0 is a heartbeat
2. what's the maximum len size?
    -> let assume 4096
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
    ptr: usize,
    buf: Vec<u8>,
}

impl FrameParser {
    pub fn new() -> Self {
        // todo!("rehearsal")
        Self {
            ptr: 0,
            buf: Vec::new(),
        }
    }

    fn may_compact(&mut self) {
        if self.ptr > 4096 {
            self.buf.drain(..=4096);
        }
    }

    fn parse(&mut self) -> Vec<Frame> {
        let mut ans = Vec::new();
        // 0,0,0,2,32,24,0,0,0 | len is 9
        //               6
        while (self.buf.len() - self.ptr) >= 4 {
            let (left, right) = self.buf[self.ptr..].split_at(4);
            // left ensure split with 4 length
            let length = u32::from_be_bytes(left.try_into().unwrap());
            if length == 0 {
                ans.push(Frame::Heartbeat);
                self.ptr += 4;
                continue;
            }
            if right.len() < length as usize {
                break;
            }
            let data = right[..length as usize].to_vec();
            ans.push(Frame::Data(data));
            // take ex. [0,0,0,2,32,24,0,0,0,0,] ptr = 0, len = 2, 4 + 2 = 6, ptr = 0 + 6
            self.ptr += (4 + length) as usize;
        }
        self.may_compact();
        ans
    }

    /// 吃進這次新到的 bytes,回傳**這次新完成**的所有 frame(依 stream 順序)。
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Frame> {
        // todo!("rehearsal")
        self.buf.extend_from_slice(bytes);
        self.parse()
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
