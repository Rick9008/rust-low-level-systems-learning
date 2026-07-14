//! framing:byte stream → frame stream。**本主題最重要、最容易錯的一段。**
//!
//! TCP 是 **byte stream,不是 message**:一次 `read()` 拿到的可能是
//! 半個 frame、一個半 frame、三個 frame——kernel 只保證 byte 順序,
//! 不保證你 send 的切割方式。length-prefix 的意義就是在 stream 上
//! **重建 message 邊界**。
//!
//! `FrameReader` 是有狀態的黏合層:`feed()` 進 bytes,`next_frame()`
//! 把切得動的 frame 依序吐出來;切不動的殘料留在內部 buffer 等下一批。
//!
//! Off-by-one 高發區(每一條都有測試釘著):
//! - len 含不含自己(本協定:不含,見 protocol.rs)
//! - consumed 之後剩餘 bytes 的搬移偏移
//! - 恰好切在 len 欄位中間 / opcode 後 / payload 差 1 byte

use super::protocol::{DecodeError, RawFrame, try_decode};

pub struct FrameReader {
    buf: Vec<u8>,
    /// 已消費前綴的長度(邏輯上 buf[..read_pos] 已切走)。
    /// 延遲回收:每切一個 frame 就 `drain(..n)` 是 O(剩餘量) 的 memmove,
    /// n 個小 frame 疊成 O(n²);改成游標前進 + 週期性壓實,攤銷 O(1)/byte。
    read_pos: usize,
}

impl Default for FrameReader {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameReader {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            read_pos: 0,
        }
    }

    /// 收下一批剛 read() 到的 bytes。O(bytes)。
    pub fn feed(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// 試切下一個 frame。呼叫端 loop 到 `Ok(None)` 為止
    /// (一次 feed 可能解鎖多個 frame——漏 loop 是「訊息延遲一拍」bug:
    /// 第二個 frame 要等到下一次 read 才被處理)。
    pub fn next_frame(&mut self) -> Result<Option<RawFrame>, DecodeError> {
        match try_decode(&self.buf[self.read_pos..])? {
            Some((frame, consumed)) => {
                self.read_pos += consumed;
                self.maybe_compact();
                Ok(Some(frame))
            }
            None => {
                self.maybe_compact();
                Ok(None) // 殘料(可能半個 frame)留在 buf 等下次 feed
            }
        }
    }

    /// 未消費的殘料量(觀測/測試用)。
    pub fn pending_len(&self) -> usize {
        self.buf.len() - self.read_pos
    }

    /// 壓實策略:已消費前綴 ≥ 4KB 且 ≥ 殘料量時,把殘料搬回開頭。
    /// 搬移成本 O(殘料),但每 byte 至多被搬 O(1) 次(攤銷論證:
    /// 搬移只在消費量 ≥ 殘料量時發生)⇒ 整體攤銷 O(1)/byte。
    fn maybe_compact(&mut self) {
        const COMPACT_THRESHOLD: usize = 4096;
        if self.read_pos >= COMPACT_THRESHOLD && self.read_pos >= self.pending_len() {
            self.buf.drain(..self.read_pos);
            self.read_pos = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hw_bridge::protocol::{Command, MAX_FRAME_LEN, encode_frame};

    fn frames_of(reader: &mut FrameReader) -> Vec<RawFrame> {
        let mut out = Vec::new();
        while let Some(f) = reader.next_frame().unwrap() {
            out.push(f);
        }
        out
    }

    /// [Dry-Run] partial frame:一個 7-byte frame 拆兩次 feed(4+3)。
    /// trace:feed [0,0,0,3,0x02] → try_decode:len=3、total=7 > 5 → None,
    /// 5 bytes 留存 → feed [0x01,0x02] 補齊 → 切出 frame、read_pos=7。
    /// **且窮舉所有切割點**(1..6):任何一個先到的前綴都不能出 frame、
    /// 補齊後都要恰好出一個——這是手寫 framing 最常錯的地方。
    #[test]
    fn boundary_partial_frame_all_split_points() {
        let full = Command::ReadSensor { sensor_id: 0x0102 }.encode();
        for split in 1..full.len() {
            let mut r = FrameReader::new();
            r.feed(&full[..split]);
            assert!(
                frames_of(&mut r).is_empty(),
                "split at {split}: 不該出 frame"
            );
            r.feed(&full[split..]);
            let frames = frames_of(&mut r);
            assert_eq!(frames.len(), 1, "split at {split}: 補齊後恰一個 frame");
            assert_eq!(
                Command::try_from_frame(&frames[0]).unwrap(),
                Command::ReadSensor { sensor_id: 0x0102 }
            );
            assert_eq!(r.pending_len(), 0);
        }
    }

    /// boundary:multiple frames——兩個 frame 一次 feed,一次 loop 全吐出、順序不亂。
    #[test]
    fn boundary_two_frames_single_feed() {
        let mut bytes = Command::Ping.encode();
        bytes.extend(Command::SetFan { rpm: 1200 }.encode());
        let mut r = FrameReader::new();
        r.feed(&bytes);
        let frames = frames_of(&mut r);
        assert_eq!(frames.len(), 2);
        assert_eq!(Command::try_from_frame(&frames[0]).unwrap(), Command::Ping);
        assert_eq!(
            Command::try_from_frame(&frames[1]).unwrap(),
            Command::SetFan { rpm: 1200 }
        );
    }

    /// boundary:byte-by-byte feed(最碎的 stream 切割)——
    /// 3 個 frame 逐 byte 餵,frame 邊界與內容都不能亂。
    #[test]
    fn boundary_byte_by_byte_feed() {
        let cmds = [
            Command::Ping,
            Command::ReadSensor { sensor_id: 42 },
            Command::SetFan { rpm: 800 },
        ];
        let bytes: Vec<u8> = cmds.iter().flat_map(|c| c.encode()).collect();
        let mut r = FrameReader::new();
        let mut got = Vec::new();
        for b in bytes {
            r.feed(&[b]);
            got.extend(frames_of(&mut r));
        }
        let parsed: Vec<Command> = got
            .iter()
            .map(|f| Command::try_from_frame(f).unwrap())
            .collect();
        assert_eq!(parsed, cmds);
    }

    /// boundary:「一個半 frame」一次 feed——完整的先出,半個留著。
    #[test]
    fn boundary_one_and_a_half_frames() {
        let f1 = Command::Ping.encode();
        let f2 = Command::SetFan { rpm: 500 }.encode();
        let mut r = FrameReader::new();
        let mut fed: Vec<u8> = f1.clone();
        fed.extend(&f2[..3]); // 第二個只給 3 bytes
        r.feed(&fed);
        let frames = frames_of(&mut r);
        assert_eq!(frames.len(), 1);
        assert_eq!(r.pending_len(), 3); // 半個 frame 留存
        r.feed(&f2[3..]);
        assert_eq!(frames_of(&mut r).len(), 1);
    }

    /// boundary:malformed len 穿過 framer 上拋(連線該死的訊號)。
    #[test]
    fn boundary_malformed_propagates() {
        let mut r = FrameReader::new();
        r.feed(&(MAX_FRAME_LEN + 9).to_be_bytes());
        assert_eq!(
            r.next_frame(),
            Err(crate::hw_bridge::protocol::DecodeError::FrameTooLarge(
                MAX_FRAME_LEN + 9
            ))
        );
    }

    /// 壓實不破壞語意:灌 3000 個小 frame(> 閾值多次觸發壓實),
    /// 全部完整切出、殘料歸零、內部 buffer 沒有無限長大。
    #[test]
    fn compaction_preserves_stream_and_bounds_memory() {
        let one = encode_frame(0x01, &[]); // 5 bytes
        let mut r = FrameReader::new();
        let mut count = 0;
        for _ in 0..3000 {
            r.feed(&one);
            count += frames_of(&mut r).len();
        }
        assert_eq!(count, 3000);
        assert_eq!(r.pending_len(), 0);
        // 攤銷壓實後,內部 buffer 不應是 3000×5 = 15KB 的量級
        assert!(r.buf.len() < 10 * 1024, "buf grew to {}", r.buf.len());
    }
}
