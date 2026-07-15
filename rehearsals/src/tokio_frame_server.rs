//! rehearsal d:tokio_frame_server —— 題目見 rehearsals/README.md。
//!
//! 唯一用 crate 的一題(tokio;pad 實測清單有)。
//! wire format 同題目 c:`[u32 len(BE)][payload]`,`len == 0` 是 heartbeat。
//! 只給 API 簽名;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`
//! (`#[tokio::test]` 照用)。

use std::io;
use std::time::Duration;
use tokio::net::TcpListener;

/// 服務到 listener 出錯為止;每個連線並發服務、互相獨立。
///
/// - data frame(`len > 0`)→ 原封不動 echo 回去(同 wire format)。
/// - heartbeat(`len == 0`)→ 不回應,但算流量。
/// - 一條連線超過 `idle_timeout` 沒有任何 bytes 進來 → 關閉該連線。
pub async fn serve(listener: TcpListener, idle_timeout: Duration) -> io::Result<()> {
    todo!("rehearsal")
}
