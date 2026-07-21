//! drill:mpsc_list —— 填 Vyukov MPSC 的「佔位/發布」兩步與三態 pop。
//!
//! 已給:節點與串列結構、stub 建構、Drop 回收、把手(Producer 可 Clone、
//! Consumer 不可——單消費是型別強制)。
//! 要填:`push`(恰好兩步,順序就是全部)/ `pop`(三態:Item/Empty/縫)。
//!
//! 填之前紙上回答:
//! 1. push 為什麼**必須**先 swap tail、再接 prev.next?反過來兩個 producer
//!    會發生什麼?
//! 2. swap 與 store 之間(縫)consumer 看到什麼?為什麼回 Inconsistent
//!    而不是 Empty?caller 拿到 Inconsistent 該做什麼?
//! 3. 為什麼這個結構不需要 epoch/hazard pointer 就能安全 free 節點?
//!    (誰釋放?釋放的前提是什麼?)
//!
//! 填完後跑 `cargo test -p reference --test loom_mpsc`——縫只有 loom 看得到。

use std::cell::UnsafeCell;
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};

struct Node<T> {
    next: AtomicPtr<Node<T>>,
    /// stub 是 None;真節點 push 時 Some,被取走後變回 None(成為新 stub)。
    val: UnsafeCell<Option<T>>,
}

/// pop 的三值結果——縫是顯式 API,不是藏起來的細節。
#[derive(Debug, PartialEq, Eq)]
pub enum PopResult<T> {
    Item(T),
    /// 真空:tail 就是 consumer 腳下這個節點。
    Empty,
    /// 縫:有 producer 佔了位(swap 完)還沒發布(store next)。重試訊號。
    Inconsistent,
}

pub struct MpscList<T> {
    /// 生產端:最新 push 的節點。多寫者,但只 swap(無 CAS 迴圈)。
    tail: AtomicPtr<Node<T>>,
    /// 消費端:stub 或最後一個已消費節點。單寫者 = consumer。
    head: UnsafeCell<*mut Node<T>>,
}

// SAFETY:tail 只走原子 swap/load;head 單寫者(Consumer 不可 Clone、
// pop 拿 &mut);節點資料可見性走 next 的 Release→Acquire;
// 釋放只由 consumer 做、且只釋放已越過的節點。T: Send 因元素跨執行緒移動。
unsafe impl<T: Send> Send for MpscList<T> {}
unsafe impl<T: Send> Sync for MpscList<T> {}

pub struct Producer<T> {
    list: Arc<MpscList<T>>,
}

impl<T> Clone for Producer<T> {
    fn clone(&self) -> Self {
        Self {
            list: Arc::clone(&self.list),
        }
    }
}

pub struct Consumer<T> {
    list: Arc<MpscList<T>>,
}

/// 建構(已給):stub 讓串列永遠非空,push/pop 不用處理 null 分支。
pub fn channel<T>() -> (Producer<T>, Consumer<T>) {
    let stub = Box::into_raw(Box::new(Node {
        next: AtomicPtr::new(ptr::null_mut()),
        val: UnsafeCell::new(None),
    }));
    let list = Arc::new(MpscList {
        tail: AtomicPtr::new(stub),
        head: UnsafeCell::new(stub),
    });
    (
        Producer {
            list: Arc::clone(&list),
        },
        Consumer { list },
    )
}

impl<T> Producer<T> {
    /// spec:wait-free push,恰好兩步:
    /// 1. `Box::into_raw` 配置節點(next=null, val=Some(item))
    /// 2. **佔位**:`tail.swap(node, Ordering?)`——為什麼要 AcqRel?
    ///    (Release 給誰看?Acquire 從誰拿?)
    /// 3. **發布**:`(*prev).next.store(node, Ordering?)`
    ///    (SAFETY 論證:prev 為什麼保證還活著?)
    ///
    /// 2 和 3 之間就是縫。
    pub fn push(&self, item: T) {
        let _ = item;
        todo!("spec: into_raw → swap(AcqRel) 佔位 → store(Release) 發布")
    }
}

impl<T> Consumer<T> {
    /// spec:單 consumer pop,O(1) 無 CAS。
    /// 1. 讀 head(UnsafeCell,單寫者是自己)
    /// 2. `(*head).next.load(Ordering?)`:
    ///    - 非 null → head 前進 → `Box::from_raw` 回收舊 head →
    ///      take 新節點的 val → Item(它從此是新 stub)
    ///    - null → 讀 tail 分辨:tail==head → Empty;否則 → Inconsistent
    pub fn pop(&mut self) -> PopResult<T> {
        todo!("spec: next 非 null 前進+回收;null 時用 tail 分辨 Empty/縫")
    }
}

impl<T> Drop for MpscList<T> {
    fn drop(&mut self) {
        // 已無並發:從 head 沿 next 回收整條鏈(未消費元素隨 Box drop)。
        let mut cur = unsafe { *self.head.get() };
        while !cur.is_null() {
            // SAFETY:&mut self 獨佔;cur 來自 head 或前一節點的 next。
            let boxed = unsafe { Box::from_raw(cur) };
            cur = boxed.next.load(Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::thread;

    /// boundary:stub 的一生(Empty → push → Item → Empty)。
    #[test]
    #[ignore = "drill:mpsc_list 未填"]
    fn stub_lifecycle() {
        let (tx, mut rx) = channel();
        assert_eq!(rx.pop(), PopResult::Empty);
        tx.push(7);
        assert_eq!(rx.pop(), PopResult::Item(7));
        assert_eq!(rx.pop(), PopResult::Empty);
    }

    /// boundary:單 producer FIFO。
    #[test]
    #[ignore = "drill:mpsc_list 未填"]
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

    /// boundary:帶著未消費元素 drop,不洩漏(DropSpy 計數)。
    #[test]
    #[ignore = "drill:mpsc_list 未填"]
    fn drop_reclaims_unconsumed() {
        struct DropSpy(Arc<AtomicUsize>);
        impl Drop for DropSpy {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }
        let n = Arc::new(AtomicUsize::new(0));
        {
            let (tx, mut rx) = channel();
            for _ in 0..3 {
                tx.push(DropSpy(n.clone()));
            }
            match rx.pop() {
                PopResult::Item(spy) => drop(spy),
                _ => panic!("預期 Item"),
            }
        }
        assert_eq!(n.load(Ordering::Relaxed), 3);
    }

    /// 多 producer 煙霧測試:4 執行緒各推 10k,不丟不重、各自保序。
    /// (真正的證明是 reference 的 loom_mpsc。)
    #[test]
    #[ignore = "drill:mpsc_list 未填"]
    fn four_producers_smoke() {
        const PER: u64 = 10_000;
        let (tx, mut rx) = channel();
        let mut handles = Vec::new();
        for pid in 0..4u64 {
            let tx = tx.clone();
            handles.push(thread::spawn(move || {
                for i in 0..PER {
                    tx.push((pid << 32) | i);
                }
            }));
        }
        drop(tx);
        let mut got = 0u64;
        let mut last = [None::<u64>; 4];
        while got < 4 * PER {
            match rx.pop() {
                PopResult::Item(v) => {
                    let (pid, i) = ((v >> 32) as usize, v & 0xffff_ffff);
                    if let Some(prev) = last[pid] {
                        assert!(i > prev, "producer {pid} 亂序");
                    }
                    last[pid] = Some(i);
                    got += 1;
                }
                PopResult::Empty | PopResult::Inconsistent => thread::yield_now(),
            }
        }
        for h in handles {
            h.join().unwrap();
        }
    }
}
