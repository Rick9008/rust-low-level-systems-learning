//! # signal_pipeline —— JD 本尊圖:訊號源 → SPSC ring → 消費執行緒
//!
//! ## [Clarify]
//! 解決:硬體訊號高速湧入(爆發時 >1M/s),一條 IO/訊號執行緒收、
//! 一條處理執行緒聚合——兩者之間用什麼接、滿了怎麼辦、沒事時誰睡誰醒。
//! 這是 telemetry JD 的核心圖,也是 HFT 的標準管線形狀
//! (pinned thread 之間全用 SPSC,熱路徑零鎖、零 syscall、零配置)。
//! Constraints:std-only、恰好一產一消(所以才輪得到 SPSC)、
//! 訊號可丟(telemetry 語意)但**丟多少要可觀測**。
//!
//! ## [Abstract]
//! 三個正交的決策,各自獨立選:
//! 1. **佇列**:[`crate::spsc_ring`](已有,無鎖 O(1) 無 syscall);
//! 2. **full policy**:drop-newest + `dropped` 計數——注意 **SPSC ring 上
//!    做不到 drop-oldest**(`head` 是 consumer 單寫的,producer 不能動它);
//!    要「新蓋舊」得換結構(per-key conflation slot,market data 的答案);
//! 3. **消費端等待策略**:spin(HFT,燒核換 ns 級喚醒)→ spin-then-park
//!    (本實作:先燒一小段,沒貨再睡)→ 純 park(省電,喚醒 ~µs)。
//!
//! ## [Iterate]
//! naive:consumer 忙輪詢(100% CPU)→ 加 park:誰來叫醒?producer 每筆
//! unpark = 每筆一次 syscall,SPSC 的零 syscall 白省了 → **掛牌握手**:
//! consumer 掛牌(`parked=true`)才睡,producer 只在看到牌子時 unpark
//! ——快路徑(consumer 醒著)零 syscall。
//!
//! ## [Trade-offs]
//! - **掛牌握手需要 SeqCst fence(repo 第一個真的需要 SeqCst 的地方)**:
//!   consumer「掛牌 → 再查一次佇列」、producer「push → 查牌子」是教科書
//!   store-buffering(SB)litmus:兩邊都是「先寫後讀」,Release/Acquire
//!   允許雙方都讀到舊值 → consumer 帶著貨睡死。兩邊各插一道
//!   `fence(SeqCst)` 禁掉這個交錯。park 的 token 語意再兜底另一半
//!   (unpark 先於 park 不丟)。
//! - spin 長度:太短 → 爆發間隙頻繁睡醒(µs 級 syscall);太長 → 燒 CPU。
//!   本實作 100 次 spin_loop(~百 ns 級)是示範值;HFT 直接 spin 到底。
//! - dropped 計數是 producer 單寫的普通 `u64` 欄位——單寫者連 atomic
//!   都不用,這本身就是 SPSC 思維的延伸。
//! - **誠實邊界**:掛牌握手的交錯靠手 trace + stress 測試把關;
//!   loom 版要把 fence 與 park 都收進 shim,留作延伸。
//!
//! ## [Dry-Run]
//! 測試:守恆(accepted + dropped == sent 且 count == accepted)、
//! park 後被喚醒(lost wakeup 會讓測試卡死,顯性失敗)、
//! shutdown drain(殘料處理完才退)。
//!
//! 對應 docs/signal_pipeline.md:等待策略階梯、SB litmus 手 trace、
//! drop-newest vs conflation、HFT 對照。

use crate::spsc_ring::{Consumer, Producer, channel};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering, fence};
use std::thread::{self, JoinHandle, Thread};

/// 一筆訊號(16B:對映 cost-model 的容量算式)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signal {
    pub sensor_id: u16,
    pub value: i64,
}

/// 聚合結果:O(1) 記憶體,與樣本數脫鉤(telemetry 的 space 答案)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub count: u64,
    pub sum: i64,
    pub min: i64,
    pub max: i64,
}

impl Stats {
    fn new() -> Self {
        Self {
            count: 0,
            sum: 0,
            min: i64::MAX,
            max: i64::MIN,
        }
    }

    fn record(&mut self, s: &Signal) {
        self.count += 1;
        self.sum += s.value;
        self.min = self.min.min(s.value);
        self.max = self.max.max(s.value);
    }
}

/// producer 端:push 訊號,滿了 drop-newest 並計數。
pub struct SignalSender {
    tx: Producer<Signal>,
    /// 單寫者(只有 producer 執行緒動它)→ 普通 u64 就夠,不用 atomic。
    dropped: u64,
    parked: Arc<AtomicBool>,
    consumer: Thread,
}

impl SignalSender {
    /// 快路徑(consumer 醒著):一次 SPSC push,零 syscall。
    /// 滿了:**drop-newest**(丟這一筆、計數),回 `false`。
    pub fn send(&mut self, s: Signal) -> bool {
        match self.tx.push(s) {
            Ok(()) => {
                // SB litmus 的 producer 半邊:push(寫)之後讀牌子,
                // 中間的 SeqCst fence 與 consumer 側的 fence 配對,
                // 禁止「雙方都讀到舊值」的交錯。
                fence(Ordering::SeqCst);
                if self.parked.load(Ordering::Relaxed) {
                    self.consumer.unpark(); // 慢路徑才付 syscall
                }
                true
            }
            Err(_) => {
                self.dropped += 1; // 可觀測的洩壓閥
                false
            }
        }
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }
}

/// consumer 端的控制把手:shutdown 會 drain 完殘料才回。
pub struct PipelineHandle {
    stop: Arc<AtomicBool>,
    consumer: Thread,
    join: JoinHandle<Stats>,
}

impl PipelineHandle {
    /// 置 stop → unpark(consumer 可能睡著)→ join。
    /// 回傳的 Stats 保證含 shutdown 前所有已 push 成功的訊號(drain 語意)。
    pub fn shutdown(self) -> Stats {
        self.stop.store(true, Ordering::Release);
        self.consumer.unpark();
        self.join.join().expect("consumer thread panicked")
    }
}

/// 起一條 consumer 執行緒,回(producer 把手, 控制把手)。
/// capacity 上取 2 的冪(spsc_ring 的規則)。
pub fn start(capacity: usize) -> (SignalSender, PipelineHandle) {
    let (tx, rx) = channel(capacity);
    let parked = Arc::new(AtomicBool::new(false));
    let stop = Arc::new(AtomicBool::new(false));
    let (parked2, stop2) = (Arc::clone(&parked), Arc::clone(&stop));
    let join = thread::Builder::new()
        .name("signal-consumer".into())
        .spawn(move || consumer_loop(rx, &parked2, &stop2))
        .expect("spawn consumer");
    let consumer = join.thread().clone();
    (
        SignalSender {
            tx,
            dropped: 0,
            parked,
            consumer: consumer.clone(),
        },
        PipelineHandle {
            stop,
            consumer,
            join,
        },
    )
}

/// spin-then-park 的 spin 額度(示範值;HFT 直接 spin 到底)。
const SPIN_LIMIT: u32 = 100;

fn consumer_loop(mut rx: Consumer<Signal>, parked: &AtomicBool, stop: &AtomicBool) -> Stats {
    let mut stats = Stats::new();
    let mut spins: u32 = 0;
    loop {
        match rx.pop() {
            Some(s) => {
                stats.record(&s);
                spins = 0;
            }
            None => {
                // 空:先確認是不是該收工(pop 已回 None ⇒ 殘料 drain 完畢)。
                if stop.load(Ordering::Acquire) {
                    return stats;
                }
                if spins < SPIN_LIMIT {
                    spins += 1;
                    std::hint::spin_loop();
                    continue;
                }
                if let Some(s) = idle_park(&mut rx, parked, stop) {
                    stats.record(&s); // 掛牌後 re-check 撈到的那筆
                }
                spins = 0;
            }
        }
    }
}

/// 掛牌睡覺的握手(lost-wakeup 的防線全在這幾行):
/// 1. 掛牌 `parked = true`
/// 2. `fence(SeqCst)`——與 producer 的 fence 配對,殺掉 SB 交錯
/// 3. **再查一次佇列**:producer 若在掛牌前 push、查牌前錯過牌子,
///    這一查會撈到貨(或至少不睡)
/// 4. 空 → park(unpark 先到也不丟:park 的 token 語意)
/// 5. 醒來摘牌
fn idle_park(rx: &mut Consumer<Signal>, parked: &AtomicBool, stop: &AtomicBool) -> Option<Signal> {
    parked.store(true, Ordering::SeqCst);
    fence(Ordering::SeqCst);
    if let Some(s) = rx.pop() {
        parked.store(false, Ordering::Release);
        return Some(s);
    }
    if !stop.load(Ordering::Acquire) {
        thread::park();
    }
    parked.store(false, Ordering::Release);
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// 守恆:狂灌 100k 筆(capacity 8,幾乎必有 drop)——
    /// accepted + dropped == sent,且聚合 count == accepted、
    /// sum == accepted(每筆 value=1)。掉的每一筆都被數到,沒有黑洞。
    #[test]
    fn conservation_under_burst() {
        let (mut tx, handle) = start(8);
        const N: u64 = 100_000;
        let mut accepted = 0u64;
        for _ in 0..N {
            if tx.send(Signal {
                sensor_id: 1,
                value: 1,
            }) {
                accepted += 1;
            }
        }
        let dropped = tx.dropped();
        drop(tx); // producer 收工,shutdown 前不再有人 push
        let stats = handle.shutdown();
        assert_eq!(accepted + dropped, N, "每一筆要嘛進要嘛被數到");
        assert_eq!(stats.count, accepted, "drain 語意:進去的全被聚合");
        assert_eq!(stats.sum, accepted as i64);
    }

    /// park 路徑:先送一筆、等 consumer 睡著、再送一筆——第二筆必須把它
    /// 叫醒。若掛牌握手有 lost wakeup,shutdown 的 join 會卡死
    /// (測試以 hang 的形式顯性失敗,而不是靜默漏資料)。
    #[test]
    fn wakes_parked_consumer() {
        let (mut tx, handle) = start(8);
        assert!(tx.send(Signal {
            sensor_id: 1,
            value: 10,
        }));
        // spin 額度 ~百 ns,50ms 後 consumer 必然已掛牌睡著
        std::thread::sleep(Duration::from_millis(50));
        assert!(tx.send(Signal {
            sensor_id: 2,
            value: 32,
        }));
        std::thread::sleep(Duration::from_millis(50)); // 讓喚醒與消費發生
        drop(tx);
        let stats = handle.shutdown();
        assert_eq!(stats.count, 2);
        assert_eq!(stats.sum, 42);
        assert_eq!((stats.min, stats.max), (10, 32));
    }

    /// shutdown drain:push 完立刻 shutdown——殘料要處理完才退,
    /// 一筆不少(對照 thread_pool 的 drain-then-exit 語意)。
    #[test]
    fn shutdown_drains_backlog() {
        let (mut tx, handle) = start(1024);
        let mut accepted = 0u64;
        for i in 0..500 {
            if tx.send(Signal {
                sensor_id: 0,
                value: i,
            }) {
                accepted += 1;
            }
        }
        drop(tx);
        let stats = handle.shutdown();
        assert_eq!(stats.count, accepted);
    }
}
