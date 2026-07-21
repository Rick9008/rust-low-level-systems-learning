//! # rcu_snapshot —— 讀多寫少的快照發布(std 版 poor-man's ArcSwap)
//!
//! [Clarify]
//! - 場景:讀多寫少(讀:寫 ≳ 100:1)的共享狀態——routing table、config、
//!   feature flags、(並發 trie/graph 的工程解也是這個形狀:讀走快照,
//!   寫 copy-path 換版本)。
//! - 需求:讀者拿到**一致的完整快照**(絕不撕裂)、讀的期間不擋任何人、
//!   讀者手上的舊版本在它讀完之前必須一直有效。
//! - 非目標:高頻小改(每次寫 clone 整份 T,寫熱就去 sharded_map /
//!   atomics);讀者要「最新值」的即時性(快照天生允許讀到舊版)。
//!
//! [Abstract]
//! - 核心:`Mutex<Arc<T>>`。讀者 `load()` = 鎖住→clone Arc→放鎖
//!   (臨界區 ~20ns,只有指標操作);之後**在鎖外**慢慢讀那份不可變快照。
//!   寫者在鎖外建好新版本,鎖住換指標。
//! - **RCU 的三件事,這裡各由誰做**:
//!   Read(讀者無鎖讀)→ 快照是 `Arc<T>`,讀取零同步;
//!   Copy(寫者旁路建新版)→ `update` 在鎖外跑 f;
//!   Update(發布)→ 換 `Arc` 指標,一步原子(對讀者而言)。
//!   剩下 RCU 最難的「**寬限期**(舊版何時能回收?)」——kernel RCU 要
//!   quiescent state 偵測、epoch 要 pin/unpin——在這裡被 `Arc` 的引用計數
//!   **免費解掉**:最後一個持有舊快照的讀者 drop 時,舊版自動回收。
//!   reclamation 問題(mpmc_list 的真 boss)再次被型別系統拆掉。
//! - 寫者的 read-copy-update 走**樂觀重試**:鎖外算新版,回來用
//!   `Arc::ptr_eq` 驗證沒人插隊,輸了拿最新版重算——f 可能跑多次,
//!   必須無副作用(mini-STM 的形狀)。
//!
//! [Iterate]
//! - V0(壞):`RwLock<T>` 直接讀——讀者要持鎖到讀完,長讀擋寫者;
//!   讀者之間還在 lock 的原子計數上互撞 cache line。
//! - V1(壞):`update` 持鎖跑 f——f 一慢,所有讀者跟著卡在 load 的
//!   lock 上,整個模式的意義蒸發。教訓:**鎖裡只准碰指標**。
//! - V2(本版):`Mutex<Arc<T>>` + 鎖外 CoW + ptr_eq 樂觀驗證。
//! - V3(本 repo 不做):讀路徑上那顆 Mutex 仍是全讀者共享的熱點,
//!   真正 lock-free 的 load 是 arc-swap crate 的本行——見 [Trade-offs]
//!   「為什麼 std 沒有 AtomicArc」。
//!
//! [Trade-offs]
//! - 讀者成本:uncontended lock + Arc clone ≈ ~20–40ns,之後讀取零同步、
//!   想讀多久讀多久。對照 `RwLock<T>`:讀者互不擋但持鎖期=整段讀取,
//!   且 read-unlock 也是一次原子 RMW。
//! - 擴展上限:所有讀者在同一顆 Mutex + 同一個 Arc 計數器上做 RMW——
//!   核多讀熱時這條 cache line 會 ping-pong。升級階梯:
//!   `arc-swap`(load 近乎 wait-free,靠 debt list/分散計數)→
//!   kernel RCU(讀端**零原子操作**,寬限期用 quiescent state 偵測)。
//! - **為什麼 std 沒有 `AtomicArc`**:「load 裸指標」和「把引用計數 +1」
//!   是兩個操作——中間那道縫裡,最後一個持有者可能 drop 掉物件,
//!   你 +1 的是已釋放的記憶體。要嘛拿鎖把兩步變一步(本版),
//!   要嘛上 hazard pointer / epoch / arc-swap 的 debt 機制。
//!   與 mpmc_list 的 reclamation、ws_deque 的「偷看可能正被覆寫的槽」
//!   是同一個問題的三張臉。
//! - 記憶體:讀者押著舊版不放,舊版就活著——版本數上限 = 並發讀者數 + 1。
//!   寫者每次 clone 整份 T:CoW 粒度是「整個值」,T 大且寫頻高時
//!   改用持久化結構(im 系)或收窄快照範圍。
//!
//! [Dry-Run]
//! - [`tests::grace_period_hand_trace`] 手走「舊版活到最後一個讀者放手」;
//!   [`tests::torn_read_impossible`] 用不變量驗快照永不撕裂;
//!   [`tests::optimistic_update_no_lost_write`] 驗兩個寫者對撞不丟更新。
//! - 全模組零 unsafe(組合 std 安全原語),不需 loom——正確性由
//!   Mutex/Arc 自身的契約承擔,這正是它值得當「第一步」的原因。

use std::sync::{Arc, Mutex};

/// 讀多寫少的快照格。`load` 拿快照、`store`/`update` 換版本。
pub struct RcuCell<T> {
    /// 唯一的臨界區:換/取指標。**鎖裡只准碰指標**(V1 的教訓)。
    current: Mutex<Arc<T>>,
}

impl<T> RcuCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            current: Mutex::new(Arc::new(value)),
        }
    }

    /// 讀者入口:抓當前快照。臨界區 = 一次 Arc clone(指標 + 計數 +1)。
    /// 拿到的 `Arc<T>` 在鎖外要讀多久都行——舊版本因你持有而保持有效
    /// (grace period = 引用計數)。O(1)。
    pub fn load(&self) -> Arc<T> {
        Arc::clone(&self.current.lock().unwrap())
    }

    /// 寫者:發布整個算好的新版本。適合「與舊值無關」的整份替換。O(1)。
    pub fn store(&self, value: T) {
        *self.current.lock().unwrap() = Arc::new(value);
    }

    /// 寫者:read-copy-update。`f` 在**鎖外**執行(讀者全程不受影響),
    /// 回鎖後用 `Arc::ptr_eq` 驗證期間沒有別的寫者插隊;輸了拿最新版重算。
    ///
    /// 契約:`f` 可能被執行多次(樂觀重試),必須無副作用。
    /// 均攤 O(clone T + f);寫者對撞時多付重算,讀者永遠不付。
    pub fn update(&self, mut f: impl FnMut(&T) -> T) {
        loop {
            let cur = self.load();
            let next = Arc::new(f(&cur)); // 鎖外:copy(讀者照跑)
            let mut slot = self.current.lock().unwrap();
            if Arc::ptr_eq(&slot, &cur) {
                *slot = next; // update:對讀者而言一步原子(換指標)
                return;
            }
            // 有寫者插隊:放鎖、拿新版重算(f 無副作用,重跑安全)。
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RcuCell;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    /// 每次 drop +1:量測「版本何時真正被回收」。
    struct DropSpy {
        n: Arc<AtomicUsize>,
        #[allow(dead_code)]
        tag: u32,
    }
    impl Drop for DropSpy {
        fn drop(&mut self) {
            self.n.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// boundary:grace period 手走一整圈。
    ///
    /// 1. new(v1):cell 持有 v1(計數 1)。
    /// 2. reader = load():v1 計數 2。
    /// 3. store(v2):cell 改持 v2;v1 計數回 1(**只剩 reader 押著**)——
    ///    drop 數 = 0,舊版還活著,reader 繼續讀到一致的 v1。
    /// 4. drop(reader):v1 計數歸零 → 此刻才回收(drop 數 = 1)。
    ///    寬限期結束的判定,一行都沒寫——Arc 計數就是它。
    #[test]
    fn grace_period_hand_trace() {
        let n = Arc::new(AtomicUsize::new(0));
        let cell = RcuCell::new(DropSpy {
            n: n.clone(),
            tag: 1,
        });
        let reader = cell.load(); // 2
        cell.store(DropSpy {
            n: n.clone(),
            tag: 2,
        }); // 3
        assert_eq!(n.load(Ordering::Relaxed), 0, "讀者押著,v1 不得回收");
        assert_eq!(reader.tag, 1, "手上的快照不因發布而改變");
        drop(reader); // 4
        assert_eq!(n.load(Ordering::Relaxed), 1, "最後的讀者放手,v1 立即回收");
    }

    /// boundary:快照永不撕裂——雙欄位不變量(a + b == 100)在
    /// 寫者連續換版下,每一次 load 都必須完整成立。
    #[test]
    fn torn_read_impossible() {
        #[derive(Clone)]
        struct Pair {
            a: u32,
            b: u32,
        }
        let cell = Arc::new(RcuCell::new(Pair { a: 0, b: 100 }));
        let writer = {
            let cell = Arc::clone(&cell);
            thread::spawn(move || {
                for i in 1..=1000u32 {
                    let v = i % 101;
                    cell.store(Pair { a: v, b: 100 - v });
                }
            })
        };
        let mut readers = Vec::new();
        for _ in 0..4 {
            let cell = Arc::clone(&cell);
            readers.push(thread::spawn(move || {
                for _ in 0..2000 {
                    let snap = cell.load();
                    assert_eq!(snap.a + snap.b, 100, "快照撕裂:讀到半新半舊");
                }
            }));
        }
        writer.join().unwrap();
        for r in readers {
            r.join().unwrap();
        }
    }

    /// boundary:兩個寫者用 update 對撞,樂觀重試不丟更新
    /// (ptr_eq 驗證失敗 → 拿新版重算,效果同 CAS 迴圈)。
    #[test]
    fn optimistic_update_no_lost_write() {
        const PER: usize = 1000;
        let cell = Arc::new(RcuCell::new(0usize));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let cell = Arc::clone(&cell);
            handles.push(thread::spawn(move || {
                for _ in 0..PER {
                    cell.update(|v| v + 1);
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(
            *cell.load(),
            2 * PER,
            "有更新在對撞中蒸發 = ptr_eq 驗證壞了"
        );
    }

    /// 讀多寫少煙霧測試:8 讀者狂 load、1 寫者偶爾換版,
    /// 讀到的版本號必須單調不減(發布是全序的:都經過同一把鎖)。
    #[test]
    fn readers_see_monotonic_versions() {
        let cell = Arc::new(RcuCell::new(0u64));
        let writer = {
            let cell = Arc::clone(&cell);
            thread::spawn(move || {
                for v in 1..=100u64 {
                    cell.store(v);
                    thread::yield_now();
                }
            })
        };
        let mut readers = Vec::new();
        for _ in 0..8 {
            let cell = Arc::clone(&cell);
            readers.push(thread::spawn(move || {
                let mut last = 0u64;
                for _ in 0..1000 {
                    let v = *cell.load();
                    assert!(v >= last, "版本倒退:{last} 之後讀到 {v}");
                    last = v;
                }
            }));
        }
        writer.join().unwrap();
        for r in readers {
            r.join().unwrap();
        }
    }
}
