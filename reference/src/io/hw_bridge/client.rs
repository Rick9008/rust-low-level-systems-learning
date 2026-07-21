//! sync client:連線、送命令、等對應回應。
//!
//! **同步版的對應規則:一次一個 in-flight + FIFO。**
//! `call()` 送出後阻塞等下一個 response frame——協定沒有 request-id,
//! 「第 n 個回應對應第 n 個請求」是唯一的對應依據。
//!
//! async / pipeline 版怎麼做(doc-only,面試常見 follow-up):
//! 協定加 `request_id`(如 u32,client 遞增),response 原樣帶回;
//! client 維護 `HashMap<request_id, 等待者>`(pending map),
//! 收到 response 就按 id 喚醒對應等待者(oneshot channel / waker)。
//! 這樣多條命令可同時在飛、server 端也能多 worker 亂序完成。

use super::framer::FrameReader;
use super::protocol::{Command, DecodeError, Response, WireError};
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};

#[derive(Debug)]
pub enum ClientError {
    Io(io::Error),
    /// server 回的 byte 流 framing 損毀(協定失去同步)。
    Framing(DecodeError),
    /// frame 完好但不是合法 response(unknown opcode / payload 長度不對)。
    Wire(WireError),
    /// server 回了 Error frame。
    Server {
        code: u8,
    },
    /// 回應型別與請求不匹配(例:Ping 卻回 Ack)——協定違約。
    Unexpected(Response),
    /// 等回應時對端關線。
    ServerClosed,
}

impl From<io::Error> for ClientError {
    fn from(e: io::Error) -> Self {
        ClientError::Io(e)
    }
}

pub struct HwClient {
    stream: TcpStream,
    reader: FrameReader,
}

impl HwClient {
    pub fn connect(addr: SocketAddr) -> io::Result<Self> {
        Ok(Self {
            stream: TcpStream::connect(addr)?, // 阻塞 socket:sync client 的正確選擇
            reader: FrameReader::new(),
        })
    }

    /// 送一條命令、等對應回應(FIFO 對應,見模組 doc)。
    pub fn call(&mut self, cmd: Command) -> Result<Response, ClientError> {
        self.stream.write_all(&cmd.encode())?;
        let mut buf = [0u8; 4096];
        loop {
            // 先看 buffer 裡是否已有完整 frame(上一輪 read 可能多收了)
            match self.reader.next_frame() {
                Ok(Some(frame)) => {
                    return Response::try_from_frame(&frame).map_err(ClientError::Wire);
                }
                Ok(None) => {}
                Err(e) => return Err(ClientError::Framing(e)),
            }
            let n = self.stream.read(&mut buf)?;
            if n == 0 {
                return Err(ClientError::ServerClosed);
            }
            self.reader.feed(&buf[..n]);
        }
    }

    // ---- 型別化的便利介面:call + 回應形狀檢查 ----

    pub fn ping(&mut self) -> Result<(), ClientError> {
        match self.call(Command::Ping)? {
            Response::Pong => Ok(()),
            Response::Error { code } => Err(ClientError::Server { code }),
            other => Err(ClientError::Unexpected(other)),
        }
    }

    /// 回傳毫攝氏度。回應裡的 sensor_id 必須 echo 我們問的那顆——
    /// 便宜的 sanity check,抓「回應錯位」這種 framing/對應 bug。
    pub fn read_sensor(&mut self, sensor_id: u16) -> Result<i32, ClientError> {
        match self.call(Command::ReadSensor { sensor_id })? {
            Response::SensorValue {
                sensor_id: echoed,
                millicelsius,
            } if echoed == sensor_id => Ok(millicelsius),
            Response::Error { code } => Err(ClientError::Server { code }),
            other => Err(ClientError::Unexpected(other)),
        }
    }

    pub fn set_fan(&mut self, rpm: u16) -> Result<(), ClientError> {
        match self.call(Command::SetFan { rpm })? {
            Response::Ack => Ok(()),
            Response::Error { code } => Err(ClientError::Server { code }),
            other => Err(ClientError::Unexpected(other)),
        }
    }

    /// 測試後門:直接送裸 bytes(製造 malformed / partial frame)。
    #[doc(hidden)]
    pub fn send_raw(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.stream.write_all(bytes)
    }

    /// 測試後門:等下一個 response frame(配合 send_raw)。
    #[doc(hidden)]
    pub fn recv_response(&mut self) -> Result<Response, ClientError> {
        let mut buf = [0u8; 4096];
        loop {
            match self.reader.next_frame() {
                Ok(Some(frame)) => {
                    return Response::try_from_frame(&frame).map_err(ClientError::Wire);
                }
                Ok(None) => {}
                Err(e) => return Err(ClientError::Framing(e)),
            }
            let n = self.stream.read(&mut buf)?;
            if n == 0 {
                return Err(ClientError::ServerClosed);
            }
            self.reader.feed(&buf[..n]);
        }
    }
}
