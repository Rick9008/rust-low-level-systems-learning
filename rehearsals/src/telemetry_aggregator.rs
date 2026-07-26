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

pub struct TimeStat {
    pub epoch: usize,
    pub count: u64,
    pub sum: i64,
    pub min: i64,
    pub max: i64,
}

impl TimeStat {
    pub fn new() -> Self {
        TimeStat {
            epoch: 0,
            count: 0,
            sum: 0,
            min: i64::MAX,
            max: i64::MIN,
        }
    }
}

impl Default for TimeStat {
    fn default() -> Self {
        Self::new()
    }
}

// Trade off:
// we use i64 for calculate the epoch valid size
// with Time Windows to aggregate the stats, we cannot store too old's data.
// however the memory usage is extreamly lower than store every data's information when data input is going very
// large while time goes huge.

pub struct Aggregator {
    window_slots: Vec<TimeStat>,
    latest_epoch: Option<usize>,
    window_ms: u64,
    window_size: usize,
}

impl Aggregator {
    fn slots_idx(&self, epoch: usize) -> usize {
        epoch % self.window_size
    }
    /// `window_ms`:每個 window 的寬度;`num_windows`:保留最近幾個 window。
    /// 記憶體固定 O(num_windows),與樣本數無關。兩者皆 >= 1。
    pub fn new(window_ms: u64, num_windows: usize) -> Self {
        // todo!("rehearsal")
        let mut window_slots = Vec::new();
        window_slots.resize_with(num_windows, TimeStat::new);
        Self {
            window_slots,
            latest_epoch: None,
            window_ms,
            window_size: num_windows,
        }
    }

    /// 記錄一筆。window 邊界是半開區間 `[k*window_ms, (k+1)*window_ms)`。
    /// ts 落在「已被淘汰的過去」(比保留範圍還舊)→ 回 false 且不記。
    /// ts 跳到未來 → 成為新的最新 window,中間被跳過的 window 視同空。
    /// TC: O(1)
    /// SC: O(1)
    pub fn record(&mut self, ts_ms: u64, value: i64) -> bool {
        // SANITY TEST:
        let e = ts_ms as usize / self.window_ms as usize;
        if e > i64::MAX as usize {
            panic!("this ts_ms is too large")
        }
        if self.latest_epoch.is_none() {
            self.latest_epoch = Some(e);
        }
        let latest = self.latest_epoch.unwrap();
        if latest < e {
            self.latest_epoch = Some(e);
        }

        // 6 - 2 = 4 but we can only use 5 and 6
        // TODO: if the time will exceed u32, we should use u64 directly
        let min_epoch = latest as i64 - (self.window_size as i64);
        if e as i64 <= min_epoch {
            return false;
        }

        let idx = self.slots_idx(e);
        if self.window_slots[idx].epoch != e {
            self.window_slots[idx] = TimeStat::new();
            self.window_slots[idx].epoch = e;
        }

        let slot = &mut self.window_slots[idx];
        slot.min = slot.min.min(value);
        slot.max = slot.max.max(value);
        slot.count += 1;
        slot.sum += value;

        true
    }

    /// 回傳 ts 所屬 window 的目前統計。
    /// 該 window 沒有任何資料(空、被淘汰、尚未發生)→ None。
    /// Time: O(1)
    /// Space: O(1)
    pub fn stats(&self, ts_ms: u64) -> Option<WindowStats> {
        // it should use / instead of %
        if ts_ms / self.window_ms > i64::MAX as u64 {
            panic!("ts_ms too large");
        }
        let e = (ts_ms / self.window_ms) as i64;
        let latest = self.latest_epoch? as i64;
        if e <= latest - self.window_size as i64 {
            return None;
        }
        if e > latest {
            return None;
        }
        let idx = self.slots_idx(e as usize);
        let slot_ref = &self.window_slots[idx];
        if slot_ref.count == 0 || e != slot_ref.epoch as i64 {
            return None;
        }
        Some(WindowStats {
            count: slot_ref.count,
            sum: slot_ref.sum,
            min: slot_ref.min,
            max: slot_ref.max,
        })
    }
}

// happy path
#[test]
fn dryrun() {
    let mut aggr = Aggregator::new(100, 3);
    assert!(aggr.record(40, 4));
    assert!(aggr.record(301, 1));
    // exceed time
    assert!(!aggr.record(40, 7));
    assert!(aggr.record(302, 3));
    assert_eq!(
        aggr.stats(302),
        Some(WindowStats {
            count: 2,
            sum: 4,
            min: 1,
            max: 3,
        })
    );
}

// boundary test
#[test]
fn boundary_test() {
    let mut aggr = Aggregator::new(100, 2);
    assert!(aggr.record(0, 1));
    assert!(aggr.record(99, 20));
    assert!(aggr.stats(101).is_none());
    assert!(aggr.record(200, 50));
    assert!(aggr.stats(0).is_none());
    assert_eq!(
        aggr.stats(299),
        Some(WindowStats {
            count: 1,
            sum: 50,
            min: 50,
            max: 50
        })
    );
    aggr.record(700, 2);
    assert_eq!(aggr.stats(699), None);
}
