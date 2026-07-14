//! drill:hw_bridge —— 填 try_decode 與 FrameReader(本主題的評分重心)。
//!
//! server/client 不重挖(讀 reference);這裡專練 framing:
//! - `protocol::try_decode`:從 buf 開頭試切一個 frame
//! - `framer::FrameReader::next_frame`:有狀態的 stream 黏合層
//!
//! Off-by-one 三連環(測試全部釘著):len 含不含自己、殘料偏移、差一 byte 的等待。

pub mod framer;
pub mod protocol;
