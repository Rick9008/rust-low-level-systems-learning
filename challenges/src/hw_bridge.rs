//! ★ challenge:length-prefix framing(try_decode + FrameReader)
//!
//! 【題目】你在寫硬體控制器的通訊層。wire format 已經定案:
//!
//! ```text
//! [u32 len(big-endian)][u8 opcode][payload:len-1 bytes]
//! ```
//!
//! `len` = opcode + payload 的長度,**不含 len 欄位自己**。
//! TCP 是 byte stream:一次 read 可能拿到半個 frame、一個半、或三個。
//! 實作把 byte stream 重建成 frame stream 的兩層:
//! 1. `try_decode`——無狀態:從 buf 開頭試切一個 frame
//! 2. `FrameReader`——有狀態:feed bytes 進來、依序吐 frame 出去
//!
//! 【constraints】
//! - std-only;無 serde、無 bytes crate
//! - `len == 0` 或 `len > MAX_FRAME_LEN` 是協定損毀(Err;連線會被關)
//! - bytes 不足是**正常狀態**(Ok(None)),不是錯誤——分清楚這兩者
//! - FrameReader 長期運行不可無限吃記憶體(攤銷 O(1)/byte)
//!
//! 【clarify points——動手前先自答】
//! - 「差 1 byte 就完整」時你等在哪個判斷式上?
//! - 切出一個 frame 後,剩餘 bytes 怎麼處理?每次 memmove 會有什麼複雜度問題?
//! - 為什麼 framing 損毀只能斷線(而 unknown opcode 不用)?
//!
//! 【要實作】下方簽名。
//! 【驗收】tests/hw_bridge.rs 轉綠——包含用 **reference 的 server** 當
//! harness 的整合測試:你的 FrameReader 要能在真 TCP 上解析回應。

pub const MAX_FRAME_LEN: u32 = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFrame {
    pub opcode: u8,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    EmptyFrame,
    FrameTooLarge(u32),
}

/// 從 `buf` 開頭試切一個完整 frame。
/// Ok(None) = bytes 還不夠;Ok(Some((frame, consumed))) = 切出一個,
/// caller 應丟棄前 consumed bytes;Err = 協定損毀。
pub fn try_decode(buf: &[u8]) -> Result<Option<(RawFrame, usize)>, DecodeError> {
    todo!("challenge: 從空白開始")
}

pub struct FrameReader {
    // ↓ 佔位:動手時整個換成你的設計。
    _todo: (),
}

impl Default for FrameReader {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameReader {
    pub fn new() -> Self {
        todo!("challenge")
    }

    /// 收下一批剛從 socket 讀到的 bytes。
    pub fn feed(&mut self, bytes: &[u8]) {
        todo!("challenge")
    }

    /// 試切下一個 frame(caller 會 loop 到 Ok(None))。
    pub fn next_frame(&mut self) -> Result<Option<RawFrame>, DecodeError> {
        todo!("challenge")
    }
}
