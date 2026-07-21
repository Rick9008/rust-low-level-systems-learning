//! drill:wire protocol —— 填 try_decode。
//!
//! Frame 佈局:`[u32 len BE][u8 opcode][payload...]`,
//! **len = 1(opcode)+ payload 長度,不含 len 欄位自己**。
//! 已給:常數、RawFrame/錯誤型別、encode_frame、Command 的 encode。

pub const MAX_FRAME_LEN: u32 = 64 * 1024;

pub const OP_PING: u8 = 0x01;
pub const OP_READ_SENSOR: u8 = 0x02;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFrame {
    pub opcode: u8,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    EmptyFrame,         // len == 0
    FrameTooLarge(u32), // len > MAX_FRAME_LEN
}

/// spec:從 `buf` 開頭試切一個完整 frame。
/// - buf 連 4 bytes 的 len 欄位都不夠 → Ok(None)
/// - len == 0 → Err(EmptyFrame);len > MAX → Err(FrameTooLarge)
/// - buf 不足 `4 + len` bytes(半個 frame)→ Ok(None)
/// - 足夠 → Ok(Some((RawFrame{ opcode: buf[4], payload: buf[5..4+len] },
///   consumed = 4 + len)))
///
/// 提示:`u32::from_be_bytes`、`buf.get(..4)`。
pub fn try_decode(buf: &[u8]) -> Result<Option<(RawFrame, usize)>, DecodeError> {
    // todo!("spec: 讀 len → 驗 len → 等齊 → 切出 (frame, consumed)")
    if buf.len() < 4 {
        return Ok(None);
    }
    let (frame_len, frame) = buf.split_at(4);
    let frame_len = u32::from_be_bytes(
        frame_len
            .try_into()
            .expect("Invariant: buf should not be empty and len of frame_len is 4 by split_at."),
    );
    if frame_len == 0 {
        return Err(DecodeError::EmptyFrame);
    }
    if frame_len > MAX_FRAME_LEN {
        return Err(DecodeError::FrameTooLarge(frame_len));
    }
    if frame.len() < frame_len as usize || frame.is_empty() {
        return Ok(None);
    }
    Ok(Some((
        RawFrame {
            opcode: frame[0],
            payload: frame[1..].to_vec(),
        },
        frame_len as usize + 4,
    )))
}

/// 已給:組 frame。
pub fn encode_frame(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let len = 1 + payload.len();
    let mut out = Vec::with_capacity(4 + len);
    out.extend_from_slice(&(len as u32).to_be_bytes());
    out.push(opcode);
    out.extend_from_slice(payload);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// boundary:精確 byte 佈局(手 trace:len=3 → [0,0,0,3,op,p0,p1])。
    #[test]
    // #[ignore = "填完 try_decode 後移除"]
    fn exact_layout_roundtrip() {
        let bytes = encode_frame(OP_READ_SENSOR, &[0x01, 0x02]);
        assert_eq!(bytes, vec![0, 0, 0, 3, OP_READ_SENSOR, 0x01, 0x02]);
        let (frame, consumed) = try_decode(&bytes).unwrap().unwrap();
        assert_eq!(consumed, 7);
        assert_eq!(frame.opcode, OP_READ_SENSOR);
        assert_eq!(frame.payload, vec![0x01, 0x02]);
    }

    /// boundary:**每一個**不完整前綴都回 Ok(None)。
    #[test]
    // #[ignore = "填完 try_decode 後移除"]
    fn every_partial_prefix_is_none() {
        let full = encode_frame(OP_PING, &[9, 9, 9]);
        for cut in 0..full.len() {
            assert_eq!(try_decode(&full[..cut]).unwrap(), None, "prefix {cut}");
        }
    }

    /// boundary:兩個 frame 背靠背——consumed 恰好指到界線。
    #[test]
    // #[ignore = "填完 try_decode 後移除"]
    fn two_frames_consumed_exact() {
        let mut buf = encode_frame(OP_PING, &[]);
        buf.extend(encode_frame(OP_READ_SENSOR, &[1, 2]));
        let (f1, n1) = try_decode(&buf).unwrap().unwrap();
        assert_eq!(f1.opcode, OP_PING);
        let (f2, n2) = try_decode(&buf[n1..]).unwrap().unwrap();
        assert_eq!(f2.opcode, OP_READ_SENSOR);
        assert_eq!(n1 + n2, buf.len());
    }

    /// boundary:malformed len(0 與超大)是 Err 不是 None。
    #[test]
    // #[ignore = "填完 try_decode 後移除"]
    fn malformed_len_is_err() {
        assert_eq!(
            try_decode(&0u32.to_be_bytes()),
            Err(DecodeError::EmptyFrame)
        );
        assert_eq!(
            try_decode(&(MAX_FRAME_LEN + 1).to_be_bytes()),
            Err(DecodeError::FrameTooLarge(MAX_FRAME_LEN + 1))
        );
    }
}
