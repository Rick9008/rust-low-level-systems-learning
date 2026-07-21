//! wire protocol:frame 佈局 + opcode 定義 + encode / try_decode。
//!
//! Frame 佈局(所有多 byte 整數一律 big-endian,network byte order):
//!
//! ```text
//! ┌──────────────┬───────────┬──────────────────┐
//! │ len: u32 BE  │ opcode:u8 │ payload: len-1 B │
//! └──────────────┴───────────┴──────────────────┘
//! ```
//!
//! `len` = opcode + payload 的 byte 數(**不含 len 欄位自己**)。
//! 這是本協定唯一的 off-by-one 決策點,定義寫死在這一行註解裡;
//! 「含不含自己」兩種都常見,錯拿對方的定義就是 4-byte 錯位、永不同步。

/// 防禦線:宣稱超大 len 的 frame(攻擊或 bug)直接視為協定損毀,
/// 不然 server 會乖乖預留 4GB 緩衝等它。64KB 對控制協定綽綽有餘。
pub const MAX_FRAME_LEN: u32 = 64 * 1024;

// ---- opcodes:command(client → server)最高位 0,response 最高位 1 ----
pub const OP_PING: u8 = 0x01;
pub const OP_READ_SENSOR: u8 = 0x02;
pub const OP_SET_FAN: u8 = 0x03;

pub const OP_PONG: u8 = 0x81;
pub const OP_SENSOR_VALUE: u8 = 0x82;
pub const OP_ACK: u8 = 0x83;
pub const OP_ERROR: u8 = 0xFF;

// Error response 的 code
pub const ERR_UNKNOWN_OPCODE: u8 = 1;
pub const ERR_BAD_PAYLOAD: u8 = 2;

/// 切出來、還沒做型別解析的 frame(framing 層與語意層分離:
/// framing 錯 = 連線報廢;語意錯 = 回 Error frame,連線活著)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFrame {
    pub opcode: u8,
    pub payload: Vec<u8>,
}

/// framing 層的致命錯誤:byte 流已不可信,唯一正解是關連線
/// (length-prefix 協定沒有 resync 點——你不知道下一個 frame 從哪開始)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// len == 0:連 opcode 都沒有,非法。
    EmptyFrame,
    /// len > MAX_FRAME_LEN:宣稱的大小離譜。
    FrameTooLarge(u32),
}

/// 語意層錯誤:frame 完好,內容不認得——回 Error frame,連線繼續。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireError {
    UnknownOpcode(u8),
    BadPayloadLen {
        opcode: u8,
        expected: usize,
        got: usize,
    },
}

/// **本協定的核心函式**:從 `buf` 開頭試切一個完整 frame。
///
/// 回傳:
/// - `Ok(None)`——bytes 還不夠一個完整 frame(半個 frame):**留著等下次**。
/// - `Ok(Some((frame, consumed)))`——切出一個 frame,呼叫端把 buf 前
///   `consumed` bytes 丟掉,**剩餘 bytes 保留**(可能是下一個 frame 的開頭)。
/// - `Err(_)`——framing 損毀,連線該關。
///
/// O(len) 時間(payload 拷貝一次)、零狀態(狀態在呼叫端的 buffer)。
pub fn try_decode(buf: &[u8]) -> Result<Option<(RawFrame, usize)>, DecodeError> {
    // 連 len 欄位都不齊:等。
    let Some(len_bytes) = buf.get(..4) else {
        return Ok(None);
    };
    let len = u32::from_be_bytes(len_bytes.try_into().expect("4 bytes"));
    if len == 0 {
        return Err(DecodeError::EmptyFrame);
    }
    if len > MAX_FRAME_LEN {
        return Err(DecodeError::FrameTooLarge(len));
    }
    let total = 4 + len as usize; // len 不含自己 ⇒ 整個 frame 佔 4+len
    if buf.len() < total {
        return Ok(None); // 半個 frame:等更多 bytes
    }
    Ok(Some((
        RawFrame {
            opcode: buf[4],
            payload: buf[5..total].to_vec(),
        },
        total,
    )))
}

/// 組 frame:`[len(BE)][opcode][payload]`。O(len)。
pub fn encode_frame(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let len = 1 + payload.len(); // opcode + payload,不含 len 欄位
    debug_assert!(len as u32 <= MAX_FRAME_LEN);
    let mut out = Vec::with_capacity(4 + len);
    out.extend_from_slice(&(len as u32).to_be_bytes());
    out.push(opcode);
    out.extend_from_slice(payload);
    out
}

/// client → server 的命令(模擬硬體控制器的 opcode 表)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Ping,
    ReadSensor { sensor_id: u16 },
    SetFan { rpm: u16 },
}

impl Command {
    /// 手寫序列化:整數一律 to_be_bytes,無 serde。
    pub fn encode(&self) -> Vec<u8> {
        match *self {
            Command::Ping => encode_frame(OP_PING, &[]),
            Command::ReadSensor { sensor_id } => {
                encode_frame(OP_READ_SENSOR, &sensor_id.to_be_bytes())
            }
            Command::SetFan { rpm } => encode_frame(OP_SET_FAN, &rpm.to_be_bytes()),
        }
    }

    /// 語意解析:opcode 分派 + payload 長度嚴格檢查(多一 byte 都不收——
    /// 寬鬆解析會把「兩端版本不一致」這種 bug 藏到很深)。
    pub fn try_from_frame(f: &RawFrame) -> Result<Command, WireError> {
        let expect = |n: usize| -> Result<(), WireError> {
            if f.payload.len() == n {
                Ok(())
            } else {
                Err(WireError::BadPayloadLen {
                    opcode: f.opcode,
                    expected: n,
                    got: f.payload.len(),
                })
            }
        };
        match f.opcode {
            OP_PING => {
                expect(0)?;
                Ok(Command::Ping)
            }
            OP_READ_SENSOR => {
                expect(2)?;
                Ok(Command::ReadSensor {
                    sensor_id: u16::from_be_bytes([f.payload[0], f.payload[1]]),
                })
            }
            OP_SET_FAN => {
                expect(2)?;
                Ok(Command::SetFan {
                    rpm: u16::from_be_bytes([f.payload[0], f.payload[1]]),
                })
            }
            other => Err(WireError::UnknownOpcode(other)),
        }
    }
}

/// server → client 的回應。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Response {
    Pong,
    SensorValue { sensor_id: u16, millicelsius: i32 },
    Ack,
    Error { code: u8 },
}

impl Response {
    pub fn encode(&self) -> Vec<u8> {
        match *self {
            Response::Pong => encode_frame(OP_PONG, &[]),
            Response::SensorValue {
                sensor_id,
                millicelsius,
            } => {
                let mut payload = [0u8; 6];
                payload[..2].copy_from_slice(&sensor_id.to_be_bytes());
                payload[2..].copy_from_slice(&millicelsius.to_be_bytes());
                encode_frame(OP_SENSOR_VALUE, &payload)
            }
            Response::Ack => encode_frame(OP_ACK, &[]),
            Response::Error { code } => encode_frame(OP_ERROR, &[code]),
        }
    }

    pub fn try_from_frame(f: &RawFrame) -> Result<Response, WireError> {
        let expect = |n: usize| -> Result<(), WireError> {
            if f.payload.len() == n {
                Ok(())
            } else {
                Err(WireError::BadPayloadLen {
                    opcode: f.opcode,
                    expected: n,
                    got: f.payload.len(),
                })
            }
        };
        match f.opcode {
            OP_PONG => {
                expect(0)?;
                Ok(Response::Pong)
            }
            OP_SENSOR_VALUE => {
                expect(6)?;
                Ok(Response::SensorValue {
                    sensor_id: u16::from_be_bytes([f.payload[0], f.payload[1]]),
                    millicelsius: i32::from_be_bytes([
                        f.payload[2],
                        f.payload[3],
                        f.payload[4],
                        f.payload[5],
                    ]),
                })
            }
            OP_ACK => {
                expect(0)?;
                Ok(Response::Ack)
            }
            OP_ERROR => {
                expect(1)?;
                Ok(Response::Error { code: f.payload[0] })
            }
            other => Err(WireError::UnknownOpcode(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [Dry-Run] 手 trace:ReadSensor{sensor_id=0x0102} 的 wire bytes——
    ///   payload = [0x01, 0x02](u16 BE)
    ///   len = 1(opcode)+ 2 = 3 → BE bytes [0,0,0,3]
    ///   frame = [0,0,0,3, 0x02, 0x01, 0x02](共 7 bytes)
    /// try_decode 吃回去:len=3、total=7、opcode=0x02、payload=[1,2] ✓
    #[test]
    fn encode_bytes_exact_layout() {
        let bytes = Command::ReadSensor { sensor_id: 0x0102 }.encode();
        assert_eq!(bytes, vec![0, 0, 0, 3, OP_READ_SENSOR, 0x01, 0x02]);
        let (frame, consumed) = try_decode(&bytes).unwrap().unwrap();
        assert_eq!(consumed, 7);
        assert_eq!(
            Command::try_from_frame(&frame).unwrap(),
            Command::ReadSensor { sensor_id: 0x0102 }
        );
    }

    /// 全 command / response roundtrip(encode → try_decode → typed parse)。
    /// boundary:負數溫度(i32 BE 的符號位)、u16 極值。
    #[test]
    fn all_variants_roundtrip() {
        let cmds = [
            Command::Ping,
            Command::ReadSensor { sensor_id: 0 },
            Command::ReadSensor {
                sensor_id: u16::MAX,
            },
            Command::SetFan { rpm: 12000 },
        ];
        for cmd in cmds {
            let bytes = cmd.encode();
            let (frame, n) = try_decode(&bytes).unwrap().unwrap();
            assert_eq!(n, bytes.len());
            assert_eq!(Command::try_from_frame(&frame).unwrap(), cmd);
        }
        let resps = [
            Response::Pong,
            Response::SensorValue {
                sensor_id: 3,
                millicelsius: -40_000, // 負溫度:BE 符號位
            },
            Response::Ack,
            Response::Error { code: 2 },
        ];
        for resp in resps {
            let bytes = resp.encode();
            let (frame, _) = try_decode(&bytes).unwrap().unwrap();
            assert_eq!(Response::try_from_frame(&frame).unwrap(), resp);
        }
    }

    /// boundary:**每一個不完整前綴長度**都回 Ok(None)——半個 len、
    /// 只有 len、len+opcode 但 payload 缺一 byte……全部要「等」而非錯。
    #[test]
    fn boundary_every_partial_prefix_returns_none() {
        let full = Command::ReadSensor { sensor_id: 7 }.encode(); // 7 bytes
        for cut in 0..full.len() {
            assert_eq!(
                try_decode(&full[..cut]).unwrap(),
                None,
                "prefix of {cut} bytes should be incomplete"
            );
        }
    }

    /// boundary:一個 buffer 兩個 frame——第一次切完 consumed 指到界線,
    /// 剩餘 bytes 恰是第二個 frame。
    #[test]
    fn boundary_two_frames_back_to_back() {
        let mut buf = Command::Ping.encode();
        buf.extend(Command::SetFan { rpm: 900 }.encode());
        let (f1, n1) = try_decode(&buf).unwrap().unwrap();
        assert_eq!(Command::try_from_frame(&f1).unwrap(), Command::Ping);
        let (f2, n2) = try_decode(&buf[n1..]).unwrap().unwrap();
        assert_eq!(
            Command::try_from_frame(&f2).unwrap(),
            Command::SetFan { rpm: 900 }
        );
        assert_eq!(n1 + n2, buf.len());
    }

    /// boundary:malformed——len 超大 / len=0,framing 層直接判死。
    #[test]
    fn boundary_malformed_len_is_fatal() {
        let huge = (MAX_FRAME_LEN + 1).to_be_bytes();
        assert_eq!(
            try_decode(&huge),
            Err(DecodeError::FrameTooLarge(MAX_FRAME_LEN + 1))
        );
        assert_eq!(
            try_decode(&0u32.to_be_bytes()),
            Err(DecodeError::EmptyFrame)
        );
    }

    /// boundary:unknown opcode / payload 長度不符——frame 層 OK、語意層報錯
    /// (連線不必死,回 Error frame 即可)。
    #[test]
    fn boundary_semantic_errors_are_recoverable() {
        let bytes = encode_frame(0x7E, &[1, 2, 3]);
        let (frame, _) = try_decode(&bytes).unwrap().unwrap();
        assert_eq!(
            Command::try_from_frame(&frame),
            Err(WireError::UnknownOpcode(0x7E))
        );
        let bytes = encode_frame(OP_READ_SENSOR, &[1]); // 該 2 bytes 只給 1
        let (frame, _) = try_decode(&bytes).unwrap().unwrap();
        assert_eq!(
            Command::try_from_frame(&frame),
            Err(WireError::BadPayloadLen {
                opcode: OP_READ_SENSOR,
                expected: 2,
                got: 1
            })
        );
    }
}
