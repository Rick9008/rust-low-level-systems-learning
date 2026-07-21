//! # mpsc_list —— Vyukov intrusive MPSC(tokio 遠端 wake queue 的機制)
//!
//! [Clarify]
//! - 多生產、**單消費**、unbounded 的無鎖佇列:任意執行緒 `push`(wait-free,
//!   不會失敗、不會被 backpressure 卡住),單一 consumer `pop`。
//! - unbounded 是**需求**不是偷懶:這個結構的原生場景是 runtime 的
//!   跨執行緒 wake——wake 端在別人的執行緒上,絕不能因為佇列滿而阻塞或丟事件。
//!   記憶體帳因此轉嫁給上游(task 數有界 ⇒ 佇列自然有界)。
//! - pop 有三態:`Item` / `Empty` / **`Inconsistent`(縫)**——見下。
//!
//! [Abstract]
//! - 連結串列 + stub 節點:head=tail=stub 起步,push = 「swap tail(佔位)→
//!   接上 prev.next(發布)」恰好兩步,pop = 沿 next 走。
//! - **佔位與發布之間的縫是結構性的**:swap 完成的瞬間 tail 已指向新節點,
//!   但舊鏈還沒接上;consumer 此刻看到 `next==null && tail!=head` ⇒
//!   `Inconsistent`——元素邏輯上「在佇列裡」,實體上還走不到。
//!   MPMC ring 用 per-slot seq 把縫藏進「滿/空」判斷;這裡把縫做成顯式回值,
//!   因為 caller(runtime)知道怎麼處置:yield 後重試,producer 只差一個 store。
//! - 單 consumer 是型別強制(`Consumer` 不是 Clone、`pop` 拿 `&mut self`),
//!   它同時買到兩件事:head 無競爭(免 CAS),以及**免 epoch/hazard-pointer
//!   的記憶體回收**——unbounded lock-free 的真 boss(reclamation)被
//!   「只有 consumer 釋放、且只釋放已越過的節點」這條不變量整個拆掉。
//!
//! [Iterate]
//! - V0(壞):沒有 stub——空佇列時 head/tail 都是 null,push 要同時改兩者,
//!   單一 swap 做不到,被迫上 CAS 迴圈。stub 讓「串列永遠非空」,分支消失。
//! - V1(壞):push 先接鏈再 swap tail——兩個 producer 同時接同一個 prev.next,
//!   後者蓋掉前者,整條鏈斷。教訓與 mpmc_ring 相同:多寫者必須先佔位。
//! - V2(本版):swap 先佔位、store 後發布,縫顯式化為 `Inconsistent`。
//!
//! [Trade-offs]
//! - vs `mpmc_ring`:push 端 wait-free(swap 無重試)勝過 CAS 迴圈;
//!   代價是每 push 一次 heap 配置(~20–50ns,追求極致時上 node cache/arena)
//!   + pop 端的縫。消費端只有一個時它幾乎全面勝出——tokio 的選擇。
//! - vs `Mutex<VecDeque>`:無競爭時鎖版更簡單、cache 更友善(連續記憶體);
//!   本結構贏在高競爭 push(N 核同推:swap 一次 vs futex 排隊)與
//!   「push 端絕不阻塞」的硬保證。先量再換(cost-model 鐵律)。
//! - **lockless ≠ lock-free(正式定義)**:producer 卡在縫裡,consumer 就
//!   拿不到後面所有已 push 的元素(只能一直 Inconsistent)。與 mpmc_ring
//!   同一條誠實邊界,證據點在 push 的兩步之間。
//! - 反向組合:單生產多消費?本結構做不到(head 單寫者是地基)——
//!   要 MC 就回 [`crate::concurrency::mpmc_ring`](work-stealing deque 是
//!   另一條路,見 lockfree 家族手冊)。
//!
//! [Dry-Run]
//! - 邊界測試 [`tests::stub_lifecycle_hand_trace`] 手走 stub 的一生
//!   (空 pop → push → pop → 節點易主成新 stub)。
//! - 縫(Inconsistent)單執行緒觀察不到(push 兩步一氣呵成),
//!   `tests/loom_mpsc.rs` 的窮舉會走進去;stepper 圖解見
//!   `html_p/mpsc-interleaving-stepper.html`。

mod core_impl;

pub use core_impl::{Consumer, MpscList, PopResult, Producer, channel};

#[cfg(test)]
mod tests {
    use super::{PopResult, channel};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    /// boundary:stub 的一生,手走一整圈。
    ///
    /// 初始:stub S,head=tail=S,S.next=null,S.val=None。
    /// 1. pop():head=S,S.next==null,tail==S ⇒ **Empty**(不是 Inconsistent:
    ///    沒人佔過位)。
    /// 2. push(7):new node A{val:Some(7)};swap tail S→A(佔位);
    ///    S.next=A(發布)。狀態:S→A,head=S,tail=A。
    /// 3. pop():head=S,S.next=A(非 null)→ head 前進到 A → 釋放 S →
    ///    take A.val ⇒ **Item(7)**。A 從此 val=None——它就是新 stub。
    /// 4. pop():head=A,A.next==null,tail==A ⇒ **Empty**。一圈結束,
    ///    結構回到初始形狀(只是 stub 換人)。
    #[test]
    fn stub_lifecycle_hand_trace() {
        let (tx, mut rx) = channel();
        assert_eq!(rx.pop(), PopResult::Empty); // 1
        tx.push(7); // 2
        assert_eq!(rx.pop(), PopResult::Item(7)); // 3
        assert_eq!(rx.pop(), PopResult::Empty); // 4
    }

    /// boundary:單 producer FIFO(鏈的順序 = push 順序)。
    #[test]
    fn single_producer_fifo() {
        let (tx, mut rx) = channel();
        for i in 0..100 {
            tx.push(i);
        }
        for i in 0..100 {
            assert_eq!(rx.pop(), PopResult::Item(i));
        }
        assert_eq!(rx.pop(), PopResult::Empty);
    }

    /// 每次被 drop 就把共享計數器 +1(驗 Drop 回收,u64 看不出漏)。
    #[derive(Debug)]
    struct DropSpy(Arc<AtomicUsize>);
    impl Drop for DropSpy {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// boundary:帶著未消費元素 drop——連 stub 一起回收,不洩漏不重複。
    #[test]
    fn drop_reclaims_unconsumed() {
        let n = Arc::new(AtomicUsize::new(0));
        {
            let (tx, mut rx) = channel();
            for _ in 0..3 {
                tx.push(DropSpy(n.clone()));
            }
            match rx.pop() {
                PopResult::Item(spy) => drop(spy), // 消費 1 → +1
                other => panic!("預期 Item,得到 {other:?}"),
            }
        } // 剩 2 由 MpscList::drop 回收
        assert_eq!(n.load(Ordering::Relaxed), 3);
    }

    /// 多 producer 煙霧測試:4 條執行緒各推 25k,驗
    /// ①不丟不重 ②每個 producer 各自保序 ③push 端從不阻塞。
    /// (真正的並發證明在 tests/loom_mpsc.rs。)
    #[test]
    fn four_producers_smoke() {
        const PER: u64 = 25_000;
        let (tx, mut rx) = channel();
        let mut handles = Vec::new();
        for pid in 0..4u64 {
            let tx = tx.clone();
            handles.push(thread::spawn(move || {
                for i in 0..PER {
                    tx.push((pid << 32) | i); // 永不失敗、永不等待
                }
            }));
        }
        drop(tx);
        let mut got = 0u64;
        let mut last = [None::<u64>; 4];
        let mut all = Vec::new();
        while got < 4 * PER {
            match rx.pop() {
                PopResult::Item(v) => {
                    let (pid, i) = ((v >> 32) as usize, v & 0xffff_ffff);
                    if let Some(prev) = last[pid] {
                        assert!(i > prev, "producer {pid} 亂序:{prev} 後來 {i}");
                    }
                    last[pid] = Some(i);
                    all.push(v);
                    got += 1;
                }
                PopResult::Empty | PopResult::Inconsistent => thread::yield_now(),
            }
        }
        for h in handles {
            h.join().unwrap();
        }
        all.sort_unstable();
        let expect: Vec<u64> = (0..4u64)
            .flat_map(|pid| (0..PER).map(move |i| (pid << 32) | i))
            .collect();
        assert_eq!(all, expect);
    }
}
