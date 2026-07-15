//! solution:題 f telemetry_aggregator——**寫完彩排才開**。
//! canonical 設計:W 個 slot 的環,`slot = window_num % W`;每個 slot 記住自己的
//! window_num(防 slot 重用讀到上一輪舊資料);前進時清掉被跳過的 slot;
//! 空 window 回 None——絕不 zero-init(min/max 為 0 是毒藥)。
//! 驗證:rehearsals/tests/telemetry_aggregator_test.rs 全綠。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowStats {
    pub count: u64,
    pub sum: i64,
    pub min: i64,
    pub max: i64,
}

struct Slot {
    window_num: u64,
    stats: WindowStats,
}

pub struct Aggregator {
    window_ms: u64,
    slots: Vec<Option<Slot>>,
    newest: Option<u64>, // 目前見過的最新 window 編號
}

impl Aggregator {
    pub fn new(window_ms: u64, num_windows: usize) -> Self {
        assert!(window_ms >= 1 && num_windows >= 1);
        Self {
            window_ms,
            slots: (0..num_windows).map(|_| None).collect(),
            newest: None,
        }
    }

    fn in_range(&self, w: u64) -> bool {
        match self.newest {
            None => false,
            Some(n) => w <= n && w + self.slots.len() as u64 > n,
        }
    }

    pub fn record(&mut self, ts_ms: u64, value: i64) -> bool {
        let w = ts_ms / self.window_ms;
        let cap = self.slots.len() as u64;
        match self.newest {
            Some(n) if w + cap <= n => return false, // 已淘汰的過去:拒絕
            Some(n) if w > n => {
                // 前進到新 window:被跳過、將進入保留範圍的 slot 要先清
                let start = if w - n >= cap { w - (cap - 1) } else { n + 1 };
                for k in start..=w {
                    self.slots[(k % cap) as usize] = None;
                }
                self.newest = Some(w);
            }
            None => self.newest = Some(w),
            _ => {} // 亂序但仍在保留範圍:直接記
        }
        let slot = &mut self.slots[(w % cap) as usize];
        match slot {
            Some(s) if s.window_num == w => {
                s.stats.count += 1;
                s.stats.sum += value;
                s.stats.min = s.stats.min.min(value);
                s.stats.max = s.stats.max.max(value);
            }
            _ => {
                *slot = Some(Slot {
                    window_num: w,
                    stats: WindowStats {
                        count: 1,
                        sum: value,
                        min: value,
                        max: value,
                    },
                });
            }
        }
        true
    }

    pub fn stats(&self, ts_ms: u64) -> Option<WindowStats> {
        let w = ts_ms / self.window_ms;
        if !self.in_range(w) {
            return None;
        }
        match &self.slots[(w % self.slots.len() as u64) as usize] {
            Some(s) if s.window_num == w => Some(s.stats),
            _ => None,
        }
    }
}

fn main() {
    let mut a = Aggregator::new(100, 4);
    assert!(a.record(50, 7)); // window 0
    assert!(a.record(550, 30)); // 跳到 window 5:0 被淘汰、slot 重用要乾淨
    assert_eq!(a.stats(50), None);
    assert_eq!(a.stats(550).map(|s| s.sum), Some(30));
    assert_eq!(a.stats(450), None); // 被跳過的 window 視同空
    assert!(!a.record(150, 9)); // 已淘汰的過去:拒絕
    println!("sol_telemetry_aggregator: ok");
}
