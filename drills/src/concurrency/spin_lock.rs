//! drill:spin_lock —— 填 TTAS 迴圈、try_lock、DerefMut、Drop(RAII guard 首寫)。
//!
//! 已給:結構、`unsafe impl Sync`(SAFETY 已寫,讀懂它)、guard 構造 helper、
//! `Deref`(當 worked example——SAFETY 註解的寫法照著學)、免鎖雙路徑。
//! 要填:`lock` / `try_lock` / `DerefMut` / `Drop`——四個都要能說出 Ordering 理由。
//!
//! 填之前紙上回答:
//! 1. 為什麼 bound 是 `T: Send` 而不是 `T: Sync`?兩個 auto trait 各保證什麼?
//! 2. `lock` 的 Acquire 配對誰的 Release?這條邊讓「誰的哪些寫入」對「誰」可見?
//! 3. TTAS 外圈 swap、內圈純 load——為什麼一貴一便宜?(提示:MESI、cache line 獨占)
//! 4. guard 為什麼不可以 `Send`?A 執行緒鎖、B 執行緒解,哪裡壞掉?
//! 5. 臨界區 panic 時這把鎖發生什麼事?std `Mutex` 為什麼選了另一條路(poisoning)?

use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;

pub struct SpinLock<T> {
    /// false = 沒人持有。true = 有人在臨界區內。
    locked: AtomicBool,
    /// 內部可變性:lock() 拿 `&self` 卻要發出 `&mut T`,
    /// 跨執行緒的互斥由 `locked` 協定在執行期扛。
    data: UnsafeCell<T>,
}

// SAFETY(已給,讀懂再往下):跨執行緒共享 `&SpinLock<T>` 安全,因為 lock 協定
// 保證任一時刻至多一個 guard(= 至多一個 `&mut T`)。bound 是 `T: Send` 而非
// `T: Sync`——T 不會被兩執行緒同時觸碰,需要的只是「獨占存取可移到別的執行緒」。
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    /// helper(已給):構造 guard。只在「本執行緒剛把 locked 從 false 翻成 true」後呼叫。
    fn guard(&self) -> SpinGuard<'_, T> {
        SpinGuard {
            lock: self,
            _not_send: PhantomData,
        }
    }

    /// spec:TTAS(test-and-test-and-set)拿鎖。
    /// 1. 外圈 `swap(true, Ordering::?)`——舊值 false = 搶到。哪個 Ordering?配對誰?
    /// 2. 搶不到進內圈:純 `load(Ordering::Relaxed)` 等它變 false,
    ///    每圈配 `std::hint::spin_loop()`——為什麼內圈不繼續用 swap 硬撞?
    /// 3. 內圈看到 false 只代表「剛剛」沒鎖——回外圈重新 swap(可能又被搶走)。
    #[must_use = "guard 一落地鎖就放了——`let _g = lock.lock();` 才會護住臨界區"]
    pub fn lock(&self) -> SpinGuard<'_, T> {
        todo!("spec: swap-Acquire 外圈 + Relaxed load 內圈 + spin_loop hint")
    }

    /// spec:試拿一次,不等;佔用中回 None。
    /// 用 `compare_exchange(false, true, ?, ?)`——成功側/失敗側各什麼 Ordering?
    /// 失敗側為什麼可以 Relaxed?(沒拿到鎖的人有臨界區嗎?)
    #[must_use]
    pub fn try_lock(&self) -> Option<SpinGuard<'_, T>> {
        todo!("spec: 一次 CAS,成功 -> Some(self.guard())")
    }

    /// 已給:`&mut self` 已向借用檢查器證明獨占——不需要碰 atomic。
    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }

    /// 已給:所有權即獨占證明,同樣免鎖。
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

/// RAII guard:活著 = 持鎖;離開作用域(或 panic unwind)= 放鎖。
pub struct SpinGuard<'a, T> {
    lock: &'a SpinLock<T>,
    /// `*mut ()` 不是 Send:guard 被釘在拿鎖的那條執行緒上。
    _not_send: PhantomData<*mut ()>,
}

impl<T> Deref for SpinGuard<'_, T> {
    type Target = T;
    /// worked example——DerefMut 的 SAFETY 照這個寫法自己寫一遍。
    fn deref(&self) -> &T {
        // SAFETY:guard 存在 ⇒ 本執行緒持鎖 ⇒ 協定保證沒有其他 &mut T;
        // 回傳的 &T 綁在 &self 上,放鎖後借用即失效。
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinGuard<'_, T> {
    /// spec:回 `&mut T`。unsafe 一行 + **SAFETY 註解自己寫**——
    /// 憑什麼可以發出 `&mut`?guard 存在保證了什麼?`&mut self` 又保證了什麼?
    fn deref_mut(&mut self) -> &mut T {
        todo!("spec: unsafe {{ &mut *… }} + 自己寫 SAFETY 理由")
    }
}

impl<T> Drop for SpinGuard<'_, T> {
    /// spec:放鎖,一行 store。哪個 Ordering?
    /// 說得出「用 Relaxed 的話,下一個持鎖者會讀到什麼」才算會。
    fn drop(&mut self) {
        todo!("spec: store(false, ?) —— 臨界區的寫入要推到這個 store 之前")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// 手 trace:new(0) → lock(swap 得 false)→ *g+=41 → 持鎖中 try_lock=None
    /// → drop(store false, Release)→ try_lock=Some,讀到 41。
    #[test]
    #[ignore = "drill:填完 lock/try_lock/DerefMut/Drop 後移除"]
    fn single_thread_roundtrip() {
        let lock = SpinLock::new(0_u64);
        let mut g = lock.lock();
        *g += 41;
        assert!(lock.try_lock().is_none(), "持鎖中必須拿不到");
        drop(g);
        let g2 = lock.try_lock().expect("放鎖後必須拿得到");
        assert_eq!(*g2, 41);
    }

    /// 互斥本體:2 執行緒 × 100k 遞增,一次不少。
    #[test]
    #[ignore = "drill:填完後移除"]
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

    /// 臨界區 panic → unwind 跑 Drop → 鎖不壞死;資料停在 panic 前(不毒化)。
    #[test]
    #[ignore = "drill:填完後移除"]
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
        assert_eq!(*g, 1);
    }
}
