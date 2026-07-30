// ⚠⚠ 防雷:本檔是 sim k(per-core telemetry fan-in)的解法。計時場排在 8/1——
// 在那之前不要讀本檔(含註解與測試)。自定規則:跑題前不開 oracle/sol。

//! # percpu_fanin —— per-core telemetry fan-in(sim k 教學版)
//!
//! ## [Clarify]
//! 題幹:`docs/interviews/sim-problems.md` sim k(彩排 harness:`rehearsals/src/sim_k_fanin.rs`)。
//! N 個核各自產 telemetry,一條 aggregator 收攏。隱藏 spec:
//! - producer **絕不 block**(它跑在人家的熱路徑上)——滿了是你的 policy,但要可觀測;
//! - 熱核不能餓死冷核(Phase 2 的 `budget`);
//! - per-core 順序要保住;跨核全域順序**沒有**(說出口,clarify 的料);
//! - shutdown:停之前每條 channel 都要 drain 完。
//!
//! ## [Abstract]
//! 為什麼是每核一條 SPSC 而不是共用一條 MPSC?——tail 上的 CAS 競爭把 SPSC 的
//! 單寫者優勢全丟了(同 [`crate::concurrency::signal_pipeline`] 的扇入節,這裡是
//! spec-heavy 版)。三個決定:
//! 1. producer:try_push 失敗 → dropped 計數,一行收工;每筆 wake(flag 合併,不貴);
//! 2. 公平性:每輪對**單一條** channel 最多拿 `budget` 筆,輪完所有 channel 再回頭
//!    ——熱核塞爆自己的 ring 也搶不走冷核的輪次;
//! 3. 睡前條件:一整輪掃過去**零收穫**才准睡(那一刻 N 條 ring 同時空);
//!    Waker 的 sticky flag 補住「掃完 → 睡」縫隙裡新來的貨。
//!
//! ## [Iterate]
//! naive:aggregator 忙輪詢 N 條 ring → 加 sleep:睡前條件錯(只看單條空)會漏
//! → 全空一輪才睡 → Phase 2 加 budget:無上限的單條 drain 在熱核下會把
//! 這一輪永遠困在 ch0。
//!
//! ## [Trade-offs]
//! - budget 太小:輪次開銷占比升(每輪鎖/喚醒攤到更少筆);太大:冷核等待上限
//!   變成 `(N-1) × budget` 筆的處理時間。64 是「一次 cacheline 批量」量級的示範值。
//! - 每核一條 ring 的記憶體成本 = N × cap × 16B——用 cost-model 反推 cap,
//!   別憑感覺(4 核 × 256 × 16B = 16KB,毛毛雨;512 核就是 2MB,要想)。
//! - scale 到多條 aggregator:source 靜態分片,**不要** work-stealing——
//!   偷工作同時破壞單寫者與 per-core FIFO。
//!
//! ## [Dry-Run]
//! 測試 `budget_bounds_hot_core_per_round` 的手 trace(budget=4,熱核 ch0 塞 12、
//! 冷核 ch1 塞 1):輪 1:ch0 拿 seq0..=3 → ch1 拿它唯一一筆 → 輪 2:ch0 拿 4..=7
//! → 輪 3:ch0 拿 8..=11 → 輪 4:零收穫,睡。冷核那筆在 out 裡排在 ch0 第 5 筆
//! 之前——它只等了一個 budget,不是等熱核清空。
//!
//! 對照:彩排解答 `rehearsals/examples/sol_sim_k_fanin.rs`(同設計,單檔面試版)。

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

// ===================== 題目給的介面(與 sim k 相同,英文保留)=====================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    pub core: usize,
    pub seq: u64,
}

/// One SPSC channel per core (mock). Producer side is try-only —
/// **producers must never block**.(Mutex 換單檔簡潔;協定同 spsc_ring。)
pub struct Chan {
    cap: usize,
    q: Mutex<VecDeque<Record>>,
}

impl Chan {
    pub fn new(cap: usize) -> Self {
        Self {
            cap,
            q: Mutex::new(VecDeque::new()),
        }
    }

    pub fn try_push(&self, r: Record) -> Result<(), Record> {
        let mut q = self.q.lock().unwrap();
        if q.len() == self.cap {
            return Err(r);
        }
        q.push_back(r);
        Ok(())
    }

    pub fn try_pop(&self) -> Option<Record> {
        self.q.lock().unwrap().pop_front()
    }
}

/// Create `n` channels (core `i` uses channel `i`).
pub fn make_channels(n: usize, cap: usize) -> Vec<Arc<Chan>> {
    (0..n).map(|_| Arc::new(Chan::new(cap))).collect()
}

/// Wake-up primitive, same as `isr_pipeline`: wakes may coalesce, sleep may be
/// spurious(mock 帶 1s 安全網)。
#[derive(Default)]
pub struct Waker {
    flag: Mutex<bool>,
    cv: Condvar,
}

impl Waker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn wake(&self) {
        *self.flag.lock().unwrap() = true;
        self.cv.notify_one();
    }

    pub fn sleep(&self) {
        let mut flag = self.flag.lock().unwrap();
        if !*flag {
            let (f, _) = self.cv.wait_timeout(flag, Duration::from_secs(1)).unwrap();
            flag = f;
        }
        *flag = false;
    }
}

// ===================== 實作 =====================

/// producer 端:try_push,滿了計數(Relaxed——統計,不同步資料);
/// 喚醒交給 sticky flag 合併,每筆 wake 也只是一次鎖 + 偶爾 notify。
pub fn produce(rec: Record, ch: &Chan, waker: &Waker, dropped: &AtomicU64) {
    if ch.try_push(rec).is_err() {
        dropped.fetch_add(1, Ordering::Relaxed);
    }
    waker.wake();
}

/// aggregator:round-robin + per-ring budget;**一整輪零收穫才睡**;
/// stop 時上面剛 drain 到全空,殘貨已清,直接退。
pub fn aggregator_loop(
    chs: &[Arc<Chan>],
    waker: &Waker,
    stop: &AtomicBool,
    budget: usize,
    out: &Mutex<Vec<Record>>,
) {
    loop {
        // 掃到一整輪零收穫為止——這一刻所有 ring 同時是空的。
        loop {
            let mut moved = false;
            for ch in chs {
                let mut batch = Vec::new();
                while batch.len() < budget {
                    match ch.try_pop() {
                        Some(r) => batch.push(r),
                        None => break,
                    }
                }
                if !batch.is_empty() {
                    moved = true;
                    out.lock().unwrap().extend(batch);
                }
            }
            if !moved {
                break;
            }
        }
        if stop.load(Ordering::SeqCst) {
            return;
        }
        waker.sleep();
    }
}

// ===================== 測試 =====================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Instant;

    /// producer 永不 block 的 policy 面:cap=2 塞 5 筆 → 前 2 筆保序留下、
    /// 後 3 筆 drop + 計數。全單執行緒、確定性。
    #[test]
    fn producer_never_blocks_drop_counted() {
        let ch = Chan::new(2);
        let waker = Waker::new();
        let dropped = AtomicU64::new(0);
        for seq in 0..5 {
            produce(Record { core: 0, seq }, &ch, &waker, &dropped);
        }
        assert_eq!(dropped.load(Ordering::Relaxed), 3);
        assert_eq!(ch.try_pop().unwrap().seq, 0);
        assert_eq!(ch.try_pop().unwrap().seq, 1);
        assert!(ch.try_pop().is_none());
    }

    /// 檔頭 [Dry-Run] 的劇本(單執行緒、確定性):budget=4,ch0 塞 12、ch1 塞 1,
    /// stop 先立好 → aggregator 在本執行緒跑一趟 drain-到全空就返回。
    /// 斷言:ch1 那筆的位置在 out[4](第一輪 ch0 只拿得走 4 筆)——
    /// 冷核只等一個 budget,不是等熱核清空。
    #[test]
    fn budget_bounds_hot_core_per_round() {
        let chs = make_channels(2, 64);
        let waker = Waker::new();
        let stop = AtomicBool::new(true); // 先立 stop:drain 完直接退,不睡
        let out = Mutex::new(Vec::new());
        let dropped = AtomicU64::new(0);
        for seq in 0..12 {
            produce(Record { core: 0, seq }, &chs[0], &waker, &dropped);
        }
        produce(Record { core: 1, seq: 0 }, &chs[1], &waker, &dropped);
        aggregator_loop(&chs, &waker, &stop, 4, &out);
        let out = out.into_inner().unwrap();
        assert_eq!(out.len(), 13);
        assert_eq!(out[4], Record { core: 1, seq: 0 }, "冷核被熱核餓到了");
    }

    /// 守恆 + 每核順序(threaded):4 核 × 1000,out + dropped == 4000,
    /// 且每核 seq 嚴格遞增(per-core FIFO);跨核順序不斷言——本來就沒有。
    #[test]
    fn conservation_and_per_core_order() {
        const CORES: usize = 4;
        const PER_CORE: u64 = 1000;
        let chs = make_channels(CORES, 256);
        let waker = Arc::new(Waker::new());
        let stop = Arc::new(AtomicBool::new(false));
        let out = Arc::new(Mutex::new(Vec::new()));
        let dropped = Arc::new(AtomicU64::new(0));
        let agg = {
            let (c, w, s, o) = (chs.clone(), waker.clone(), stop.clone(), out.clone());
            thread::spawn(move || aggregator_loop(&c, &w, &s, 64, &o))
        };
        let producers: Vec<_> = (0..CORES)
            .map(|core| {
                let ch = chs[core].clone();
                let (w, d) = (waker.clone(), dropped.clone());
                thread::spawn(move || {
                    for seq in 0..PER_CORE {
                        produce(Record { core, seq }, &ch, &w, &d);
                    }
                })
            })
            .collect();
        for p in producers {
            p.join().unwrap();
        }
        let total = CORES as u64 * PER_CORE;
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let seen = out.lock().unwrap().len() as u64 + dropped.load(Ordering::SeqCst);
            if seen == total {
                break;
            }
            assert!(Instant::now() < deadline, "沒收斂:掉筆或 aggregator 睡死");
            thread::sleep(Duration::from_millis(10));
        }
        stop.store(true, Ordering::SeqCst);
        waker.wake();
        agg.join().unwrap();
        let out = out.lock().unwrap();
        let mut last = [None::<u64>; CORES];
        for r in out.iter() {
            if let Some(prev) = last[r.core] {
                assert!(r.seq > prev, "核 {} 順序破了", r.core);
            }
            last[r.core] = Some(r.seq);
        }
    }

    /// shutdown drain(單執行緒、確定性):3 核各塞 10 筆、stop 先立好 →
    /// aggregator 一趟收完 30 筆才返回——停之前每條 channel 都 drain 乾淨。
    #[test]
    fn shutdown_drains_every_channel() {
        let chs = make_channels(3, 64);
        let waker = Waker::new();
        let stop = AtomicBool::new(true);
        let out = Mutex::new(Vec::new());
        let dropped = AtomicU64::new(0);
        for (core, ch) in chs.iter().enumerate() {
            for seq in 0..10 {
                produce(Record { core, seq }, ch, &waker, &dropped);
            }
        }
        aggregator_loop(&chs, &waker, &stop, 4, &out);
        assert_eq!(out.into_inner().unwrap().len(), 30);
    }
}
