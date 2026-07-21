//! MPMC list(Michael–Scott unbounded queue,教學版)核心演算法。
//!
//! 這個檔案被兩個地方 include:
//! 1. `mpmc_list/mod.rs`(lib 本體,sync = std)
//! 2. `tests/loom_mpmc_list.rs`(loom 驗證,sync = loom)
//!
//! 所以這裡只准用 `crate::sync_shim` 提供的同步原語,不直接碰 std::sync。
//!
//! **教學版的邊界(誠實聲明)**:退休節點(被 pop 越過的 dummy)**不在
//! 運行期回收**,全部留到 Drop 一次清——記憶體帳 = 歷史 push 總量,
//! 不是佇列深度。這不是偷懶,是把「reclamation 是 unbounded MPMC 的
//! 真 boss」這件事攤開來:運行期安全回收需要 epoch / hazard pointer
//! 整套機器(crossbeam-epoch),那超出 45 分鐘與本 repo 的教學範圍。

use crate::sync_shim as sync;
use std::mem::MaybeUninit;
use std::ptr;
use sync::atomic::{AtomicPtr, Ordering};

/// 串列節點。dummy(head 所指)與已消費節點的 val 已「邏輯搬走」;
/// 「誰的 val 還活著」由 head 位置隱含追蹤:head 之後(不含)都活著。
struct Node<T> {
    next: AtomicPtr<Node<T>>,
    val: sync::UnsafeCell<MaybeUninit<T>>,
}

/// Michael–Scott unbounded MPMC queue(教學版:退休節點 Drop 才回收)。
///
/// 與 [`crate::concurrency::mpmc_ring`](Vyukov)的本質差異一句話:
/// **M-S 的 push 用「CAS 接上 prev.next」同時完成佔位與發布——沒有縫**,
/// 所以它滿足正式的 lock-free 定義(任一執行緒被 deschedule,其他人
/// 仍能推進);代價是節點動態配置 + 回收問題(見檔頭聲明)。
pub struct MpmcList<T> {
    /// dummy 指標;`head.next` 才是第一個真元素。consumers CAS 推進。
    head: AtomicPtr<Node<T>>,
    /// 最後一個節點(可能落後——見 push 的 help)。producers CAS 推進。
    tail: AtomicPtr<Node<T>>,
    /// 歷史鏈起點(建構時的第一個 dummy,此後不變)。
    /// 所有節點永遠串在一條 next 鏈上,Drop 從這裡走就能全部回收。
    origin: *mut Node<T>,
}

// SAFETY:
// - head/tail 只走 CAS/load;節點在 Drop 前不釋放(教學版聲明),
//   所以任何執行緒持有的節點指標永遠有效——不需要 hazard pointer。
// - val 的互斥:push 在發布前寫(獨佔新節點);pop 只複製讀,
//   唯一「擁有」該值的是 head CAS 的贏家;Drop 靠 head 位置分辨死活。
// - T: Send 因為元素跨執行緒移動。
unsafe impl<T: Send> Send for MpmcList<T> {}
unsafe impl<T: Send> Sync for MpmcList<T> {}

impl<T> Default for MpmcList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> MpmcList<T> {
    /// 建立空佇列(一個 dummy,head=tail=origin=它)。
    pub fn new() -> Self {
        let dummy = Box::into_raw(Box::new(Node {
            next: AtomicPtr::new(ptr::null_mut()),
            val: sync::UnsafeCell::new(MaybeUninit::uninit()),
        }));
        Self {
            head: AtomicPtr::new(dummy),
            tail: AtomicPtr::new(dummy),
            origin: dummy,
        }
    }

    /// lock-free push(正式定義的 lock-free:CAS 輸了代表**別人成功了**,
    /// 系統整體必有進度——對照 Vyukov 的縫:那裡輸家可能在等一個
    /// 睡死的贏家)。
    pub fn push(&self, item: T) {
        let node = Box::into_raw(Box::new(Node {
            next: AtomicPtr::new(ptr::null_mut()),
            val: sync::UnsafeCell::new(MaybeUninit::new(item)),
        }));
        loop {
            // Acquire:tail 由別人的 Release CAS 發布,拿到才能安全 deref。
            let t = self.tail.load(Ordering::Acquire);
            // SAFETY:t 是鏈上節點且 Drop 前不釋放 ⇒ 指標有效。
            let next = unsafe { (*t).next.load(Ordering::Acquire) };
            if !next.is_null() {
                // tail 落後(別人接了鏈還沒推 tail):幫它推——M-S 的招牌
                // 「help」。失敗無所謂,代表又有別人幫過了。
                let _ =
                    self.tail
                        .compare_exchange_weak(t, next, Ordering::Release, Ordering::Relaxed);
                continue;
            }
            // 接鏈:這一個 CAS 同時是佔位(搶到唯一的 null next)與發布
            // (consumer 沿 next 立刻走得到)。Release:node 的 val 寫入
            // happens-before 任何 Acquire 讀到它的人。
            // SAFETY:同上,t 有效;CAS 目標是 t 的 next 欄位。
            if unsafe {
                (*t).next.compare_exchange(
                    ptr::null_mut(),
                    node,
                    Ordering::Release,
                    Ordering::Relaxed,
                )
            }
            .is_ok()
            {
                // 把 tail 推上來。失敗 = 有人幫忙推了,不用管。
                let _ = self
                    .tail
                    .compare_exchange(t, node, Ordering::Release, Ordering::Relaxed);
                return;
            }
            // CAS 輸了:別人接上了。回圈重讀(下一圈會走 help 分支)。
        }
    }

    /// lock-free pop。空時 None——而且 None 就是真的空:
    /// M-S 沒有 Inconsistent 態(佔位=發布合一),對照 mpsc_list 的縫。
    pub fn try_pop(&self) -> Option<T> {
        loop {
            // Acquire:安全 deref head 所指節點。
            let h = self.head.load(Ordering::Acquire);
            // SAFETY:h 在鏈上、Drop 前不釋放。
            // Acquire 配對 push 接鏈的 Release:看到非 null ⇒ val 可見。
            let next = unsafe { (*h).next.load(Ordering::Acquire) };
            if next.is_null() {
                return None; // 真空(dummy 之後沒有已發布節點)
            }
            // 先「偷看」值(bitwise copy)。此刻可能有多個 popper 同看同一個
            // next——都是唯讀,合法;唯一的擁有權在下面的 CAS 決定。
            // SAFETY(with 唯讀):push 發布後不再寫 val;讀已由 Acquire 同步。
            let peeked = unsafe { (*next).val.with(|p| ptr::read(p)) };
            // 贏了 CAS = 擁有 peeked、退休舊 dummy h(不釋放,Drop 收)。
            // AcqRel:Release 發布「h 已退休」;Acquire 折疊進後續讀。
            if self
                .head
                .compare_exchange(h, next, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                // SAFETY:CAS 贏家唯一,val 的所有權歸我們;next 成為新 dummy,
                // 它的 val 從此「邏輯已空」(Drop 靠 head 位置知道這件事)。
                return Some(unsafe { peeked.assume_init() });
            }
            // 輸了:丟掉手上的 bitwise copy(MaybeUninit 不會 drop 內容,
            // 不會 double-drop),重試。
        }
    }
}

impl<T> Drop for MpmcList<T> {
    /// 已無並發(&mut self)。從 origin 沿鏈走:
    /// 段 1(origin..=head):dummy 與已消費節點,val 已邏輯搬走,只回收 Box;
    /// 段 2(head 之後):未消費元素,val 活著,先 drop 值再回收 Box。
    fn drop(&mut self) {
        let head = self.head.load(Ordering::Relaxed);
        let mut cur = self.origin;
        // 段 1
        loop {
            // SAFETY:cur 在鏈上且尚未釋放;本迴圈每個節點只經過一次。
            let next = unsafe { (*cur).next.load(Ordering::Relaxed) };
            let at_head = ptr::eq(cur, head);
            drop(unsafe { Box::from_raw(cur) });
            cur = next;
            if at_head {
                break;
            }
        }
        // 段 2
        while !cur.is_null() {
            // SAFETY:head 之後的節點 val 必為已初始化且未被取走。
            let next = unsafe { (*cur).next.load(Ordering::Relaxed) };
            unsafe {
                (*cur).val.with_mut(|p| (*p).assume_init_drop());
                drop(Box::from_raw(cur));
            }
            cur = next;
        }
    }
}
