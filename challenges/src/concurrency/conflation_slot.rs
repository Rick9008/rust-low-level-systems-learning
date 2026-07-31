//! ★ challenge:per-key conflation slot(值層/通知層分離)
//!
//! 【題目】producer 高速發布 (key, seq, val),consumer 低速消費,且**只在乎
//! 每個 key 的最新值**。實作一個發布/訂閱結構:同 key 新值就地覆蓋舊值,
//! consumer 阻塞等待「有 key 變髒」,一次拿走一個 key 的最新值。
//!
//! 【constraints】
//! - std-only(Mutex/Condvar 可用);publish O(1) amortized、永不長時間阻塞
//! - 記憶體 **O(K)**(K = key 基數)——與 update 速率完全脫鉤,這是本題的全部價值
//! - 跨 key FIFO 公平:吵鬧 key 不得把安靜 key 擠出通知
//! - 丟棄要留帳:consumer 要能知道「這輪被摺疊掉幾筆」;亂序(舊 seq)要拒收且可觀測
//! - close():之後 publish 忽略;consumer drain 完剩貨後收到 None,不掛死
//!
//! 【clarify points——動手前先自答】
//! - payload 是絕對快照還是 delta?覆蓋什麼時候不合法?(delta → 要改成可結合 merge)
//! - consumer 怎麼知道「哪些 key 髒了」而不掃全表?通知層的長度上界靠什麼鎖住?
//! - 「取值」和「清髒旗標」拆成兩次上鎖會發生什麼?(想一個 producer 插隊的劇本)
//! - notify 在持鎖時發還是放鎖後發?差在哪?
//! - 需要完整事件序列(audit/replay)的需求來了怎麼辦?(答案不是改這個結構)
//!
//! 【要實作】下方簽名。struct 內部完全自己設計(佔位欄位整個換掉)。
//! 【驗收】tests/conflation_slot.rs 轉綠(含吵鬧/安靜 key 隔離與雙執行緒
//! 最終狀態保證),然後 diff reference 的 `concurrency/conflation_slot`,
//! 再開 `html_p/conflation-slot-stepper.html` 把三個 stepper 走完對答案。

use std::hash::Hash;
use std::marker::PhantomData;

pub struct Conflator<K, V> {
    // ↓ 佔位:讓空殼能編譯。動手時整個換成你的設計。
    _todo: PhantomData<(K, V)>,
}

impl<K: Copy + Eq + Hash, V: Clone> Conflator<K, V> {
    pub fn new() -> Self {
        todo!("challenge: 從空白開始")
    }

    /// 發布:同 key 覆蓋;亂序拒收;必要時通知 consumer。
    pub fn publish(&self, key: K, seq: u64, val: V) {
        let _ = (key, seq, val);
        todo!("challenge")
    }

    /// 阻塞取:回 (key, 最新值, 這輪摺疊掉幾筆);close 且 drain 完 → None。
    pub fn recv(&self) -> Option<(K, V, u32)> {
        todo!("challenge")
    }

    /// shutdown:喚醒所有等待者;之後 publish 忽略。
    pub fn close(&self) {
        todo!("challenge")
    }

    /// 被亂序丟棄的筆數。
    pub fn stale_count(&self) -> u64 {
        todo!("challenge")
    }
}

impl<K: Copy + Eq + Hash, V: Clone> Default for Conflator<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
