//! drill:conflation_slot —— 填 publish / recv / close(值層/通知層分離)。
//!
//! 已給:結構(值層 slots + 通知層 ready + queued 旗標)、new/stale_count。
//! 要填:`publish` / `recv` / `close`——動手前先把 invariant 唸一遍:
//!
//! ```text
//! slot.queued == true ⟺ key ∈ ready
//! ```
//!
//! 填之前紙上回答(圖解與 stepper:`html_p/conflation-slot-stepper.html`):
//! 1. `queued` 這個欄位的存在理由是什麼?拿掉它,ready 的長度上界變成什麼?
//! 2. recv 的「pop + 讀值 + 清旗標」為什麼必須同一個 critical section?
//!    拆成三段上鎖時,producer 插在哪個縫會造成什麼?(lost update 劇本講一遍)
//! 3. notify 為什麼在持鎖時發?放鎖再 notify 的縫隙裡 consumer 會發生什麼?
//! 4. recv 醒來後為什麼要 loop 重查而不是 if?closed 的檢查為什麼放在
//!    「pop 不到東西之後」而不是迴圈開頭?(先 drain 後關店)
//! 5. seq 相等的重送要不要套用?為什麼對「絕對狀態快照」是安全的?
//!
//! MutexGuard 陷阱(已在註解裡給你):`let inner = &mut *g;` 先解一層,
//! 借用檢查器才能對 slots/ready/stale 做欄位切分借用。

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Condvar, Mutex};

struct Slot<V> {
    val: V,       // 最新快照值(覆蓋即合法)
    seq: u64,     // 該 key 已接受的最大 seq(亂序保護)
    count: u32,   // 自上次被取走後,被摺疊了幾筆(丟棄留帳)
    queued: bool, // 是否已在 ready 裡(去重;invariant 的本體)
}

struct Inner<K, V> {
    slots: HashMap<K, Slot<V>>,
    ready: VecDeque<K>, // dirty set:長度 ≤ K,同 key 絕不重複
    closed: bool,
    stale: u64,
}

pub struct Conflator<K, V> {
    inner: Mutex<Inner<K, V>>,
    cv: Condvar,
}

impl<K: Copy + Eq + Hash, V: Clone> Default for Conflator<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Copy + Eq + Hash, V: Clone> Conflator<K, V> {
    pub fn new() -> Self {
        Conflator {
            inner: Mutex::new(Inner {
                slots: HashMap::new(),
                ready: VecDeque::new(),
                closed: false,
                stale: 0,
            }),
            cv: Condvar::new(),
        }
    }

    /// spec:producer。O(1) amortized,永不長時間阻塞。
    /// 1. 上鎖;`let inner = &mut *g;`(欄位切分)
    /// 2. closed → 靜默返回(shutdown 語意)
    /// 3. `entry(key).or_insert(…)` 開格或拿格
    /// 4. `seq < slot.seq` → stale += 1,返回(遲到的舊訊息不許蓋新值;相等照樣套用)
    /// 5. 蓋值、更新 seq、count 飽和加一
    /// 6. `!slot.queued` 才:置 T、push_back(key)、**持鎖 notify_one**
    pub fn publish(&self, key: K, seq: u64, val: V) {
        let _ = (key, seq, val);
        todo!("spec: 蓋值 + queued 去重 + 持鎖 notify;想清楚 slot 借用何時結束")
    }

    /// spec:consumer。回 (key, 最新值, 這輪摺疊數);close 且 drain 完 → None。
    /// loop {
    ///   pop_front 到 key → **同一個 critical section**:清 queued、
    ///     `mem::replace(&mut slot.count, 0)`、`slot.val.clone()` → return Some
    ///   pop 不到 → closed? → return None
    ///   都不是 → `g = self.cv.wait(g).unwrap()`(醒來 ≠ 有貨,回圈重查)
    /// }
    pub fn recv(&self) -> Option<(K, V, u32)> {
        todo!("spec: 三件事一次鎖內做完;closed 檢查在 pop 失敗之後(先 drain 後關店)")
    }

    /// spec:shutdown——closed=true + **持鎖 notify_all**(叫醒所有等待者走 None 路徑)。
    pub fn close(&self) {
        todo!("spec: 兩行;想清楚為什麼是 notify_all 不是 notify_one")
    }

    /// 已給:被亂序丟棄的筆數(≠ 摺疊數——那筆帳在 recv 的 count 裡)。
    pub fn stale_count(&self) -> u64 {
        self.inner.lock().unwrap().stale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// 手 trace:三筆 publish 摺成一筆 (7,300,3);再 publish 重新排隊 → (7,400,1)。
    #[test]
    #[ignore = "drill:填完 publish/recv/close 後移除"]
    fn fold_semantics() {
        let c = Conflator::new();
        c.publish(7_u32, 1, 100_u64);
        c.publish(7, 2, 200);
        c.publish(7, 3, 300);
        assert_eq!(c.recv(), Some((7, 300, 3)));
        c.publish(7, 4, 400);
        assert_eq!(c.recv(), Some((7, 400, 1)));
    }

    /// 吵鬧 key 摺成一格,擠不掉安靜 key;值層大小 = key 基數。
    #[test]
    #[ignore = "drill:填完後移除"]
    fn noisy_key_cannot_evict_quiet_key() {
        let c = Conflator::new();
        c.publish(1_u32, 1, 11_u64);
        for s in 1..=1000 {
            c.publish(2, s, s);
        }
        assert_eq!(c.recv(), Some((1, 11, 1)));
        assert_eq!(c.recv(), Some((2, 1000, 1000)));
        assert_eq!(c.inner.lock().unwrap().slots.len(), 2);
    }

    /// 亂序保護:遲到舊 seq 不覆蓋,入 stale 帳。
    #[test]
    #[ignore = "drill:填完後移除"]
    fn stale_seq_rejected() {
        let c = Conflator::new();
        c.publish(5_u32, 10, 999_u64);
        c.publish(5, 3, 111);
        assert_eq!(c.recv(), Some((5, 999, 1)));
        assert_eq!(c.stale_count(), 1);
    }

    /// close 語意:drain 完剩貨 → None 不掛死;關店後 publish 忽略。
    #[test]
    #[ignore = "drill:填完後移除"]
    fn close_drains_then_none() {
        let c = Conflator::new();
        c.publish(1_u32, 1, 10_u64);
        c.publish(2, 1, 20);
        c.close();
        c.publish(3, 1, 30);
        let mut got = vec![c.recv(), c.recv()];
        got.sort();
        assert_eq!(got, vec![Some((1, 10, 1)), Some((2, 20, 1))]);
        assert_eq!(c.recv(), None);
    }

    /// 最終狀態保證:每 key 最後收到的值 = 它最後一次 publish;單調不回退。
    #[test]
    #[ignore = "drill:填完後移除"]
    fn two_threads_final_state_delivered() {
        let c = Arc::new(Conflator::new());
        let p = {
            let c = Arc::clone(&c);
            thread::spawn(move || {
                for s in 1..=5000_u64 {
                    c.publish((s % 4) as u32, s, s);
                }
                c.close();
            })
        };
        let cc = Arc::clone(&c);
        let consumer = thread::spawn(move || {
            let mut last: HashMap<u32, u64> = HashMap::new();
            while let Some((k, v, _n)) = cc.recv() {
                if let Some(&prev) = last.get(&k) {
                    assert!(v > prev, "key {k} 回退 {prev} -> {v}");
                }
                last.insert(k, v);
            }
            last
        });
        p.join().unwrap();
        let last = consumer.join().unwrap();
        for k in 0..4_u32 {
            let expect = (1..=5000_u64)
                .filter(|s| (s % 4) as u32 == k)
                .last()
                .unwrap();
            assert_eq!(last[&k], expect, "key {k} 最終狀態未送達");
        }
    }
}
