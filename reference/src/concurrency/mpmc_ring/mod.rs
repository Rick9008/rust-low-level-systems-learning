//! # mpmc_ring —— Vyukov bounded MPMC queue(SPSC 的兩刀升級)
//!
//! [Clarify]
//! - 多生產多消費的固定容量佇列:任意執行緒 `try_push`、任意執行緒 `try_pop`。
//! - 滿時 Err 歸還(backpressure 交給 caller)、空時 None——**非阻塞**;
//!   要 block-on-full/empty 語意就退回 [`crate::concurrency::bounded_queue`]
//!   (等待是 condvar 的主場,lock-free 繞不掉它)。
//! - FIFO 以「取號順序」定義:每個 producer 自己的元素保序;跨 producer 看
//!   誰先搶到號。
//!
//! [Abstract]
//! - 從 [`crate::concurrency::spsc_ring`] 出發只動兩刀:
//!   ① index 有多個寫者 → 佔位改 **CAS 取號**(load+store 會兩人拿同號);
//!   ② 佔位先行後,「號進了、資料還沒進」出現**縫** → tail 不再能當發布訊號,
//!   每槽加一個 `seq` 接手:`pos`(輪空)→ `pos+1`(已發布)→ `pos+cap`(已釋放)。
//! - happens-before 全掛在 seq(Release/Acquire);head/tail 退化成取號機
//!   (CAS Relaxed)。SPSC 那兩條「Acquire 讀對方 index」的邊整個消失,
//!   head 與 tail 甚至互不比較。
//! - seq 自帶圈數 ⇒ ABA 免疫,與自由跑計數器同招。
//!
//! [Iterate]
//! - V0(壞):沿用 SPSC 的「先寫槽、再 store tail」→ 兩個 producer 讀到同一個
//!   tail,同槽對寫、一筆消失。教訓:多寫者的 index 必須先佔後寫。
//! - V1(壞):CAS 取號後仍靠 tail 當發布訊號 → consumer 看 tail 前進就讀,
//!   讀到佔了號還沒寫完的垃圾。教訓:佔位≠發布,需要 per-slot 訊號。
//! - V2(本版):per-slot seq 三態。代價:每槽多 8B + 每 op 多一對 seq 存取。
//!
//! [Trade-offs]
//! - vs `Mutex<VecDeque>`:無競爭時差距不大(uncontended lock ~20ns);
//!   高競爭下勝在無 futex syscall、臨界區為零,但 **tail 那條 cache line
//!   仍在所有 producer 核心間 ping-pong**——吞吐不隨 producer 數線性成長。
//!   真要線性 scale,答案在佇列之外:N×SPSC fan-in(`signal_pipeline`)或
//!   per-core sharding(`hw_bridge` 的 sharded server)。
//! - vs SPSC:單 op 多一對 seq Acquire/Release + CAS;換到的是任意端點數。
//!   端點數已知且少時,SPSC 陣列仍是效能上界。
//! - **lockless ≠ lock-free(正式定義)**:producer 在佔位與發布之間停滯,
//!   所有 consumer 就卡在那一格——沒有全系統進度保證。面試必講的誠實邊界。
//! - 無 `len()`:MPMC 下任何 len 都是「算出來那刻就過期」的快照
//!   (見 ring_buffer 教材「len 之死」)。
//!
//! [Dry-Run]
//! - 邊界測試 [`tests::cap1_seq_state_machine_hand_trace`] 手走 cap=1 的
//!   seq 三態一整圈;wrap 測試用 `with_start` 把計數器逼到 usize::MAX 附近。
//! - 並發正確性不可 fuzz:`tests/loom_mpmc.rs` 窮舉 2P1C / 1P2C 的所有交錯。

mod core_impl;

pub use core_impl::MpmcRing;

#[cfg(test)]
mod tests {
    use super::MpmcRing;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    /// boundary:手走 cap=2 的 seq 三態一整圈 + cap=1 為什麼不准存在。
    ///
    /// **cap=1 的退化**(這個測試先驗它):「已發布」= seq=pos+1、
    /// 「下一圈輪空」= seq=pos+cap;cap=1 時兩者同值,producer 分不出
    /// 「滿」跟「可搶」,會覆寫未消費資料——所以 `new(1)` 的實際容量是 2
    /// (原版 Vyukov 同樣 assert cap ≥ 2)。
    ///
    /// **cap=2 全程 trace**(mask=1;初始 head=tail=0,seq=[0,1]):
    /// 1. try_push(7):pos=0,slot0.seq=0,dif=0 → CAS tail 0→1 → 寫 7 →
    ///    seq=1(已發布)。
    /// 2. try_push(8):pos=1,slot1.seq=1,dif=0 → CAS tail 1→2 → 寫 8 →
    ///    seq=2。
    /// 3. try_push(9):pos=2,slot=buf[2&1]=slot0,seq=1,
    ///    dif=1−2=−1<0 → **滿**,Err(9) 歸還。
    /// 4. try_pop():pos=head=0,slot0.seq=1,dif=1−(0+1)=0 → CAS head 0→1
    ///    → 讀出 7 → seq=0+2=2(已釋放,恰是下一圈 producer 期望的 pos)。
    /// 5. try_pop():pos=1,slot1.seq=2,dif=0 → 讀出 8 → seq=3。
    /// 6. try_pop():pos=2,slot0.seq=2,dif=2−3=−1<0 → **空**,None。
    /// 7. try_push(9):pos=2,slot0.seq=2,dif=0(第二圈輪空)→ 寫 9 → seq=3。
    #[test]
    fn cap2_seq_state_machine_hand_trace() {
        let q = MpmcRing::new(1);
        assert_eq!(q.capacity(), 2, "cap=1 必須被上調到 2(三態塌縮)");
        q.try_push(7).unwrap();
        q.try_push(8).unwrap();
        assert_eq!(q.try_push(9), Err(9)); // 步驟 3:滿,歸還同一元素
        assert_eq!(q.try_pop(), Some(7));
        assert_eq!(q.try_pop(), Some(8));
        assert_eq!(q.try_pop(), None); // 步驟 6:空
        q.try_push(9).unwrap(); // 步驟 7:第二圈重用 slot0
        assert_eq!(q.try_pop(), Some(9));
    }

    /// boundary:滿/空/歸還 + FIFO(cap=4,單執行緒)。
    #[test]
    fn full_empty_fifo_roundtrip() {
        let q = MpmcRing::new(4);
        for i in 0..4 {
            q.try_push(i).unwrap();
        }
        assert_eq!(q.try_push(99), Err(99));
        for i in 0..4 {
            assert_eq!(q.try_pop(), Some(i));
        }
        assert_eq!(q.try_pop(), None);
    }

    /// boundary:計數器溢位——head/tail 從 usize::MAX-1 起跑,跨過 0 的
    /// wrap 點 FIFO 不亂(自由跑計數器 + power-of-2 mask 的圈數算術)。
    #[test]
    fn counter_overflow_wrap_is_seamless() {
        let q = MpmcRing::with_start(2, usize::MAX - 1);
        for i in 0..8 {
            q.try_push(i).unwrap();
            assert_eq!(q.try_pop(), Some(i));
        }
    }

    /// 每次被 drop 就把共享計數器 +1(u64 沒有解構子,漏 drop 看不出來)。
    #[derive(Debug)]
    struct DropSpy(Arc<AtomicUsize>);
    impl Drop for DropSpy {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// boundary:帶著未消費元素 drop——Drop 排空回收,不洩漏、不 double-drop。
    #[test]
    fn drop_reclaims_unconsumed() {
        let n = Arc::new(AtomicUsize::new(0));
        {
            let q = MpmcRing::new(4);
            for _ in 0..3 {
                q.try_push(DropSpy(n.clone())).unwrap();
            }
            drop(q.try_pop()); // 消費 1 → +1
        } // 剩 2 由 MpmcRing::drop 回收
        assert_eq!(n.load(Ordering::Relaxed), 3);
    }

    /// 2P2C 壓力測試:每個 producer 的元素帶 producer id,驗
    /// ①一個不丟不重(multiset 相等)②**每個 producer 各自保序**
    /// (MPMC 的 FIFO 契約以取號順序定義)。
    /// (真正的並發證明在 tests/loom_mpmc.rs;這裡是量大的 sanity。)
    #[test]
    fn two_producers_two_consumers_stress() {
        const PER_PRODUCER: u64 = 50_000;
        let q = Arc::new(MpmcRing::new(8));
        let mut producers = Vec::new();
        for pid in 0..2u64 {
            let q = Arc::clone(&q);
            producers.push(thread::spawn(move || {
                for i in 0..PER_PRODUCER {
                    let mut item = (pid << 32) | i;
                    while let Err(back) = q.try_push(item) {
                        item = back;
                        thread::yield_now();
                    }
                }
            }));
        }
        let done = Arc::new(AtomicUsize::new(0));
        let mut consumers = Vec::new();
        for _ in 0..2 {
            let q = Arc::clone(&q);
            let done = Arc::clone(&done);
            consumers.push(thread::spawn(move || {
                let mut got = Vec::new();
                loop {
                    match q.try_pop() {
                        Some(v) => {
                            got.push(v);
                            done.fetch_add(1, Ordering::Relaxed);
                        }
                        None => {
                            if done.load(Ordering::Relaxed) as u64 == 2 * PER_PRODUCER {
                                break;
                            }
                            thread::yield_now();
                        }
                    }
                }
                got
            }));
        }
        for p in producers {
            p.join().unwrap();
        }
        let mut all: Vec<u64> = Vec::new();
        for c in consumers {
            let got = c.join().unwrap();
            // 每個 consumer 收到的序列裡,同一 producer 的 i 必須遞增(保序)。
            let mut last = [None::<u64>; 2];
            for &v in &got {
                let (pid, i) = ((v >> 32) as usize, v & 0xffff_ffff);
                if let Some(prev) = last[pid] {
                    assert!(i > prev, "producer {pid} 的順序亂了:{prev} 之後來 {i}");
                }
                last[pid] = Some(i);
            }
            all.extend(got);
        }
        // multiset 相等:總量對、每個值恰出現一次。
        all.sort_unstable();
        let expect: Vec<u64> = (0..2u64)
            .flat_map(|pid| (0..PER_PRODUCER).map(move |i| (pid << 32) | i))
            .collect();
        assert_eq!(all, expect);
    }
}
