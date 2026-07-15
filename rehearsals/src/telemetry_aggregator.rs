//! rehearsal f:telemetry_aggregator —— 題目見 rehearsals/README.md。
//!
//! 時間用邏輯毫秒(u64),不用 Instant——測試可決定性地控制時間。
//! 只給 API 簽名;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowStats {
    pub count: u64,
    pub sum: i64,
    pub min: i64,
    pub max: i64,
}

pub struct Aggregator {
    // ↓ 佔位:動手時整個換成你的設計。
    _todo: (),
}

impl Aggregator {
    /// `window_ms`:每個 window 的寬度;`num_windows`:保留最近幾個 window。
    /// 記憶體固定 O(num_windows),與樣本數無關。兩者皆 >= 1。
    pub fn new(window_ms: u64, num_windows: usize) -> Self {
        todo!("rehearsal")
    }

    /// 記錄一筆。window 邊界是半開區間 `[k*window_ms, (k+1)*window_ms)`。
    /// ts 落在「已被淘汰的過去」(比保留範圍還舊)→ 回 false 且不記。
    /// ts 跳到未來 → 成為新的最新 window,中間被跳過的 window 視同空。
    pub fn record(&mut self, ts_ms: u64, value: i64) -> bool {
        todo!("rehearsal")
    }

    /// 回傳 ts 所屬 window 的目前統計。
    /// 該 window 沒有任何資料(空、被淘汰、尚未發生)→ None。
    pub fn stats(&self, ts_ms: u64) -> Option<WindowStats> {
        todo!("rehearsal")
    }
}
