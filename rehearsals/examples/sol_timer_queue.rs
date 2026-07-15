//! solution:題 h timer_queue——**寫完彩排才開**。
//! canonical 設計:`BinaryHeap<Reverse<(deadline, id, interval)>>`——tuple 排序
//! 天然給出 (deadline, id) 的 tie-break;`peek` 就是 park 目標;
//! 重排從**舊 deadline** 起算(now 起算會飄移);catch-up 靠「重排後可能仍
//! <= now → 迴圈繼續收」自然發生。timer 極多時換 hashed timer wheel
//! (O(1) 攤銷,用精度換吞吐)。
//! 驗證:rehearsals/tests/timer_queue_test.rs 全綠。

use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub struct TimerQueue {
    heap: BinaryHeap<Reverse<(u64, u64, u64)>>, // (deadline, id, interval)
}

impl TimerQueue {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    pub fn schedule(&mut self, id: u64, first_at_ms: u64, interval_ms: u64) {
        assert!(interval_ms >= 1); // interval 0 會讓 pop_due 停不下來
        self.heap.push(Reverse((first_at_ms, id, interval_ms)));
    }

    pub fn next_deadline(&self) -> Option<u64> {
        self.heap.peek().map(|Reverse((d, _, _))| *d)
    }

    pub fn pop_due(&mut self, now_ms: u64) -> Vec<u64> {
        let mut out = Vec::new();
        while let Some(&Reverse((d, id, interval))) = self.heap.peek() {
            if d > now_ms {
                break;
            }
            self.heap.pop();
            out.push(id);
            // 舊 deadline + interval:不飄移;若仍 <= now,迴圈會再收(補發)
            self.heap.push(Reverse((d + interval, id, interval)));
        }
        out
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}

impl Default for TimerQueue {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let mut q = TimerQueue::new();
    q.schedule(1, 100, 100);
    q.schedule(2, 150, 300);
    assert_eq!(q.next_deadline(), Some(100)); // park 到 100,不是輪詢
    assert_eq!(q.pop_due(300), vec![1, 2, 1, 1]); // 100,150,200,300
    assert_eq!(q.next_deadline(), Some(400)); // 從舊 deadline 起算,不飄移
    println!("sol_timer_queue: ok");
}
