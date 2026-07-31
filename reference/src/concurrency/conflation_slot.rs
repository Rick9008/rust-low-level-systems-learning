//! # conflation_slot —— per-key conflation(值層/通知層分離)
//!
//! ## [Clarify]
//! 解決:producer 產得比 consumer 吃得快,而 consumer **只在乎每個 key 的最新值**
//! (市場報價、感測器最新讀數、dirty-rect)。同 key 舊值是死重量——覆蓋即合法。
//! Constraints:std-only、publish 不阻塞(除短鎖)、記憶體 **O(K)**(K = key 基數,
//! 與 update 速率完全脫鉤)、跨 key FIFO 公平、丟棄要留帳(count/stale)。
//! 不適用:payload 是 delta(要改可結合 merge)、需要完整事件序列(audit → durable log)。
//!
//! ## [Abstract]
//! `K: Copy + Eq + Hash`、`V: Clone`;seq 帶亂序保護。摺疊函數固定為「覆蓋」——
//! merge 版(sum/max)是後續迭代,那就走回 aggregate window 的方向。
//!
//! ## [Iterate]
//! tier 0 無界佇列 → OOM;tier 1 有界 ring drop-oldest → 丟棄粒度是「事件」,
//! 吵鬧 key 擠掉安靜 key(公平性錯);tier 2 per-key map → consumer 不知誰髒;
//! tier 3 = 本模組:map(值層)+ ready 佇列(通知層)+ `queued` 旗標去重。
//! 完整推導與三個互動 stepper:`html_p/conflation-slot-stepper.html`。
//!
//! ## [Trade-offs]
//! - **唯一 invariant:`slot.queued == true ⟺ key ∈ ready`**。三種破法 = 三種 bug:
//!   recv 三段上鎖(lost update)、沒有 queued(ready 無界)、先宣告髒再寫值(讀舊值)。
//! - notify 在**持鎖時**發:放鎖再 notify 的縫隙裡 consumer 可能檢查完就睡,錯過喚醒。
//! - producer 永不阻塞的代價:感受不到 consumer 慢——backpressure 訊號要另外走
//!   (count 就是現成的量表)。
//!
//! ## [Dry-Run]
//! 見下方測試:`fold_semantics` 逐行手 trace;boundary 涵蓋吵鬧/安靜 key 隔離、
//! 亂序拒收、close 後 drain、雙執行緒最終狀態保證。
//!
//! 複雜度:publish O(1) amortized、recv O(1);空間 O(K) 值層 + O(K) 通知層。

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
    ready: VecDeque<K>, // dirty set:只放髒 key,長度 ≤ K,同 key 絕不重複
    closed: bool,
    stale: u64, // 被亂序丟棄的筆數(可觀測性)
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

    /// producer:蓋值 + 必要時排通知。O(1) amortized;永不長時間阻塞。
    /// close 後的 publish 靜默忽略(shutdown 語意;要留帳可再加 counter)。
    pub fn publish(&self, key: K, seq: u64, val: V) {
        let mut g = self.inner.lock().unwrap();
        // MutexGuard 先解一層拿 &mut Inner,借用檢查器才能對不同欄位做切分借用
        let inner = &mut *g;
        if inner.closed {
            return;
        }
        let slot = inner.slots.entry(key).or_insert(Slot {
            val: val.clone(),
            seq: 0,
            count: 0,
            queued: false,
        });
        if seq < slot.seq {
            inner.stale += 1; // 亂序:遲到的舊訊息不許蓋掉新值
            return;
        }
        // seq 相等照樣套用:對「絕對狀態快照」重送是 idempotent
        slot.val = val;
        slot.seq = seq;
        slot.count = slot.count.saturating_add(1);
        if !slot.queued {
            slot.queued = true;
            inner.ready.push_back(key);
            // notify 在持鎖狀態下發:若先放鎖再 notify,consumer 可能在縫隙裡
            // 「檢查 ready 為空 → 睡下」,這聲通知就打在半空中
            self.cv.notify_one();
        }
    }

    /// consumer:回傳 (key, 最新值, 這輪摺疊掉幾筆);close 且 drain 完後回 None。
    ///
    /// 關鍵:pop_front + 清 queued + 讀值,**同一個 critical section**。
    /// 拆成多段上鎖 = lost update:producer 在縫隙裡看到 queued=true 而跳過通知,
    /// 之後 consumer 清旗標——新值躺在 slot 裡,永遠沒人來拿。
    pub fn recv(&self) -> Option<(K, V, u32)> {
        let mut g = self.inner.lock().unwrap();
        loop {
            let inner = &mut *g;
            if let Some(key) = inner.ready.pop_front() {
                let slot = inner
                    .slots
                    .get_mut(&key)
                    .expect("invariant:ready 中的 key 必有 slot");
                slot.queued = false;
                let count = std::mem::replace(&mut slot.count, 0);
                return Some((key, slot.val.clone(), count));
            }
            if inner.closed {
                return None; // 醒來沒貨且已關店:drain 完畢
            }
            // 醒來 ≠ 有貨(spurious / 多 consumer 搶先),所以是 loop 重查不是 if
            g = self.cv.wait(g).unwrap();
        }
    }

    /// shutdown:之後的 publish 被忽略;喚醒所有等待者去走「drain 完回 None」路徑。
    pub fn close(&self) {
        let mut g = self.inner.lock().unwrap();
        g.closed = true;
        self.cv.notify_all(); // 同樣持鎖 notify
    }

    /// 被亂序丟棄的筆數(≠ 被摺疊的筆數——摺疊帳在每次 recv 的 count 裡)。
    pub fn stale_count(&self) -> u64 {
        self.inner.lock().unwrap().stale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// 手 trace(逐行):
    /// 1. publish(7,1,100):新 slot{val=100,seq=1,count=1,queued=T},ready=[7],notify
    /// 2. publish(7,2,200):蓋成 200,count=2;queued 已 T → 不重排、不 notify
    /// 3. publish(7,3,300):蓋成 300,count=3;同上
    /// 4. recv():pop 7 → queued=F、count 取走歸零 → (7, 300, 3)
    ///    —— 100/200 被結構性丟棄,而帳(count=3)說得出丟了幾筆
    /// 5. publish(7,4,400):queued=F → 重新排隊+notify;recv → (7,400,1)
    #[test]
    fn fold_semantics() {
        let c = Conflator::new();
        c.publish(7_u32, 1, 100_u64);
        c.publish(7, 2, 200);
        c.publish(7, 3, 300);
        assert_eq!(c.recv(), Some((7, 300, 3)), "只送最新值,count 回報摺疊數");
        c.publish(7, 4, 400);
        assert_eq!(c.recv(), Some((7, 400, 1)));
    }

    /// boundary:per-key 隔離——吵鬧 key 摺成一格,擠不掉安靜 key;
    /// 兩層記憶體都被 key 基數(=2)鎖住,與 1001 筆 update 無關。
    #[test]
    fn noisy_key_cannot_evict_quiet_key() {
        let c = Conflator::new();
        c.publish(1_u32, 1, 11_u64); // 安靜 key
        for s in 1..=1000 {
            c.publish(2, s, s); // 吵鬧 key
        }
        assert_eq!(c.recv(), Some((1, 11, 1)), "安靜 key 存活(FIFO 先進先出)");
        assert_eq!(c.recv(), Some((2, 1000, 1000)), "吵鬧 key 摺成 1 筆");
        assert_eq!(c.inner.lock().unwrap().slots.len(), 2);
    }

    /// boundary:亂序保護——遲到的舊 seq 不許覆蓋,入 stale 帳。
    #[test]
    fn stale_seq_rejected() {
        let c = Conflator::new();
        c.publish(5_u32, 10, 999_u64);
        c.publish(5, 3, 111); // 遲到
        assert_eq!(c.recv(), Some((5, 999, 1)));
        assert_eq!(c.stale_count(), 1);
    }

    /// boundary:close 語意——關店後 drain 得完剩貨,之後回 None;再 publish 被忽略。
    #[test]
    fn close_drains_then_none() {
        let c = Conflator::new();
        c.publish(1_u32, 1, 10_u64);
        c.publish(2, 1, 20);
        c.close();
        c.publish(3, 1, 30); // 關店後:忽略
        let mut got = vec![c.recv(), c.recv()];
        got.sort();
        assert_eq!(got, vec![Some((1, 10, 1)), Some((2, 20, 1))]);
        assert_eq!(c.recv(), None, "drain 完回 None,不掛死");
    }

    /// 最終狀態保證:亂流之後,每個 key 收到的最後一筆 = 它最後一次 publish;
    /// 同 key 值單調遞增(摺疊不得回退);總摺疊帳 ≤ 總 publish 數。
    #[test]
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
            let mut folded: u64 = 0;
            while let Some((k, v, n)) = cc.recv() {
                if let Some(&prev) = last.get(&k) {
                    assert!(v > prev, "key {k} 回退 {prev} -> {v}");
                }
                last.insert(k, v);
                folded += u64::from(n);
            }
            (last, folded)
        });
        p.join().unwrap();
        let (last, folded) = consumer.join().unwrap();
        for k in 0..4_u32 {
            let expect = (1..=5000_u64)
                .filter(|s| (s % 4) as u32 == k)
                .last()
                .unwrap();
            assert_eq!(last[&k], expect, "key {k} 最終狀態未送達");
        }
        assert!(folded <= 5000);
    }
}
