//! rehearsal c:frame_parser_heartbeat —— 題目見 rehearsals/README.md。
//!
//! wire format:`[u32 len(BE)][payload:len bytes]`,`len == 0` 是 heartbeat。
//! 只給 API 簽名;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`。

#[derive(Debug, PartialEq, Eq)]
pub enum Frame {
    /// `len == 0`,無 payload。
    Heartbeat,
    /// `len > 0`,帶完整 payload。
    Data(Vec<u8>),
}

pub struct FrameParser {
    // ↓ 佔位:動手時整個換成你的設計。
    _todo: (),
}

impl FrameParser {
    pub fn new() -> Self {
        todo!("rehearsal")
    }

    /// 吃進這次新到的 bytes,回傳**這次新完成**的所有 frame(依 stream 順序)。
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Frame> {
        todo!("rehearsal")
    }
}

impl Default for FrameParser {
    fn default() -> Self {
        Self::new()
    }
}
