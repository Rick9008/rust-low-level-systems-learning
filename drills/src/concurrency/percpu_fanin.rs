// ⚠⚠ 防雷:本檔是 sim k(per-core telemetry fan-in)的填空版,spec 註解含解法方向。
// 計時場排在 8/1——在那之前不要讀。自定規則:跑題前不開 oracle/sol。

//! drill:percpu_fanin —— 填 produce 與 aggregator_loop(sim k 的填空版)。
//!
//! 已給:`Chan` / `Record` / `Waker` / `make_channels`(借 reference 的,題目給定件)。
//! 要填:`produce` / `aggregator_loop` 兩個函式。
//!
//! 核心不變量:
//! - producer **絕不 block**——滿了走你的 policy(drop + 計數),不准等;
//! - 熱核不能餓死冷核——每輪對單一條 channel 最多拿 `budget` 筆;
//! - 睡前條件是「一整輪掃過去零收穫」——只看單條空就睡會漏;
//! - stop 前每條 channel 都要 drain 完。
//!
//! 設計取捨見 reference 同名模組檔頭(**跑過 sim k 之後**再讀)。

use reference::concurrency::percpu_fanin::{Chan, Record, Waker};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};

/// spec:producer——`try_push`;`Err`(滿)→ `dropped` +1(Relaxed,統計);
/// 每筆 `wake()`(sticky flag 會合併,不貴)。整個函式不准有任何等待。
pub fn produce(rec: Record, ch: &Chan, waker: &Waker, dropped: &AtomicU64) {
    todo!("spec: try_push;滿了計數;wake 交給 flag 合併")
}

/// spec:aggregator——外圈:內圈 drain 到「一整輪零收穫」(此刻全部 ring 同時空)
/// → 查 `stop`(是 → 直接 return,殘貨剛清完)→ `waker.sleep()`。
/// 內圈:round-robin 掃每條 channel,單條一輪最多 `budget` 筆,批次寫進 `out`
/// (一次鎖一批);任何一條有收穫就再掃一輪。
pub fn aggregator_loop(
    chs: &[Arc<Chan>],
    waker: &Waker,
    stop: &AtomicBool,
    budget: usize,
    out: &Mutex<Vec<Record>>,
) {
    todo!("spec: round-robin + budget;全空一輪才睡;stop 時已 drain 完直接退")
}

#[cfg(test)]
mod tests {
    use super::*;
    use reference::concurrency::percpu_fanin::make_channels;
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::{Duration, Instant};

    /// cap=2 塞 5 筆:前 2 筆保序留下、後 3 筆 drop + 計數;produce 全程不等。
    #[test]
    #[ignore = "drill:填完 produce 後拔掉"]
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

    /// budget=4,ch0 塞 12、ch1 塞 1,stop 先立好(drain 完就返回):
    /// ch1 那筆必須出現在 out[4]——冷核只等一個 budget,不是等熱核清空。
    #[test]
    #[ignore = "drill:填完 aggregator_loop 後拔掉"]
    fn budget_bounds_hot_core_per_round() {
        let chs = make_channels(2, 64);
        let waker = Waker::new();
        let stop = AtomicBool::new(true);
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

    /// 守恆 + 每核順序:4 核 × 1000,out + dropped == 4000,每核 seq 嚴格遞增。
    #[test]
    #[ignore = "drill:填完 produce/aggregator_loop 後拔掉"]
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

    /// shutdown drain:3 核各塞 10 筆、stop 先立好 → 一趟收完 30 筆才返回。
    #[test]
    #[ignore = "drill:填完 aggregator_loop 後拔掉"]
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
