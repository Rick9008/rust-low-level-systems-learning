//! Work-stealing deque(Chase–Lev,固定容量、值裝箱教學版)核心演算法。
//!
//! 這個檔案被兩個地方 include:
//! 1. `ws_deque/mod.rs`(lib 本體,sync = std)
//! 2. `tests/loom_ws_deque.rs`(loom 驗證,sync = loom)
//!
//! 所以這裡只准用 `crate::sync_shim` 提供的同步原語,不直接碰 std::sync。
//!
//! **教學版的兩個簡化(誠實聲明)**:
//! 1. 固定容量(滿了 Err)——工業版(crossbeam-deque)容量會長,
//!    舊 buffer 的回收靠 epoch。
//! 2. **值裝箱**(槽位是 `AtomicPtr<T>`)——教科書版把值 inline 存在槽裡,
//!    steal「先讀值、輸了 CAS 再丟」的那筆讀,在「同一槽位跨圈重寫」的
//!    極端交錯下與 owner 的寫**構成正式的資料競爭**(Lê et al. 論文承認、
//!    crossbeam 靠 epoch + 特殊讀法處理)。裝箱讓那筆讀變成原子指標 load,
//!    UB 消失,loom 驗得動——代價是每 push 一次配置。

use crate::sync_shim as sync;
use std::ptr;
use sync::atomic::{AtomicPtr, AtomicUsize, Ordering, fence};

/// steal 的三值結果(與 crossbeam-deque 同形)。
/// `Retry` ≠ `Empty`:輸了決勝就必須再來,當作空會漏工作。
#[derive(Debug, PartialEq, Eq)]
pub enum Steal<T> {
    Item(T),
    Empty,
    /// 與別的 stealer(或 owner 的最後一件決鬥)撞了:重試。
    Retry,
}

/// Chase–Lev deque 本體。owner 在 bottom 端 LIFO push/pop,
/// stealers 在 top 端 FIFO steal。
pub struct WsDeque<T> {
    /// 槽位存 `Box<T>` 的裸指標;指標的原子 load/store 讓「偷看」合法。
    buf: Box<[AtomicPtr<T>]>,
    mask: usize,
    cap: usize,
    /// owner 專屬端:只有 owner 寫(store,無 CAS)。自由跑計數器。
    bottom: AtomicUsize,
    /// steal 端:stealers(與最後一件決鬥的 owner)CAS 推進。
    top: AtomicUsize,
}

// SAFETY:
// - bottom 單寫者(Owner 不是 Clone、方法拿 &mut self);top 走 CAS。
// - 槽位是 AtomicPtr:所有讀寫都是原子的,「偷看到輸掉的指標」只會被
//   丟棄(唯一 CAS 贏家才 Box::from_raw),不會 double-free。
// - 值的可見性:owner 先 store 槽位、再 Release store bottom;
//   stealer Acquire load bottom 之後才讀槽位。
// - T: Send 因為元素跨執行緒移動。
unsafe impl<T: Send> Send for WsDeque<T> {}
unsafe impl<T: Send> Sync for WsDeque<T> {}

/// owner 把手:不是 Clone——單一 owner(bottom 單寫者)由型別系統保證。
pub struct Owner<T> {
    dq: sync::Arc<WsDeque<T>>,
}

/// stealer 把手:可 Clone,多個 worker 一起偷。
pub struct Stealer<T> {
    dq: sync::Arc<WsDeque<T>>,
}

impl<T> Clone for Stealer<T> {
    fn clone(&self) -> Self {
        Self {
            dq: sync::Arc::clone(&self.dq),
        }
    }
}

/// 建立容量 `cap`(上取 2 的冪)的 work-stealing deque。
pub fn deque<T>(cap: usize) -> (Owner<T>, Stealer<T>) {
    assert!(cap > 0, "capacity must be at least 1");
    let cap = cap.next_power_of_two();
    let dq = sync::Arc::new(WsDeque {
        buf: (0..cap).map(|_| AtomicPtr::new(ptr::null_mut())).collect(),
        mask: cap - 1,
        cap,
        bottom: AtomicUsize::new(0),
        top: AtomicUsize::new(0),
    });
    (
        Owner {
            dq: sync::Arc::clone(&dq),
        },
        Stealer { dq },
    )
}

impl<T> Owner<T> {
    /// owner push(bottom 端)。滿時 Err 歸還。無 CAS——bottom 單寫者。
    pub fn push(&mut self, item: T) -> Result<(), T> {
        let dq = &*self.dq;
        // 自己的端:Relaxed(單寫者讀自己)。
        let b = dq.bottom.load(Ordering::Relaxed);
        // Acquire 看 stealers 的進度:top 推進過的槽位才可重用。
        let t = dq.top.load(Ordering::Acquire);
        if b.wrapping_sub(t) >= dq.cap {
            return Err(item); // 滿(固定容量教學版)
        }
        let p = Box::into_raw(Box::new(item));
        // 槽位 store 可 Relaxed:發布走下面 bottom 的 Release——
        // stealer 必須先 Acquire 看到新 bottom,才會來讀這個槽。
        dq.buf[b & dq.mask].store(p, Ordering::Relaxed);
        dq.bottom.store(b.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    /// owner pop(bottom 端,LIFO——剛 push 的最熱,cache 局部性是
    /// work-stealing 的核心設計理由)。
    pub fn pop(&mut self) -> Option<T> {
        let dq = &*self.dq;
        let b = dq.bottom.load(Ordering::Relaxed).wrapping_sub(1);
        // 先把 bottom 降下來「預定」這一件,再看 top。
        // store 用 Release(論文版是 Relaxed):stealer 是用 Acquire 讀 bottom
        // 來決定「槽位可讀」的——它讀到哪一筆 store,就必須看見該筆之前
        // 的所有槽位寫入。論文版讓這筆放 Relaxed、靠雙 SC fence 的整體證明
        // 補洞;教學版選擇每筆 bottom store 都 Release(x86 上免費),
        // 讓不變量局部可讀、loom 可證。loom 實測:這筆改 Relaxed 會偷到
        // 尚未可見的槽位(null)——就是被窮舉抓出來的那個交錯。
        dq.bottom.store(b, Ordering::Release);
        // SeqCst fence:與 steal 的 fence 構成 SB(store-buffering)防線。
        // 沒有它:owner 的「bottom 降了」與 stealer 的「top 升了」可以互相
        // 看不見,兩人同時把最後一件當成自己的——double-take。
        // (與 signal_pipeline 掛牌握手的 SeqCst 是同一個 litmus。)
        fence(Ordering::SeqCst);
        let t = dq.top.load(Ordering::Relaxed);
        let size = b.wrapping_sub(t) as isize;
        if size < 0 {
            // 本來就空:把 bottom 恢復原位(Release,理由同上)。
            dq.bottom.store(b.wrapping_add(1), Ordering::Release);
            return None;
        }
        // 槽位是原子指標,load 本身合法;擁有權由下面的分支決定。
        let p = dq.buf[b & dq.mask].load(Ordering::Relaxed);
        if size > 0 {
            // 不是最後一件:stealer 最遠只能搶到 t < b,無人與我們爭這一槽。
            // SAFETY:p 由 push 寫入且未被取走;本分支唯一取走者是 owner。
            return Some(*unsafe { Box::from_raw(p) });
        }
        // size == 0:最後一件,與 stealers 決鬥——誰 CAS 贏 top 誰拿走。
        let won = dq
            .top
            .compare_exchange(t, t.wrapping_add(1), Ordering::SeqCst, Ordering::Relaxed)
            .is_ok();
        // 無論輸贏,deque 已空:bottom 回到 top 之上(規範化為 t+1;
        // Release,理由同上)。
        dq.bottom.store(t.wrapping_add(1), Ordering::Release);
        if won {
            // SAFETY:CAS 贏家唯一擁有 p;輸掉的 stealer 只會丟棄它的指標副本。
            Some(*unsafe { Box::from_raw(p) })
        } else {
            None // 被偷走了:同一個 p 的所有權歸 CAS 贏的 stealer
        }
    }
}

impl<T> Stealer<T> {
    /// steal(top 端,FIFO——偷最舊的,和 owner 的熱端錯開)。
    pub fn steal(&self) -> Steal<T> {
        let dq = &*self.dq;
        // Acquire:後續的 bottom/槽位讀不能重排到它之前。
        let t = dq.top.load(Ordering::Acquire);
        // SeqCst fence:SB 防線的另一半(見 Owner::pop 的註解)。
        fence(Ordering::SeqCst);
        // Acquire 配對 owner push 的 bottom Release:看到 b ⇒ 槽位 [t, b) 的
        // 指標已寫入可見。
        let b = dq.bottom.load(Ordering::Acquire);
        if b.wrapping_sub(t) as isize <= 0 {
            return Steal::Empty; // 空(或 owner 正把最後一件收走)
        }
        // 偷看:原子 load,合法;擁有權由 CAS 決定。
        let p = dq.buf[t & dq.mask].load(Ordering::Relaxed);
        if dq
            .top
            .compare_exchange(t, t.wrapping_add(1), Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            // SAFETY:CAS 贏家唯一擁有 p(owner 的最後一件決鬥輸了會丟棄;
            // 其他 stealer 的 CAS 失敗也會丟棄)。
            Steal::Item(*unsafe { Box::from_raw(p) })
        } else {
            Steal::Retry // 輸了:副本作廢,呼叫端決定重試或先幹別的
        }
    }
}

impl<T> Drop for WsDeque<T> {
    /// 兩種把手都 drop 後才會走到這(Arc 歸零):已無並發。
    /// [top, bottom) 之間是未取走的元素,逐槽回收。
    fn drop(&mut self) {
        let mut t = self.top.load(Ordering::Relaxed);
        let b = self.bottom.load(Ordering::Relaxed);
        while t != b {
            let p = self.buf[t & self.mask].load(Ordering::Relaxed);
            // SAFETY:&mut self 獨佔;[top, bottom) 的槽位必有有效指標。
            drop(unsafe { Box::from_raw(p) });
            t = t.wrapping_add(1);
        }
    }
}
