//! # mpmc_list —— Michael–Scott unbounded MPMC(教學版:reclamation 攤開講)
//!
//! [Clarify]
//! - 多生產多消費、unbounded 的無鎖佇列。push 不會失敗;pop 空時 None,
//!   且 **None = 真空**(本結構沒有 Inconsistent 態,對照 `mpsc_list` 的縫)。
//! - **教學版邊界**:退休節點 Drop 才回收,運行期記憶體 = 歷史 push 總量。
//!   生產環境不可用——要運行期回收就需要 epoch/hazard pointer(crossbeam-epoch),
//!   那是另一整章。這個「不可用」本身就是本模組要教的主課。
//!
//! [Abstract]
//! - dummy 節點 + 兩個 CAS 端點:producers 搶「唯一的 null next」接鏈,
//!   consumers CAS head 前進。tail 允許落後,誰看到誰幫推(help)。
//! - **佔位=發布合一**:push 的 CAS 接鏈成功的瞬間,元素既被佔位也被發布
//!   ——沒有 Vyukov 的縫。這就是 M-S 滿足**正式 lock-free 定義**的原因:
//!   我 CAS 輸了 ⇔ 別人成功了 ⇔ 系統整體有進度。Vyukov 家族(mpmc_ring /
//!   mpsc_list)輸家可能在等一個睡死的贏家——lockless 而非 lock-free。
//! - pop 的「偷看再 CAS」:多個 popper 併發唯讀同一個 val(bitwise copy),
//!   head CAS 的唯一贏家才擁有它;輸家的副本是 MaybeUninit,丟掉不會 drop。
//!
//! [Iterate]
//! - V0(壞):沒有 dummy——空佇列 head/tail 皆 null,push 要同改兩指標,
//!   原子性做不到。dummy 讓兩端永遠有節點可站(與 mpsc_list 同招)。
//! - V1(壞):push 接鏈後「一定要」自己推完 tail 才返回——tail 推進變成
//!   臨界區,一人卡住全隊停,lock-free 性質毀滅。help 機制(誰看到落後
//!   誰幫推)是 M-S 的靈魂:把「必須完成的第二步」變成「任何人都能替你完成」。
//! - V2(本版):運行期不回收退休節點。正式版在這裡接 epoch/hazard;
//!   我們接的是一條 origin 鏈 + Drop 全清——正確、但記憶體只增不減。
//!
//! [Trade-offs]
//! - vs `mpmc_ring`(Vyukov):+unbounded、+正式 lock-free、+push 永不失敗;
//!   −每 push 一次配置、−**reclamation**(教學版直接不回收;工業版 epoch 的
//!   讀側 pin/unpin 各 ~幾 ns,但引入整個 crossbeam-epoch 的複雜度)。
//!   容量可以有硬上限時,Vyukov 幾乎總是更好的工程答案。
//! - vs `mpsc_list`:多了 MC(head 端 CAS 競爭),失去單 consumer 的兩大紅利
//!   (免 CAS pop、免 reclamation)——「MPSC 比 MPMC 簡單一半」的實證。
//! - help 的哲學:M-S 用「幫別人做完」買進度保證;Vyukov 用「等別人做完」
//!   省 per-op 成本。面試被問「lock-free 到底 free 在哪」,這一對就是答案。
//!
//! [Dry-Run]
//! - 邊界測試 [`tests::dummy_retirement_hand_trace`] 手走 dummy 退休一整圈。
//! - `tests/loom_mpmc_list.rs` 窮舉:2P 接鏈競爭(含 help)、2C 搶同一元素、
//!   mid-stream drop 不洩漏不重收。

mod core_impl;

pub use core_impl::MpmcList;

#[cfg(test)]
mod tests {
    use super::MpmcList;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    /// boundary:dummy 退休的一整圈,手走。
    ///
    /// 初始:dummy D0,head=tail=D0,D0.next=∅。
    /// 1. try_pop():D0.next==∅ ⇒ None(真空)。
    /// 2. push(7):node A{val:7};CAS D0.next ∅→A 成功(佔位=發布合一)
    ///    → CAS tail D0→A。狀態:D0→A,head=D0,tail=A。
    /// 3. try_pop():h=D0,next=A ⇒ 偷看 A.val=7 → CAS head D0→A 贏
    ///    ⇒ Some(7)。D0 退休(不釋放,Drop 收);A 成為新 dummy。
    /// 4. try_pop():h=A,A.next==∅ ⇒ None。一圈結束,形狀回到初始
    ///    (dummy 換人;D0 還掛在 origin 鏈上等 Drop)。
    #[test]
    fn dummy_retirement_hand_trace() {
        let q = MpmcList::new();
        assert_eq!(q.try_pop(), None); // 1
        q.push(7); // 2
        assert_eq!(q.try_pop(), Some(7)); // 3
        assert_eq!(q.try_pop(), None); // 4
    }

    /// boundary:單執行緒 FIFO(接鏈順序 = push 順序)。
    #[test]
    fn single_thread_fifo() {
        let q = MpmcList::new();
        for i in 0..100 {
            q.push(i);
        }
        for i in 0..100 {
            assert_eq!(q.try_pop(), Some(i));
        }
        assert_eq!(q.try_pop(), None);
    }

    /// 每次 drop +1(u64 沒解構子,漏 drop 看不出來)。
    #[derive(Debug)]
    struct DropSpy(Arc<AtomicUsize>);
    impl Drop for DropSpy {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// boundary:帶著未消費元素 drop——退休鏈 + 活元素全數回收、剛好一次
    /// (消費過的節點 val 不能被 Drop 再收一次 ⇒ 恰好 3,不是 4)。
    #[test]
    fn drop_reclaims_unconsumed_exactly_once() {
        let n = Arc::new(AtomicUsize::new(0));
        {
            let q = MpmcList::new();
            for _ in 0..3 {
                q.push(DropSpy(n.clone()));
            }
            drop(q.try_pop()); // 消費 1 → +1;其節點退休但值已搬走
        } // 剩 2 個活值由 Drop 段 2 回收
        assert_eq!(n.load(Ordering::Relaxed), 3);
    }

    /// 2P2C 壓力測試:不丟不重 + 每個 producer 各自保序。
    /// (真正的證明在 tests/loom_mpmc_list.rs。)
    #[test]
    fn two_producers_two_consumers_stress() {
        const PER: u64 = 30_000;
        let q = Arc::new(MpmcList::new());
        let done = Arc::new(AtomicUsize::new(0));
        let mut producers = Vec::new();
        for pid in 0..2u64 {
            let q = Arc::clone(&q);
            producers.push(thread::spawn(move || {
                for i in 0..PER {
                    q.push((pid << 32) | i); // unbounded:永不失敗
                }
            }));
        }
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
                            if done.load(Ordering::Relaxed) as u64 == 2 * PER {
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
            let mut last = [None::<u64>; 2];
            for &v in &got {
                let (pid, i) = ((v >> 32) as usize, v & 0xffff_ffff);
                if let Some(prev) = last[pid] {
                    assert!(i > prev, "producer {pid} 亂序");
                }
                last[pid] = Some(i);
            }
            all.extend(got);
        }
        all.sort_unstable();
        let expect: Vec<u64> = (0..2u64)
            .flat_map(|pid| (0..PER).map(move |i| (pid << 32) | i))
            .collect();
        assert_eq!(all, expect);
    }
}
