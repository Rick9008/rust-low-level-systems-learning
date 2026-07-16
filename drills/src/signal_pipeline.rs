//! drill:signal_pipeline —— 填 send(drop-newest + 喚醒)與掛牌握手。
//!
//! 已給:結構、start、consumer_loop 骨架、聚合。
//! 要填:`SignalSender::send` 與 `idle_park`。
//! 核心不變量:**consumer「掛牌 → 再查一次」、producer「push → 查牌」,
//! 兩邊中間各一道 `fence(SeqCst)`**——這是教科書 store-buffering litmus,
//! Release/Acquire 擋不住「雙方都讀到舊值 → 帶著貨睡死」。
//! 設計取捨見 `docs/signal_pipeline.md`。

use reference::spsc_ring::{Consumer, Producer, channel};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering, fence};
use std::thread::{self, JoinHandle, Thread};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signal {
    pub sensor_id: u16,
    pub value: i64,
}

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

pub struct SignalSender {
    tx: Producer<Signal>,
    /// 單寫者 → 普通 u64,不用 atomic。
    dropped: u64,
    parked: Arc<AtomicBool>,
    consumer: Thread,
}

impl SignalSender {
    /// spec:快路徑零 syscall 的 push。
    /// 1. `self.tx.push(s)`:Ok → `fence(SeqCst)` → 牌子掛著
    ///    (`parked.load`)才 `self.consumer.unpark()`,回 true。
    /// 2. Err(滿)→ **drop-newest**:`dropped += 1`,回 false。
    ///    (SPSC 上 producer 動不了 head,drop-oldest 做不到。)
    pub fn send(&mut self, s: Signal) -> bool {
        todo!("spec: push; Ok → fence(SeqCst) → 看牌 unpark; Err → dropped+=1")
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }
}

pub struct PipelineHandle {
    stop: Arc<AtomicBool>,
    consumer: Thread,
    join: JoinHandle<Stats>,
}

impl PipelineHandle {
    pub fn shutdown(self) -> Stats {
        self.stop.store(true, Ordering::Release);
        self.consumer.unpark();
        self.join.join().expect("consumer thread panicked")
    }
}

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
                if stop.load(Ordering::Acquire) {
                    return stats; // pop None ⇒ 殘料已 drain 完
                }
                if spins < SPIN_LIMIT {
                    spins += 1;
                    std::hint::spin_loop();
                    continue;
                }
                if let Some(s) = idle_park(&mut rx, parked, stop) {
                    stats.record(&s);
                }
                spins = 0;
            }
        }
    }
}

/// spec:掛牌睡覺的握手(lost-wakeup 防線)。
/// 1. `parked.store(true, SeqCst)`(掛牌)
/// 2. `fence(SeqCst)`(與 producer 的 fence 配對)
/// 3. **再 pop 一次**:撈到 → 摘牌、回 `Some(s)`
/// 4. 空且未 stop → `thread::park()`(unpark 先到不丟:token 語意)
/// 5. 醒來(或 stop)→ 摘牌、回 `None`(caller 回外圈重試)
fn idle_park(rx: &mut Consumer<Signal>, parked: &AtomicBool, stop: &AtomicBool) -> Option<Signal> {
    todo!("spec: 掛牌 SeqCst; fence; re-pop; 空且未 stop 才 park; 摘牌")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// 守恆:accepted + dropped == sent,聚合 count == accepted。
    #[test]
    #[ignore = "填完 send/idle_park 後移除"]
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
        drop(tx);
        let stats = handle.shutdown();
        assert_eq!(accepted + dropped, N);
        assert_eq!(stats.count, accepted);
    }

    /// park 後被喚醒——lost wakeup 會讓這個測試卡死(顯性失敗)。
    #[test]
    #[ignore = "填完 send/idle_park 後移除"]
    fn wakes_parked_consumer() {
        let (mut tx, handle) = start(8);
        assert!(tx.send(Signal {
            sensor_id: 1,
            value: 10,
        }));
        std::thread::sleep(Duration::from_millis(50)); // consumer 已掛牌睡著
        assert!(tx.send(Signal {
            sensor_id: 2,
            value: 32,
        }));
        std::thread::sleep(Duration::from_millis(50));
        drop(tx);
        let stats = handle.shutdown();
        assert_eq!(stats.count, 2);
        assert_eq!(stats.sum, 42);
    }
}
