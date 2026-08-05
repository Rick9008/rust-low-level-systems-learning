//! sol_logring_0805_spin —— LogRing 的 SpinLock 變體 + 選型裁決。
//!
//! ══ 先講裁決(被問「換 spin lock 呢?」的上場答法)══════════════════
//! 這題臨界區不小:AddLog 走淘汰迴圈、ShowLog 要 memcpy 近 1KB——
//! spin 的適用格是「臨界區奈秒級 + 持鎖者不會被排程走」,兩個條件這題都缺。
//! 所以分平台裁:
//! - hosted(有 OS):答案仍是 Mutex——futex 短爭用先自旋、長等待睡覺,
//!   兩頭都拿;等待者不燒 CPU,持鎖者被搶佔也不會讓別人原地空轉。
//! - 題面是 embedded:no_std 沒有 futex,std::sync::Mutex 根本不存在——
//!   spin 是「多核心、無 OS」的中間格,這個變體就是為這格寫的。
//! - 但單核 + ISR 會碰這把鎖的話,spin 直接死鎖:ISR 搶佔持鎖者後原地轉,
//!   持鎖者永遠回不來——嵌入式正解是關中斷臨界段,或把 AddLog 端改成
//!   lock-free SPSC 環(ISR 只寫、主迴圈只讀)。
//! 三格判準一句話:hosted=Mutex|多核 no_std=spin|ISR 參與=關中斷/SPSC。
//! ═══════════════════════════════════════════════════════════════════
//!
//! SpinLock 本體的三個上場點(7/31 spin_lock drill 傷疤複驗):
//! 1. lock 成功 ordering = Acquire(上鎖時無物可發佈,AcqRel 是多餘的那半);
//!    unlock = Release(把臨界區寫入發佈給下一個拿鎖的人)。
//! 2. 等待段用 TTAS:先 load(Relaxed) 讀到 false 才回去 CAS——純 TAS 每圈
//!    都是 RMW,把 cache line 打成 Modified 乒乓(mesi-rmw-atomics 頁那課)。
//! 3. 迴圈裡 CAS 用 weak(偽失敗只是多轉一圈);try_lock 才必須 strong
//!    (單發偽失敗 = 對「被佔用」作偽證)。
//!
//! 驗證:rustc --edition 2024 編過 + smoke 4/4 ✓(sol 的三段 + 雙執行緒併發段)。

use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::hint;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ───────────────────────── SpinLock ─────────────────────────

struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

// SAFETY(三段式):
// 1) 主張:T: Send 時 SpinLock<T> 可以 Sync(多執行緒共享 &SpinLock<T>)。
// 2) 理由:對 data 的所有存取都走 lock() 拿 guard;locked 的 CAS(Acquire) /
//    store(Release) 配對保證同一時刻至多一個 guard 活著,且上一段臨界區的
//    寫入 happens-before 下一段——互斥與可見性都由鎖協定扛。
// 3) 邊界:只要求 T: Send(值被「輪流獨占」存取,等同跨執行緒搬移),
//    不要求 T: Sync——永遠不會有兩個 &T 同時存在。
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    fn lock(&self) -> SpinGuard<'_, T> {
        loop {
            // 拿鎖:成功 Acquire(接收上一位 Release 的發佈)、失敗 Relaxed
            // (沒拿到就什麼都不主張);迴圈裡 weak 即可。
            if self
                .locked
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return SpinGuard { lock: self };
            }
            // TTAS 等待段:只讀不 RMW,cache line 停在 Shared 不打乒乓。
            while self.locked.load(Ordering::Relaxed) {
                hint::spin_loop();
            }
        }
    }
}

/// 注:std 的 MutexGuard 是 !Send(pthread unlock 必須同執行緒);spin 版
/// 沒這個硬約束,但要照慣例封死可以塞 PhantomData<*const ()>——面試提一句即可。
struct SpinGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for SpinGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY:guard 存在 ⇒ 本執行緒持鎖(lock() 的 CAS 成功後才建構),
        // 互斥保證沒有其他 &T / &mut T 並存。
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY:同上,且 &mut self 保證這個 guard 本身也沒有別的借用。
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinGuard<'_, T> {
    fn drop(&mut self) {
        // Release:把臨界區的寫入發佈給下一個 Acquire 成功的人。
        self.lock.locked.store(false, Ordering::Release);
    }
}

// ──────────────────── LogRing(本體同 sol 版)────────────────────

const BYTE_BUDGET: usize = 1024;

struct Entry {
    log_id: u32,
    body: Vec<u8>,
}

impl Entry {
    fn new(log_id: u32, body: &[u8]) -> Self {
        Self {
            log_id,
            body: body.to_vec(),
        }
    }

    fn wire_size(&self) -> usize {
        8 + self.body.len()
    }
}

struct Inner {
    entries: VecDeque<Entry>,
    total_bytes: usize,
}

#[derive(Clone)]
struct LogRing {
    inner: Arc<SpinLock<Inner>>,
}

impl LogRing {
    fn new() -> Self {
        Self {
            inner: Arc::new(SpinLock::new(Inner {
                entries: VecDeque::new(),
                total_bytes: 0,
            })),
        }
    }

    fn add_log(&self, entry: Entry) {
        let mut inner = self.inner.lock();
        let es = entry.wire_size();
        if es > BYTE_BUDGET {
            return;
        }
        while inner.total_bytes + es > BYTE_BUDGET {
            let dead = inner.entries.pop_front().unwrap();
            inner.total_bytes -= dead.wire_size();
        }
        inner.total_bytes += es;
        inner.entries.push_back(entry);
    }

    fn show_log(&self, dma_buffer: &mut [u8]) -> usize {
        let inner = self.inner.lock();
        let mut written = 0;
        for entry in inner.entries.iter().rev() {
            let es = entry.wire_size();
            if written + es > dma_buffer.len() {
                break;
            }
            dma_buffer[written..written + 4].copy_from_slice(&entry.log_id.to_be_bytes());
            dma_buffer[written + 4..written + 8]
                .copy_from_slice(&(entry.body.len() as u32).to_be_bytes());
            dma_buffer[written + 8..written + 8 + entry.body.len()].copy_from_slice(&entry.body);
            written += es;
        }
        written
    }
}

fn main() {
    // ── smoke 1–3:與 sol 版相同(順序+bytes / 淘汰 / 小 buffer)──
    let ring = LogRing::new();
    ring.add_log(Entry::new(1, &[0xAA, 0xBB, 0xCC, 0xDD]));
    ring.add_log(Entry::new(2, &[0x11, 0x22, 0x33, 0x44]));
    let mut buf = [0u8; 1024];
    let n = ring.show_log(&mut buf);
    assert_eq!(n, 24);
    assert_eq!(&buf[0..4], &2u32.to_be_bytes());
    assert_eq!(&buf[4..8], &4u32.to_be_bytes());
    assert_eq!(&buf[8..12], &[0x11, 0x22, 0x33, 0x44]);
    assert_eq!(&buf[12..16], &1u32.to_be_bytes());

    let ring = LogRing::new();
    for id in 10u32..14 {
        ring.add_log(Entry::new(id, &[id as u8; 256]));
    }
    let n = ring.show_log(&mut buf);
    assert_eq!(n, 3 * 264);
    assert_eq!(&buf[n - 264..n - 260], &11u32.to_be_bytes());

    let mut small = [0u8; 300];
    let n = ring.show_log(&mut small);
    assert_eq!(n, 264);
    assert_eq!(&small[0..4], &13u32.to_be_bytes());

    // ── smoke 4:併發段 ── 兩執行緒各塞 50 筆 4B body(wire=12B),
    // 100 × 12 = 1200 > 1024 → 淘汰必然觸發;等長條目下穩態是決定性的:
    // 1024 / 12 = 85 筆 → 85 × 12 = 1020 bytes,不管交錯順序怎麼走。
    // 這段同時驗了 Clone 把手跨執行緒(unsafe impl Sync 真的被用到)。
    let ring = LogRing::new();
    let mut handles = Vec::new();
    for t in 0u32..2 {
        let r = ring.clone();
        handles.push(std::thread::spawn(move || {
            for i in 0..50u32 {
                r.add_log(Entry::new(t * 1000 + i, &[0xEE; 4]));
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    let n = ring.show_log(&mut buf);
    assert_eq!(n, 85 * 12);

    println!("smoke 4/4 ✓");
}
