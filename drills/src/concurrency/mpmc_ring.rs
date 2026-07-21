//! drill:mpmc_ring —— 填 Vyukov MPMC 的三態判斷與 CAS 取號。
//!
//! 已給:結構(per-slot seq + 自由跑計數器 + power-of-2 mask)、建構
//! (含 seq 初始化與 cap≥2 下限)、槽位存取 helper(unsafe 已包好)。
//! 要填:`try_push` / `try_pop`——重點不在 CAS 語法,在**三態判斷**與
//! 「哪些同步掛在 seq 上、哪些不用」。
//!
//! 填之前紙上回答:
//! 1. SPSC 的「先寫槽、再 Release store tail」到多 producer 為什麼直接壞掉?
//! 2. CAS 取號為什麼可以 Relaxed?happens-before 走哪條邊?
//! 3. `seq - pos` 的三態各代表什麼?為什麼 cap=1 會讓其中兩態撞在一起?
//! 4. pop 回 None 時,佇列一定是空的嗎?(誠實邊界——lockless ≠ lock-free)
//!
//! 填完後跑 `cargo test -p reference --test loom_mpmc` 感受窮舉;
//! 這裡的煙霧測試只是「跑很多次沒炸」等級。

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};

#[repr(align(64))] // head/tail 各佔一條 cache line;MPMC 下 tail 本來就會 ping-pong,別再擠
struct CachePadded<T>(T);

/// 槽位 = 資料 + 自己的發布訊號。對邏輯位置 pos:
/// seq==pos(輪空可搶)/ pos+1(已發布可讀)/ pos+cap(已釋放,等下一圈)。
struct Slot<T> {
    seq: AtomicUsize,
    val: UnsafeCell<MaybeUninit<T>>,
}

pub struct MpmcRing<T> {
    buf: Box<[Slot<T>]>,
    mask: usize,
    cap: usize,
    tail: CachePadded<AtomicUsize>, // producer 取號機(多寫者:只能 CAS)
    head: CachePadded<AtomicUsize>, // consumer 取號機(同上)
}

// SAFETY:槽位互斥由 seq 協定保證(搶到號 + seq 對值才碰資料);
// 資料可見性走 seq 的 Release→Acquire。T: Send 因元素跨執行緒移動。
unsafe impl<T: Send> Send for MpmcRing<T> {}
unsafe impl<T: Send> Sync for MpmcRing<T> {}

impl<T> MpmcRing<T> {
    /// 建構(已給):cap 上取 2 的冪且**至少 2**——cap=1 時「已發布」
    /// (pos+1)與「下一圈輪空」(pos+cap)同值,三態塌縮會覆寫未消費資料。
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0);
        let cap = cap.next_power_of_two().max(2);
        let buf: Box<[Slot<T>]> = (0..cap)
            .map(|i| Slot {
                seq: AtomicUsize::new(i), // 第一圈:槽位 i 期望 pos=i
                val: UnsafeCell::new(MaybeUninit::uninit()),
            })
            .collect();
        Self {
            buf,
            mask: cap - 1,
            cap,
            tail: CachePadded(AtomicUsize::new(0)),
            head: CachePadded(AtomicUsize::new(0)),
        }
    }

    /// helper(已給):把 item 寫進槽位 `pos & mask`。
    /// SAFETY 前提(caller 保證):已 CAS 搶到 pos 且讀到 seq==pos。
    fn slot_write(&self, pos: usize, item: T) {
        unsafe {
            (*self.buf[pos & self.mask].val.get()).write(item);
        }
    }

    /// helper(已給):把槽位 `pos & mask` 的值 move 出來。
    /// SAFETY 前提(caller 保證):已 CAS 搶到 pos 且讀到 seq==pos+1。
    fn slot_take(&self, pos: usize) -> T {
        unsafe { (*self.buf[pos & self.mask].val.get()).assume_init_read() }
    }

    /// spec:無鎖 push,滿時 Err(item) 歸還。迴圈:
    /// 1. load tail(Ordering?——它只是取號機)
    /// 2. load 槽位 seq(Ordering?——配對誰的 Release?)
    /// 3. dif = seq.wrapping_sub(tail) as isize,三態分支:
    ///    - dif == 0:CAS tail→tail+1(Ordering?為什麼夠?)
    ///      成功 → slot_write → seq.store(tail+1, Ordering?) → Ok
    ///      失敗 → 拿 CAS 回傳的新值重試
    ///    - dif < 0:滿 → Err(item)
    ///    - dif > 0:被別人搶走 → 重讀 tail
    pub fn try_push(&self, item: T) -> Result<(), T> {
        let _ = item;
        todo!("spec: 三態判斷 + CAS 取號(Relaxed)+ seq 發布(Release)")
    }

    /// spec:無鎖 pop,空時 None。對稱於 try_push:
    /// 期望值差一(seq==head+1 才可讀)、釋放時 seq 跳 head+cap。
    /// 想清楚:None 的語意是「沒有**已發布**的元素」,不是「佇列空」。
    pub fn try_pop(&self) -> Option<T> {
        todo!("spec: dif = seq - (head+1);讀完 seq.store(head+cap, Release)")
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }
}

impl<T> Drop for MpmcRing<T> {
    fn drop(&mut self) {
        // 已無並發:[head, tail) 全是已發布元素,排空即回收。
        while self.try_pop().is_some() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// boundary:滿/空/歸還 + FIFO + cap 下限(手 trace 見 reference)。
    #[test]
    #[ignore = "drill:mpmc_ring 未填"]
    fn full_empty_fifo_and_cap_floor() {
        let q = MpmcRing::new(1);
        assert_eq!(q.capacity(), 2, "cap=1 必須上調到 2(三態塌縮)");
        q.try_push(7).unwrap();
        q.try_push(8).unwrap();
        assert_eq!(q.try_push(9), Err(9));
        assert_eq!(q.try_pop(), Some(7));
        assert_eq!(q.try_pop(), Some(8));
        assert_eq!(q.try_pop(), None);
        q.try_push(9).unwrap(); // 第二圈重用
        assert_eq!(q.try_pop(), Some(9));
    }

    /// boundary:多輪 wrap——交替次數遠超容量,mask 圈數算術不亂。
    #[test]
    #[ignore = "drill:mpmc_ring 未填"]
    fn wrap_many_rounds() {
        let q = MpmcRing::new(2);
        for i in 0..1000 {
            q.try_push(i).unwrap();
            assert_eq!(q.try_pop(), Some(i));
        }
    }

    /// 2P2C 煙霧測試:不丟不重 + 每個 producer 各自保序。
    /// (真正的證明是 reference 的 loom_mpmc——這裡只是 sanity。)
    #[test]
    #[ignore = "drill:mpmc_ring 未填"]
    fn two_producers_two_consumers_smoke() {
        const PER: u64 = 20_000;
        let q = Arc::new(MpmcRing::new(8));
        let done = Arc::new(AtomicUsize::new(0));
        let mut producers = Vec::new();
        for pid in 0..2u64 {
            let q = Arc::clone(&q);
            producers.push(thread::spawn(move || {
                for i in 0..PER {
                    let mut item = (pid << 32) | i;
                    while let Err(back) = q.try_push(item) {
                        item = back;
                        thread::yield_now();
                    }
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
