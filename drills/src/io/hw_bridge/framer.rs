//! drill:framer —— 填 FrameReader::next_frame(有狀態的 stream 黏合)。
//!
//! 已給:結構(buf + read_pos 游標)、feed、壓實。
//! 要填:`next_frame`。呼叫端會 loop 到 Ok(None)——你只管「切一個」。

use super::protocol::{DecodeError, RawFrame, try_decode};

pub struct FrameReader {
    buf: Vec<u8>,
    /// 已消費前綴(邏輯上 buf[..read_pos] 已切走;壓實時才真的搬)。
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

    pub fn feed(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// spec:對「未消費區間」`&self.buf[self.read_pos..]` 呼叫 try_decode:
    /// - Ok(Some((frame, consumed))) → read_pos += consumed、
    ///   `self.maybe_compact()`、回 Ok(Some(frame))
    /// - Ok(None) → 殘料留著(半個 frame 等下次 feed)、回 Ok(None)
    /// - Err → 原樣上拋(framing 損毀,連線該關)
    pub fn next_frame(&mut self) -> Result<Option<RawFrame>, DecodeError> {
        // todo!("spec: try_decode 未消費區間;成功前進游標")
        match try_decode(&self.buf[self.read_pos..])? {
            Some((frame, consumed)) => {
                self.read_pos += consumed;
                self.maybe_compact();
                Ok(Some(frame))
            }
            None => Ok(None),
        }
    }

    pub fn pending_len(&self) -> usize {
        self.buf.len() - self.read_pos
    }

    /// 已給:攤銷壓實(消費前綴夠大時把殘料搬回開頭)。
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
    use crate::io::hw_bridge::protocol::{OP_PING, OP_READ_SENSOR, encode_frame};

    fn drain_frames(r: &mut FrameReader) -> Vec<RawFrame> {
        let mut out = Vec::new();
        while let Some(f) = r.next_frame().unwrap() {
            out.push(f);
        }
        out
    }

    /// boundary:partial frame 窮舉所有切割點。
    #[test]
    // #[ignore = "填完 next_frame 後移除"]
    fn partial_all_split_points() {
        let full = encode_frame(OP_READ_SENSOR, &[1, 2]);
        for split in 1..full.len() {
            let mut r = FrameReader::new();
            r.feed(&full[..split]);
            assert!(drain_frames(&mut r).is_empty(), "split {split}");
            r.feed(&full[split..]);
            assert_eq!(drain_frames(&mut r).len(), 1, "split {split}");
            assert_eq!(r.pending_len(), 0);
        }
    }

    /// boundary:一次 feed 兩個 frame + 半個——完整的全出、殘料正確留存。
    #[test]
    // #[ignore = "填完 next_frame 後移除"]
    fn two_and_a_half_frames() {
        let f = encode_frame(OP_PING, &[]);
        let mut bytes: Vec<u8> = f.iter().chain(f.iter()).copied().collect();
        bytes.extend(&f[..2]);
        let mut r = FrameReader::new();
        r.feed(&bytes);
        assert_eq!(drain_frames(&mut r).len(), 2);
        assert_eq!(r.pending_len(), 2);
        r.feed(&f[2..]);
        assert_eq!(drain_frames(&mut r).len(), 1);
    }

    /// boundary:byte-by-byte 餵三個 frame。
    #[test]
    // #[ignore = "填完 next_frame 後移除"]
    fn byte_by_byte() {
        let frames = [
            encode_frame(OP_PING, &[]),
            encode_frame(OP_READ_SENSOR, &[7, 7]),
            encode_frame(OP_PING, &[1]),
        ];
        let bytes: Vec<u8> = frames.iter().flatten().copied().collect();
        let mut r = FrameReader::new();
        let mut got = 0;
        for b in bytes {
            r.feed(&[b]);
            got += drain_frames(&mut r).len();
        }
        assert_eq!(got, 3);
    }

    /// boundary:一個半 frame——完整的先出,殘料(半個)留在 buffer,
    /// 補齊後第二個才出。pending_len 是可觀察的殘料量。
    #[test]
    // #[ignore = "填完 next_frame 後移除"]
    fn one_and_a_half_frames() {
        let f1 = encode_frame(OP_PING, &[]);
        let f2 = encode_frame(OP_READ_SENSOR, &[0, 25]);
        let mut r = FrameReader::new();
        let mut fed = f1.clone();
        fed.extend(&f2[..3]); // 第二個只給 3 bytes
        r.feed(&fed);
        assert_eq!(drain_frames(&mut r).len(), 1);
        assert_eq!(r.pending_len(), 3, "半個 frame 留存");
        r.feed(&f2[3..]);
        assert_eq!(drain_frames(&mut r).len(), 1);
    }

    /// boundary:壓實回收已消費前綴(counterexample:漏呼叫 maybe_compact 會紅)。
    /// 手走:50 frame × 101 bytes = 5050 bytes 一次餵入,逐個 drain。
    /// 第 41 個成功後 read_pos = 4141 ≥ 4096 且 ≥ pending(909)→ 壓實應觸發:
    /// buf 縮到 909、read_pos 歸 0;之後 read_pos 最多再爬到 909。
    /// 若 next_frame 沒呼叫 maybe_compact,read_pos 會停在 5050、buf 全額 5050 不回收
    /// ——長連線等同記憶體無上限成長。
    #[test]
    fn compaction_reclaims_consumed_prefix() {
        let f = encode_frame(OP_READ_SENSOR, &[0xAB; 96]); // 4 + 1 + 96 = 101 bytes
        let total: usize = f.len() * 50;
        assert!(total > 4096, "前提:總量要跨過壓實門檻");
        let mut r = FrameReader::new();
        for _ in 0..50 {
            r.feed(&f);
        }
        assert_eq!(drain_frames(&mut r).len(), 50);
        assert_eq!(r.pending_len(), 0);
        assert!(
            r.read_pos < 4096,
            "已消費前綴應被壓實回收,實際 read_pos = {}",
            r.read_pos
        );
        assert!(
            r.buf.len() < total,
            "buf 應縮小,實際仍持有 {} bytes",
            r.buf.len()
        );
    }

    /// boundary:malformed len(離譜大)——錯誤傳播給 caller
    /// (framing 層致命:byte 流已不可信,caller 的正解是關線)。
    #[test]
    // #[ignore = "填完 next_frame 後移除"]
    fn malformed_len_propagates_error() {
        let mut r = FrameReader::new();
        r.feed(&u32::MAX.to_be_bytes());
        assert!(r.next_frame().is_err());
    }
}
