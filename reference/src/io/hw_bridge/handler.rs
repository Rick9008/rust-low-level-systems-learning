//! command handler:協定層與「硬體」的邊界。
//!
//! [Abstract the Noise] 落地處:面試時真硬體(暫存器讀寫、I2C、韌體佇列)
//! 不存在也不重要——用 trait 隔開,mock 一個確定性的假硬體往前走。
//! server 只依賴 `CommandHandler`,換真硬體時 server 一行不改。

use super::protocol::{Command, Response};

/// server 與硬體之間唯一的介面。
///
/// `&mut self`:硬體控制器天然是有狀態的序列設備(一次一條命令);
/// 並發策略(鎖、專用執行緒)由 server 端決定,不滲進硬體抽象。
pub trait CommandHandler: Send {
    fn handle(&mut self, cmd: Command) -> Response;
}

/// 假硬體:確定性回應(測試可預測),並記錄副作用(SetFan 可驗證)。
#[derive(Default)]
pub struct MockHardware {
    pub fan_rpm: u16,
}

impl MockHardware {
    /// 假 sensor 讀值:id 的確定性函數(id×1000 - 273 毫攝氏度),
    /// 測試端可以重算預期值。
    pub fn sensor_value(sensor_id: u16) -> i32 {
        i32::from(sensor_id) * 1000 - 273
    }
}

impl CommandHandler for MockHardware {
    fn handle(&mut self, cmd: Command) -> Response {
        match cmd {
            Command::Ping => Response::Pong,
            Command::ReadSensor { sensor_id } => Response::SensorValue {
                sensor_id,
                millicelsius: Self::sensor_value(sensor_id),
            },
            Command::SetFan { rpm } => {
                self.fan_rpm = rpm; // 副作用:測試由此驗證命令真的到了
                Response::Ack
            }
        }
    }
}

/// 「handler 內部要做阻塞 IO」的模擬硬體:ReadSensor 走慢速匯流排
/// (sleep 模擬),Ping / SetFan 快路徑。
///
/// 用途:handler-IO 對照組的教具——inline 執行時它毒死整個 event loop
/// (所有連線陪等),offload / shard 之後只有自己的連線等。
/// 見 `server_evented_inline`(反面教材)與 `server_evented_sharded`。
pub struct SlowHardware {
    delay: std::time::Duration,
    inner: MockHardware,
}

impl SlowHardware {
    pub fn new(delay: std::time::Duration) -> Self {
        Self {
            delay,
            inner: MockHardware::default(),
        }
    }
}

impl CommandHandler for SlowHardware {
    fn handle(&mut self, cmd: Command) -> Response {
        if matches!(cmd, Command::ReadSensor { .. }) {
            std::thread::sleep(self.delay); // 模擬慢速匯流排 / 磁碟 / 下游 RPC
        }
        self.inner.handle(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [Dry-Run] mock 的三條路:Ping→Pong、ReadSensor 確定性值、
    /// SetFan 副作用可觀察。
    #[test]
    fn mock_hardware_deterministic() {
        let mut hw = MockHardware::default();
        assert_eq!(hw.handle(Command::Ping), Response::Pong);
        assert_eq!(
            hw.handle(Command::ReadSensor { sensor_id: 25 }),
            Response::SensorValue {
                sensor_id: 25,
                millicelsius: 24_727
            }
        );
        assert_eq!(hw.handle(Command::SetFan { rpm: 4500 }), Response::Ack);
        assert_eq!(hw.fan_rpm, 4500);
    }
}
