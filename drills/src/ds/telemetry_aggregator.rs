//! drill:telemetry_aggregator —— 滑動視窗統計(= rehearsal f 題同一份 contract)。
//!
//! 時間用邏輯毫秒(u64)——測試可決定性地控制;真實系統把 now 換成 Instant。
//! 已給:struct、Bucket、new、bucket_index。要填:`record` / `stats`。
//! 填綠(移光 `#[ignore]`)後,彩排覆蓋帳的 f 格就算親手寫過;
//! 之後拿 `rehearsals/src/telemetry_aggregator.rs` 的空白骨架重寫即是 f 的計時版。
//!
//! 記憶體模型:固定 num_windows 個桶,桶按 `epoch % num_windows` 重用——
//! 與樣本數無關的 O(num_windows),這句就是 f 題 trade-off 收尾的第一句。

/// 一個 window 的統計輸出。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowStats {
    pub count: u64,
    pub sum: i64,
    pub min: i64,
    pub max: i64,
}

/// 一個時間桶。`epoch = ts / window_ms`(第幾個 window)。
/// `epoch == u64::MAX` 當「無主」哨兵:這個桶還沒被任何 window 用過。
struct Bucket {
    epoch: u64,
    count: u64,
    sum: i64,
    min: i64,
    max: i64,
}

impl Bucket {
    fn empty() -> Self {
        Bucket {
            epoch: u64::MAX,
            count: 0,
            sum: 0,
            min: 0,
            max: 0,
        }
    }
}

pub struct Aggregator {
    window_ms: u64,
    buckets: Vec<Bucket>,
    /// 目前見過的最新 epoch;`started == false` 時無意義。
    latest_epoch: u64,
    started: bool,
}

impl Aggregator {
    /// `window_ms`:每個 window 的寬度;`num_windows`:保留最近幾個 window。
    /// 記憶體固定 O(num_windows),與樣本數無關。兩者皆 >= 1。
    pub fn new(window_ms: u64, num_windows: usize) -> Self {
        assert!(window_ms >= 1 && num_windows >= 1);
        Self {
            window_ms,
            buckets: (0..num_windows).map(|_| Bucket::empty()).collect(),
            latest_epoch: 0,
            started: false,
        }
    }

    fn bucket_index(&self, epoch: u64) -> usize {
        (epoch % self.buckets.len() as u64) as usize
    }

    /// spec:記錄一筆。window 邊界是半開區間 `[k*window_ms, (k+1)*window_ms)`。
    ///
    /// 1. `e = ts_ms / window_ms`。第一筆資料:`latest_epoch = e`,直接記。
    /// 2. `e > latest_epoch`(未來):latest 推進到 e——**被跳過的 window 視同空**,
    ///    對應的桶要能被之後的 stats 判成 None(提示:桶的 epoch 對不上就是舊資料,
    ///    寫入前重置)。這就是「未來 ts 清 window」case。
    /// 3. `e < latest_epoch` 且 `latest_epoch - e >= num_windows`(掉出保留範圍)
    ///    → 回 false 且不記(注意用減法判斷,別讓 u64 underflow)。
    /// 4. 其餘(在保留範圍內,含亂序補記)→ 累進該桶的 count/sum/min/max。
    ///    寫入前同樣要驗桶的 epoch:桶裡若躺著更舊 epoch 的殘料,先重置再記。
    pub fn record(&mut self, ts_ms: u64, value: i64) -> bool {
        let _ = (ts_ms, value);
        todo!("spec: 見上——先算 epoch,分四條路:首筆 / 未來推進 / 太舊拒收 / 範圍內累進")
    }

    /// spec:回傳 ts 所屬 window 的目前統計。
    /// 以下一律 None:還沒有任何資料、該 epoch 比保留範圍舊(被淘汰)、
    /// 比 latest 新(尚未發生)、桶是空的(被跳過或沒人記過)。
    /// 提示:桶的 `epoch` 必須恰好等於查詢的 epoch 且 `count > 0` 才算有料。
    pub fn stats(&self, ts_ms: u64) -> Option<WindowStats> {
        let _ = ts_ms;
        todo!("spec: 算 epoch → 四種 None 擋掉 → Some(桶的四個數)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// boundary:同 window 累進 + 半開區間邊界。
    /// trace:window=100 → epoch(99)=0、epoch(100)=1——99 和 100 不同桶。
    #[test]
    #[ignore = "填完 record/stats 後移除"]
    fn same_window_accumulates_and_boundary_is_half_open() {
        let mut a = Aggregator::new(100, 4);
        assert!(a.record(10, 5));
        assert!(a.record(99, 7)); // 同 epoch 0
        assert!(a.record(100, 2)); // epoch 1,新桶
        assert_eq!(
            a.stats(50),
            Some(WindowStats {
                count: 2,
                sum: 12,
                min: 5,
                max: 7
            })
        );
        assert_eq!(
            a.stats(100),
            Some(WindowStats {
                count: 1,
                sum: 2,
                min: 2,
                max: 2
            })
        );
    }

    /// boundary:掉出保留範圍的舊 ts 拒收;被淘汰的 window 查詢得 None。
    /// trace:window=100、N=2,latest 推到 epoch 2 → 保留 {1,2},epoch 0 已死。
    #[test]
    #[ignore = "填完 record/stats 後移除"]
    fn too_old_is_rejected_and_evicted_window_is_none() {
        let mut a = Aggregator::new(100, 2);
        assert!(a.record(0, 1)); // epoch 0
        assert!(a.record(250, 3)); // epoch 2 → 淘汰 epoch 0
        assert!(!a.record(50, 9)); // epoch 0 已出範圍 → false
        assert_eq!(a.stats(50), None);
        assert!(a.record(150, 8)); // epoch 1 仍在保留範圍(亂序補記 OK)
        assert_eq!(a.stats(150).unwrap().sum, 8);
    }

    /// boundary:未來 ts 清 window——f 題 contract 的招牌 case。
    /// trace:epoch 0 記一筆後跳到 epoch 9:中間全空、epoch 0 淘汰、
    /// 桶重用(9 % 4 = 1)不得殘留舊料。
    #[test]
    #[ignore = "填完 record/stats 後移除"]
    fn future_ts_clears_skipped_windows() {
        let mut a = Aggregator::new(100, 4);
        assert!(a.record(0, 1));
        assert!(a.record(999, 5)); // epoch 9,保留 {6,7,8,9}
        assert_eq!(a.stats(0), None); // 被淘汰
        assert_eq!(a.stats(750), None); // 被跳過 → 視同空
        assert_eq!(a.stats(999).unwrap().sum, 5);
        assert_eq!(a.stats(1100), None); // 尚未發生
    }

    /// boundary:桶重用不串味 + 負值的 min/max。
    /// trace:N=2,epoch 0→1→2:epoch 2 落回 index 0,舊的 epoch 0 資料必須消失。
    #[test]
    #[ignore = "填完 record/stats 後移除"]
    fn bucket_reuse_does_not_leak_and_negatives_work() {
        let mut a = Aggregator::new(100, 2);
        assert!(a.record(0, 1)); // epoch 0 → index 0
        assert!(a.record(100, 2)); // epoch 1 → index 1
        assert!(a.record(200, -4)); // epoch 2 → index 0(重用)
        assert_eq!(
            a.stats(200),
            Some(WindowStats {
                count: 1,
                sum: -4,
                min: -4,
                max: -4
            })
        );
        assert!(a.record(210, 3));
        let s = a.stats(299).unwrap();
        assert_eq!((s.count, s.sum, s.min, s.max), (2, -1, -4, 3));
    }

    /// boundary:全新 aggregator 什麼都查不到。
    #[test]
    #[ignore = "填完 record/stats 後移除"]
    fn fresh_aggregator_returns_none() {
        let a = Aggregator::new(100, 4);
        assert_eq!(a.stats(0), None);
    }
}
