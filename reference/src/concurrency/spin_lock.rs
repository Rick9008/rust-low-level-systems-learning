//! # spin_lock —— TTAS 自旋鎖 + RAII guard(`Drop`/`Deref`/`unsafe impl Sync` 教學模組)
//!
//! ## [Clarify]
//! 解決:多執行緒互斥存取一個 `T`,臨界區極短(奈秒~百奈秒級)、或身處
//! **不可睡眠的 context**(ISR、已持有另一把 spinlock)——這時 `Mutex` 的
//! park/unpark(syscall,µs 級)比臨界區本身貴,忙等反而便宜。
//! Constraints:std-only、不重入、不公平(無 FIFO 保序)、不毒化。
//!
//! ## [Abstract]
//! `T` 完全泛型。等待策略只做 spin + `spin_loop` hint;退避(backoff)、
//! yield、ticket 公平性都是後續迭代,面試先 stub 掉往前走。
//!
//! ## [Iterate]
//! naive:`compare_exchange` 硬撞到成功——每次失敗都是一次 RMW,
//! 所有等待者反覆搶同一條 cache line 的獨占權(MESI),匯流排被打爆。
//! 本模組主體:**TTAS**(test-and-test-and-set)——外圈 `swap` 搶鎖,
//! 搶不到就進內圈用 **純 load** 等待:讀共享的 cache line 不需獨占,
//! 等待者們安靜地各讀各的,鎖釋放時才回外圈再搶一次。
//!
//! ## [Trade-offs]
//! - **三不**:不重入(同執行緒二次 `lock()` = 把自己旋死)、不公平(誰搶到
//!   誰贏,飢餓可能;升級路 = ticket lock)、不毒化(臨界區 panic 時 unwind
//!   照跑 `Drop` → 鎖被放開、資料可能只改了一半;std `Mutex` 選 poisoning
//!   把這件事顯式化,我們選「使用者自己保證不變量」)。
//! - guard **不可 `Send`**:在 A 執行緒上鎖、搬到 B 執行緒解鎖,會讓
//!   「臨界區屬於誰」失去意義(也踩爛 happens-before 的推理)。
//! - 對照 production:`std::sync::Mutex`(futex,會睡)、`parking_lot`
//!   (自旋幾輪再睡的混合策略)、`spin` crate。
//!
//! ## [Dry-Run]
//! 見下方測試:`single_thread_roundtrip` 逐行手 trace;boundary 涵蓋
//! try_lock 佔用/釋放、雙執行緒互斥計數、臨界區 panic 不壞死。
//!
//! 複雜度:`lock()` 無競爭 O(1)(一次 RMW);有競爭時自旋時間無上界
//! (等於前一個持鎖者的臨界區長度)——這就是「臨界區必須極短」的原因。

use std::cell::UnsafeCell;
use std::hint;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct SpinLock<T> {
    /// false = 沒人持有。true = 有人在臨界區內。
    locked: AtomicBool,
    /// 內部可變性:lock() 拿 `&self` 卻要發出 `&mut T`,
    /// 借用檢查器管不到跨執行緒的互斥,由 `locked` 協定在執行期扛。
    data: UnsafeCell<T>,
}

// SAFETY:跨執行緒共享 `&SpinLock<T>` 是安全的,因為 lock 協定保證
// 任一時刻至多一個執行緒能拿到 guard(= 至多一個 `&mut T` 存在)。
// bound 是 `T: Send` 而非 `T: Sync`:T 永遠不會被兩個執行緒**同時**觸碰,
// 需要的只是「T 的所有權/獨占存取可以移到別的執行緒上使用」= Send。
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    /// helper:構造 guard。只在「本執行緒剛把 locked 從 false 翻成 true」後呼叫。
    fn guard(&self) -> SpinGuard<'_, T> {
        SpinGuard {
            lock: self,
            _not_send: PhantomData,
        }
    }

    /// TTAS 拿鎖(忙等,無競爭 O(1);有競爭自旋無上界)。
    ///
    /// `swap(true, Acquire)` 回傳舊值:false = 這一下是我把它翻成 true 的,搶到;
    /// true = 本來就有人持有。Acquire 與 `Drop` 的 Release store 配對——
    /// 前一個持鎖者在臨界區內的所有寫入,對我 happens-before 可見。
    #[must_use = "guard 一落地鎖就放了——`let _g = lock.lock();` 才會護住臨界區"]
    pub fn lock(&self) -> SpinGuard<'_, T> {
        while self.locked.swap(true, Ordering::Acquire) {
            // 內圈:純 load 等待。讀取讓 cache line 停在 Shared 狀態,
            // 不像 RMW 每次都要搶獨占——等待者再多,匯流排也安靜。
            while self.locked.load(Ordering::Relaxed) {
                hint::spin_loop(); // 告訴 CPU「我在忙等」:省電、讓出超執行緒資源
            }
        }
        self.guard()
    }

    /// 試拿一次,不等。佔用中回 `None`。
    ///
    /// `compare_exchange` 失敗側用 Relaxed:沒拿到鎖就沒有臨界區可言,
    /// 不需要任何可見性保證。
    #[must_use]
    pub fn try_lock(&self) -> Option<SpinGuard<'_, T>> {
        if self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(self.guard())
        } else {
            None
        }
    }

    /// `&mut self` 已向借用檢查器證明獨占——不需要碰 atomic。
    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }

    /// 消耗鎖取回資料。所有權即獨占證明,同樣免鎖。
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

/// RAII guard:活著 = 持鎖;離開作用域(或 panic unwind)= 放鎖。
/// 「解鎖」這個動作因此**不可能忘記、不可能做兩次**——這就是 guard 模式的全部價值。
pub struct SpinGuard<'a, T> {
    lock: &'a SpinLock<T>,
    /// `*mut ()` 不是 Send:guard 被釘在拿鎖的那條執行緒上,
    /// 「A 鎖 B 解」直接編譯不過。
    _not_send: PhantomData<*mut ()>,
}

impl<T> Deref for SpinGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY:guard 存在 ⇒ 本執行緒持鎖 ⇒ 協定保證沒有其他 &mut T 存在;
        // 回傳的 &T 生命週期綁在 &self(進而綁在 guard)上,放鎖後借用即失效。
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY:同上,且 &mut self 保證這是 guard 上唯一一個活著的借用,
        // 所以發出唯一的 &mut T 不會與任何 &T 重疊。
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinGuard<'_, T> {
    fn drop(&mut self) {
        // Release 與 lock()/try_lock() 的 Acquire 配對:把臨界區內的所有寫入
        // 「推」到這個 store 之前——下一個拿到鎖的人保證看到完整的修改,
        // 不會讀到殘影。Relaxed 在 x86 上多半僥倖能動,在 ARM 上就是真 bug。
        self.lock.locked.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// 手 trace(逐行):
    /// 1. `new(0)`:locked=false,data=0
    /// 2. `lock()`:swap(true, Acquire) 回傳 false → 外圈不進,拿到 guard;locked=true
    /// 3. `*g += 41`:DerefMut 發出 &mut data → data=41
    /// 4. 持鎖中 `try_lock()`:compare_exchange(false→true) 失敗(現值 true)→ None
    ///    —— 同時示範「不可重入」:這裡若呼叫 lock() 就是把自己旋死
    /// 5. `drop(g)`:store(false, Release) → locked=false
    /// 6. `try_lock()`:成功;Release→Acquire 邊保證讀到 41
    #[test]
    fn single_thread_roundtrip() {
        let lock = SpinLock::new(0_u64);
        let mut g = lock.lock();
        *g += 41;
        assert!(lock.try_lock().is_none(), "持鎖中必須拿不到");
        drop(g);
        let g2 = lock.try_lock().expect("放鎖後必須拿得到");
        assert_eq!(*g2, 41);
    }

    /// boundary:get_mut / into_inner 免鎖路徑(獨占由型別系統證明)。
    #[test]
    fn exclusive_paths_skip_the_atomic() {
        let mut lock = SpinLock::new(7);
        *lock.get_mut() += 1;
        assert_eq!(lock.into_inner(), 8);
    }

    /// 互斥本體:2 執行緒 × 100k 遞增,一次不少。
    /// (`+= 1` 是 load-modify-store 三步——沒有互斥時必然丟更新。)
    #[test]
    fn two_threads_increment() {
        const N: u64 = 100_000;
        let lock = SpinLock::new(0_u64);
        thread::scope(|s| {
            for _ in 0..2 {
                s.spawn(|| {
                    for _ in 0..N {
                        *lock.lock() += 1;
                    }
                });
            }
        });
        assert_eq!(lock.into_inner(), 2 * N);
    }

    /// 不毒化的代價與收穫,一次看清:
    /// 臨界區內 panic → unwind 照跑 guard 的 Drop → 鎖被放開(不壞死),
    /// 但資料停在 panic 前的狀態(這裡 +1 已生效)——一致性由使用者自負。
    #[test]
    fn panic_releases_instead_of_poisoning() {
        let lock = Arc::new(SpinLock::new(0_u64));
        let l = Arc::clone(&lock);
        let h = thread::spawn(move || {
            let mut g = l.lock();
            *g += 1;
            panic!("boom in critical section");
        });
        assert!(h.join().is_err());
        let g = lock.try_lock().expect("unwind 必須已放鎖");
        assert_eq!(*g, 1, "panic 前的寫入留了下來——沒有 poisoning 幫你擋");
    }
}
