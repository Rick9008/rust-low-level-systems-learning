//! MPSC list(Vyukov intrusive MPSC queue 的 Box 節點版)核心演算法。
//!
//! 這個檔案被兩個地方 include:
//! 1. `mpsc_list/mod.rs`(lib 本體,sync = std)
//! 2. `tests/loom_mpsc.rs`(loom 驗證,sync = loom)
//!
//! 所以這裡只准用 `crate::sync_shim` 提供的同步原語,不直接碰 std::sync。

use crate::sync_shim as sync;
use std::ptr;
use sync::atomic::{AtomicPtr, Ordering};

/// 串列節點。`val` 用 shim 的 UnsafeCell 包住,讓 loom 能追蹤
/// 「producer 建構時寫、consumer 取走時寫」這兩筆存取真的不重疊。
struct Node<T> {
    next: AtomicPtr<Node<T>>,
    /// stub 節點是 None;真節點 push 時裝 Some,被 pop 取走後變回 None
    /// (取走的那一刻它就成為新 stub)。
    val: sync::UnsafeCell<Option<T>>,
}

/// pop 的三值結果——「縫」是這個資料結構的**顯式 API**,不是藏起來的細節。
#[derive(Debug, PartialEq, Eq)]
pub enum PopResult<T> {
    /// 取到元素。
    Item(T),
    /// 佇列真的空(tail 就是 consumer 腳下這個節點)。
    Empty,
    /// **縫**:有 producer 已經 swap 走 tail(佔位)、還沒把 prev.next 連上
    /// (發布)。元素「在佇列裡」但 consumer 還走不到——不是空,等一下再來。
    /// tokio 對它的處置:yield 後重試(它知道 producer 只剩一個 store 就好)。
    Inconsistent,
}

/// 多生產單消費、unbounded 的無鎖連結串列佇列(Vyukov)。
///
/// 與 [`crate::concurrency::mpmc_ring`] 相反的取捨象限:不限容量、
/// push 端**wait-free**(一個 swap,無重試迴圈),代價是每 push 一次
/// heap 配置 + pop 端有「縫」(Inconsistent)。tokio 用它收跨執行緒的
/// remote wake,因為 wake 端絕不能被 backpressure 卡住。
///
/// 記憶體回收免 epoch/hazard-pointer 的原因(unbounded lock-free 的真 boss
/// 在這裡被型別系統拆掉):**只有單一 consumer 會釋放節點**,而它只釋放
/// 「已越過」的節點;要越過節點 prev 必須先看到 prev.next 非 null,
/// 也就是 producer 對 prev 的最後一筆寫入(store next)之後——
/// 沒有任何執行緒會碰已釋放的記憶體。
pub struct MpscList<T> {
    /// 生產端:最新 push 的節點。多寫者,但只用 swap(無 CAS 重試)。
    tail: AtomicPtr<Node<T>>,
    /// 消費端:stub 或最後一個已消費節點。單寫者 = consumer 本人。
    head: sync::UnsafeCell<*mut Node<T>>,
}

// SAFETY:
// - push 端只碰 tail(atomic swap)與自己剛配置、尚未共享的節點,
//   以及 prev.next(atomic store)——全是原子操作。
// - head 只有單一 consumer 讀寫(Consumer 不是 Clone、pop 拿 &mut self)。
// - 節點資料的可見性:producer 在 swap 前寫完 val,經
//   prev.next store(Release) → consumer load(Acquire) 建立 happens-before。
// - T: Send 因為元素跨執行緒移動。
unsafe impl<T: Send> Send for MpscList<T> {}
unsafe impl<T: Send> Sync for MpscList<T> {}

/// 生產把手:**可 Clone**(多生產者是本結構的重點)。
pub struct Producer<T> {
    list: sync::Arc<MpscList<T>>,
}

impl<T> Clone for Producer<T> {
    fn clone(&self) -> Self {
        Self {
            list: sync::Arc::clone(&self.list),
        }
    }
}

/// 消費把手:不是 Clone——單一 consumer 由型別系統保證,
/// head 與節點釋放的獨佔權都靠它。
pub struct Consumer<T> {
    list: sync::Arc<MpscList<T>>,
}

/// 建立 MPSC channel。初始一個 stub 節點,head=tail=stub
/// (stub 讓 push/pop 永遠不用處理「串列全空沒有節點」的分支)。
pub fn channel<T>() -> (Producer<T>, Consumer<T>) {
    let stub = Box::into_raw(Box::new(Node {
        next: AtomicPtr::new(ptr::null_mut()),
        val: sync::UnsafeCell::new(None),
    }));
    let list = sync::Arc::new(MpscList {
        tail: AtomicPtr::new(stub),
        head: sync::UnsafeCell::new(stub),
    });
    (
        Producer {
            list: sync::Arc::clone(&list),
        },
        Consumer { list },
    )
}

impl<T> Producer<T> {
    /// wait-free push:恰好一次 swap + 一次 store,無迴圈、無失敗路徑。
    /// unbounded——配置失敗以外不存在「滿」。
    pub fn push(&self, item: T) {
        let node = Box::into_raw(Box::new(Node {
            next: AtomicPtr::new(ptr::null_mut()),
            val: sync::UnsafeCell::new(Some(item)),
        }));
        // 佔位:把自己 swap 成新 tail。
        // Release:讓 node 的內容(val 寫入)對「下一個 swap 到我們的 producer」
        // 可見(它要寫我們的 next 欄位);Acquire:讓 prev 的內容對我們可見
        // (我們要寫它的 next 欄位)。
        let prev = self.list.tail.swap(node, Ordering::AcqRel);
        // ↑↓ 之間就是「縫」:tail 已指向新節點,但舊鏈還沒接上——
        // consumer 此刻走到 prev 會看到 next==null 而 tail!=prev ⇒ Inconsistent。
        // 本執行緒若在這裡被 deschedule,縫會維持到它醒來(lockless 的證據點)。
        //
        // SAFETY:prev 永遠是有效指標——只有 consumer 釋放節點,且它只釋放
        // 「已越過」的節點;越過 prev 的前提是看到 prev.next 非 null,
        // 也就是下面這行 store 之後。
        // 發布:Release 讓 node 的 val 寫入 happens-before consumer 的
        // next Acquire load。
        unsafe {
            (*prev).next.store(node, Ordering::Release);
        }
    }
}

impl<T> Consumer<T> {
    /// 單 consumer pop。O(1),無 CAS(head 只有本執行緒寫)。
    /// 回值三態見 [`PopResult`]——`Inconsistent` 是重試訊號,不是錯誤。
    pub fn pop(&mut self) -> PopResult<T> {
        let list = &*self.list;
        // SAFETY(with 唯讀):head 只有單一 consumer 讀寫(&mut self)。
        let head = list.head.with(|p| unsafe { *p });
        // SAFETY:head 指向 stub 或已消費節點,consumer 獨佔其生命週期
        // (見 MpscList doc 的回收論證),deref 安全。
        // Acquire 配對 producer 的 next store(Release):看到非 null,
        // 該節點的 val 寫入保證可見。
        let next = unsafe { (*head).next.load(Ordering::Acquire) };
        if next.is_null() {
            // 走不下去。分辨「真空」與「縫」:
            // Acquire 配對 producer swap 的 Release 側(同步邊不帶資料,
            // 只是讓 tail 的讀不會停在快取裡的舊值太久;判斷本身允許保守——
            // 誤判成 Inconsistent 只是多重試一輪)。
            let tail = list.tail.load(Ordering::Acquire);
            return if ptr::eq(tail, head) {
                PopResult::Empty // tail 還是腳下這個節點:真的沒人 push 過
            } else {
                PopResult::Inconsistent // 有人佔了位還沒發布:等它把鏈接上
            };
        }
        // 前進:next 成為新 head(它的值取走後,它就是新 stub)。
        // SAFETY(with_mut 獨佔):同上,head 單寫者。
        list.head.with_mut(|p| unsafe { *p = next });
        // SAFETY:舊 head 已被越過,此後無人可達(producer 只碰 tail 端;
        // consumer 已離開)——回收它。它的 val 必為 None(stub 或已取走),
        // 不會 double-drop 元素。
        drop(unsafe { Box::from_raw(head) });
        // SAFETY(with_mut 獨佔):val 的取走只有單一 consumer 會做,
        // 且每個節點只被越過一次;producer 對此節點的 val 寫入
        // happens-before 上面的 Acquire。
        let item = unsafe { &*next }.val.with_mut(|p| unsafe { (*p).take() });
        match item {
            Some(v) => PopResult::Item(v),
            // 走得到的節點必帶值:stub 只會出現在 head 位置,不會是 next。
            None => unreachable!("linked node must carry a value"),
        }
    }
}

impl<T> Drop for MpscList<T> {
    /// 兩個把手都 drop 後(Arc 歸零)才會走到這:已無並發。
    /// 從 head 沿 next 走到底,回收所有節點;未消費元素(val=Some)
    /// 隨 Box drop 一併釋放,不洩漏。
    fn drop(&mut self) {
        let mut cur = self.head.with(|p| unsafe { *p });
        while !cur.is_null() {
            // SAFETY:&mut self ⇒ 獨佔整條串列;cur 來自 head 或前一節點的
            // next,必為有效、未釋放的節點。
            let boxed = unsafe { Box::from_raw(cur) };
            cur = boxed.next.load(Ordering::Relaxed);
        }
    }
}
