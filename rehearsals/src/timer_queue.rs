//! rehearsal h:timer_queue —— 題目見 rehearsals/README.md。
//!
//! 時間用邏輯毫秒(u64)——測試可決定性地控制;真實系統把 now 換成 Instant。
//! 只給 API 簽名;你自己的測試寫在本檔底部 `#[cfg(test)] mod tests`。

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

pub struct TimerQueue {
    // ↓ 佔位:動手時整個換成你的設計。
    // deadline, id, interval
    queue: BinaryHeap<Reverse<(u64, u64, u64)>>,
    del_ids: HashSet<u64>,
}

impl TimerQueue {
    pub fn new() -> Self {
        // todo!("rehearsal")
        Self {
            queue: BinaryHeap::new(),
            del_ids: HashSet::new(),
        }
    }

    /// 排一個週期任務:第一次在 `first_at_ms`,之後每 `interval_ms` 一次。
    /// `interval_ms >= 1`;id 的唯一性由 caller 負責。
    pub fn schedule(&mut self, id: u64, first_at_ms: u64, interval_ms: u64) {
        // todo!("rehearsal")
        assert!(
            interval_ms >= 1,
            "If interval_ms is 0, it will be unlimited event happen in the same time"
        );
        self.queue.push(Reverse((first_at_ms, id, interval_ms)));
    }

    /// 下一個 deadline;沒有任何 timer → None。
    /// caller 拿這個值去 park 到那個時間點——不是輪詢。
    pub fn next_deadline(&self) -> Option<u64> {
        // todo!("rehearsal")
        self.queue.peek().map(|tuple| tuple.0.0)
    }

    /// 收割所有 deadline <= now 的觸發,依 (deadline, id) 排序回傳 id。
    /// 每次觸發後以「舊 deadline + interval」重排(不從 now 起算 → 不飄移);
    /// now 落後很多時,同一 timer 在這一次呼叫裡會補發多次。
    pub fn pop_due(&mut self, now_ms: u64) -> Vec<u64> {
        // todo!("rehearsal")
        let mut res = Vec::new();
        while let Some(deadline) = self.queue.peek().map(|tuple| tuple.0.0)
            && deadline <= now_ms
        {
            let Reverse((time, id, interval)) = self.queue.pop().unwrap();
            if self.del_ids.contains(&id) {
                self.del_ids.remove(&id);
                continue;
            }
            res.push(id);
            self.queue.push(Reverse((time + interval, id, interval)));
        }
        res
    }

    /// 目前排程中的 timer 數。
    pub fn len(&self) -> usize {
        // todo!("rehearsal")
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        // todo!("rehearsal")
        self.queue.is_empty()
    }

    pub fn del_id(&mut self, id: u64) -> bool {
        self.del_ids.insert(id)
    }
}

impl Default for TimerQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn ascend_id_check() {
    let mut timer_qe = TimerQueue::new();
    timer_qe.schedule(2, 10, 30);
    timer_qe.schedule(1, 10, 560);
    assert_eq!(timer_qe.pop_due(10), &[1, 2]);
}
