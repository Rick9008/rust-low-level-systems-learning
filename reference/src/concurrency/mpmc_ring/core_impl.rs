//! MPMC ring(Vyukov bounded queue)核心演算法。
//!
//! 這個檔案被兩個地方 include:
//! 1. `mpmc_ring/mod.rs`(lib 本體,sync = std)
//! 2. `tests/loom_mpmc.rs`(loom 驗證,sync = loom)
//!
//! 所以這裡只准用 `crate::sync_shim` 提供的同步原語,不直接碰 std::sync。

use crate::sync_shim as sync;
use std::mem::MaybeUninit;
use sync::atomic::{AtomicUsize, Ordering};

/// head / tail 各佔一條 cache line(理由同 spsc_ring:false sharing)。
/// MPMC 下更痛:tail 這條 line 本來就要在所有 producer 核心間 ping-pong,
/// 別再讓 consumer 的 head 流量也擠進來。
#[repr(align(64))]
struct CachePadded<T>(T);

/// 一個槽位 = 資料 + 它自己的發布訊號。
///
/// SPSC 裡「bump tail」一個動作身兼佔位與發布;多 producer 後佔位(CAS 取號)
/// 必須先行,佔位和發布從此是兩個事件——`seq` 就是補上的那個發布訊號。
/// 對邏輯位置 `pos`(自由跑計數器)而言,`seq` 是三態狀態機:
///
/// - `seq == pos`      :本圈輪空,producer 可搶
/// - `seq == pos + 1`  :已發布,consumer 可讀
/// - `seq == pos + cap`:已釋放,等下一圈的 producer(它期望的 pos' = pos + cap)
///
/// happens-before 全部掛在 seq 上(store Release / load Acquire);
/// head/tail 退化成純取號機,CAS 用 Relaxed 就夠。
struct Slot<T> {
    seq: AtomicUsize,
    val: sync::UnsafeCell<MaybeUninit<T>>,
}

/// Vyukov bounded MPMC queue:任意多執行緒 push、任意多執行緒 pop。
///
/// 與 [`crate::concurrency::spsc_ring`] 同族(自由跑計數器 + power-of-2 mask),
/// 差異只有兩刀:①佔位改 CAS(index 有多個寫者)②每槽加 `seq` 當發布訊號。
/// 沒有 Producer/Consumer 把手——SPSC 靠型別系統禁止第二個寫者,
/// MPMC 的協定本來就允許任何人做任何事,把手強制不了任何不變量。
pub struct MpmcRing<T> {
    buf: Box<[Slot<T>]>,
    mask: usize,
    cap: usize,
    /// producer 取號機。多寫者:只能 CAS,不能 load+store。
    tail: CachePadded<AtomicUsize>,
    /// consumer 取號機。同上。
    head: CachePadded<AtomicUsize>,
}

// SAFETY:槽位存取的互斥由 seq 協定保證——
// producer 只在 CAS 搶到號(獨佔該 pos)且 seq==pos(consumer 已離場)後寫;
// consumer 只在 CAS 搶到號且 seq==pos+1(producer 已發布)後讀。
// 資料可見性走 seq 的 Release→Acquire 邊。T: Send 因為元素跨執行緒移動;
// 給出 Sync 是因為 push/pop 都拿 &self,共享本來就是使用方式。
unsafe impl<T: Send> Send for MpmcRing<T> {}
unsafe impl<T: Send> Sync for MpmcRing<T> {}

impl<T> MpmcRing<T> {
    /// 建立容量 `cap`(上取 2 的冪)的 MPMC ring。
    pub fn new(cap: usize) -> Self {
        Self::with_start(cap, 0)
    }

    /// 測試後門:讓 head/tail 從指定值起跑,把「計數器溢位」拉進測試。
    /// 語意與 `new` 完全相同。
    #[doc(hidden)]
    pub fn with_start(cap: usize, start: usize) -> Self {
        assert!(cap > 0, "capacity must be at least 1");
        // 下限 2,不只是上取 2 的冪:cap=1 時「已發布」(seq=pos+1)與
        // 「下一圈輪空」(seq=pos+cap=pos+1)數值相同,三態塌縮成二態,
        // producer 會把未消費的資料當空槽覆寫。原版 Vyukov 同樣要求 ≥2。
        let cap = cap.next_power_of_two().max(2);
        let mask = cap - 1;
        let buf: Box<[Slot<T>]> = (0..cap)
            .map(|_| Slot {
                seq: AtomicUsize::new(0),
                val: sync::UnsafeCell::new(MaybeUninit::uninit()),
            })
            .collect();
        // 邏輯位置 start+k 落在槽位 (start+k)&mask;該槽第一圈的「輪空」狀態
        // 是 seq == 它期望的 pos,所以逐槽把 seq 設成 start+k。
        for k in 0..cap {
            let pos = start.wrapping_add(k);
            buf[pos & mask].seq.store(pos, Ordering::Relaxed);
        }
        Self {
            buf,
            mask,
            cap,
            tail: CachePadded(AtomicUsize::new(start)),
            head: CachePadded(AtomicUsize::new(start)),
        }
    }

    /// 無鎖 push。滿時 Err 歸還元素。均攤 O(1);競爭下 CAS 失敗會重試
    /// (次數與同時搶號的 producer 數同階)。
    pub fn try_push(&self, item: T) -> Result<(), T> {
        let mut tail = self.tail.0.load(Ordering::Relaxed);
        loop {
            let slot = &self.buf[tail & self.mask];
            // Acquire 配對 consumer 釋放槽位的 store(Release):看到 seq==tail
            // 代表上一圈的讀取已完成,我們覆寫不會撕掉一個正在被讀的值。
            let seq = slot.seq.load(Ordering::Acquire);
            // 自由跑計數器的三態判斷:wrapping 差值轉 isize,|dif| ≤ cap 不會誤判。
            let dif = seq.wrapping_sub(tail) as isize;
            if dif == 0 {
                // 輪空可搶:CAS 取號。Relaxed 就夠——這只是「搶到 pos 的獨佔權」,
                // 資料同步不走這裡,全在 seq 的 Release/Acquire 上。
                // weak:允許偽失敗(ARM LL/SC 較省),反正外圈本來就是重試迴圈。
                match self.tail.0.compare_exchange_weak(
                    tail,
                    tail.wrapping_add(1),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // SAFETY(with_mut 獨佔):CAS 成功 ⇒ 本執行緒獨佔邏輯位置
                        // tail;seq==tail(上面 Acquire 讀到)⇒ consumer 已離場、
                        // 槽位是未初始化/已搬空的 MaybeUninit,write 不 drop 舊值。
                        slot.val.with_mut(|p| unsafe {
                            (*p).write(item);
                        });
                        // 發布:槽位寫入 happens-before 這個 store;
                        // consumer 以 Acquire 讀到 seq==tail+1 才會碰資料。
                        // 「佔位(上面 CAS)≠ 發布(這行)」——中間這道縫就是
                        // MPMC 比 SPSC 多出來的本質複雜度。
                        slot.seq.store(tail.wrapping_add(1), Ordering::Release);
                        return Ok(());
                    }
                    // 別人先搶走(或偽失敗):拿最新值重試,不用重新 load。
                    Err(cur) => tail = cur,
                }
            } else if dif < 0 {
                // seq 落後期望值 ⇒ 這個槽還停在上一圈(consumer 沒消化)⇒ 滿。
                return Err(item);
            } else {
                // seq 超前 ⇒ 同 pos 被別的 producer 搶走且已發布 ⇒ 重讀 tail 追上。
                tail = self.tail.0.load(Ordering::Relaxed);
            }
        }
    }

    /// 無鎖 pop。空時 None。均攤 O(1),競爭下同 try_push。
    ///
    /// 誠實邊界:回 None 只代表「此刻沒有**已發布**的元素」——可能有 producer
    /// 佔了號還沒發布(佔位與發布之間被 deschedule)。這也是 Vyukov queue
    /// 是 lockless 而非正式 lock-free 的原因:那個停滯的 producer 會擋住
    /// 後續所有 consumer 對該槽的進度。
    pub fn try_pop(&self) -> Option<T> {
        let mut head = self.head.0.load(Ordering::Relaxed);
        loop {
            let slot = &self.buf[head & self.mask];
            // Acquire 配對 producer 發布的 store(Release):看到 seq==head+1
            // 才保證槽位資料完整可見。
            let seq = slot.seq.load(Ordering::Acquire);
            let dif = seq.wrapping_sub(head.wrapping_add(1)) as isize;
            if dif == 0 {
                // 已發布可讀:CAS 取號(Relaxed,理由同 try_push)。
                match self.head.0.compare_exchange_weak(
                    head,
                    head.wrapping_add(1),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // SAFETY(with 唯讀):CAS 成功 ⇒ 獨佔邏輯位置 head;
                        // seq==head+1 ⇒ 已初始化且寫入可見。assume_init_read 把值
                        // move 出來,槽位回到「邏輯未初始化」,之後只會被下一圈
                        // producer 覆寫,不會 double-drop。
                        let item = slot.val.with(|p| unsafe { (*p).assume_init_read() });
                        // 釋放:seq 跳到 head+cap = 下一圈 producer 期望的 pos。
                        // Release 讓「讀取完成」happens-before 下一圈的覆寫。
                        slot.seq
                            .store(head.wrapping_add(self.cap), Ordering::Release);
                        return Some(item);
                    }
                    Err(cur) => head = cur,
                }
            } else if dif < 0 {
                // seq 還停在 head ⇒ 沒有已發布的元素 ⇒ 空(見函式 doc 的誠實邊界)。
                return None;
            } else {
                // 同 pos 被別的 consumer 搶走:重讀 head 追上。
                head = self.head.0.load(Ordering::Relaxed);
            }
        }
    }

    /// 容量(已上取 2 的冪)。
    pub fn capacity(&self) -> usize {
        self.cap
    }
}

impl<T> Drop for MpmcRing<T> {
    /// 走到這裡代表已無任何共享(&mut self):不會再有 in-flight 的佔位,
    /// [head, tail) 之間必然全是已發布元素,用 try_pop 排空即可
    /// (單執行緒下 CAS 必成功,零重試)。
    fn drop(&mut self) {
        while self.try_pop().is_some() {}
    }
}
