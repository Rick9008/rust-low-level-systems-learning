//! # loom 到底在做什麼:壓力測試 vs model checking,同一個 bug
//!
//! ```sh
//! cargo run -p reference --example loom_vs_stress
//! ```
//!
//! 三個回合,同一份**故意寫壞**的 SPSC(acquire/release 全降級成 Relaxed):
//!
//! | 回合 | 驗證方式 | 預期 |
//! |---|---|---|
//! | 1 | 真 OS thread,數十萬次 push/pop | **綠的**——什麼都沒抓到 |
//! | 2 | loom,同一份原始碼 | **紅的**,毫秒級 |
//! | 3 | loom,`reference` 出貨的正確版原始碼 | 綠的 |
//!
//! 回合 1 綠、回合 2 紅——這個落差就是 loom 存在的全部理由。
//!
//! ## 為什麼壓力測試抓不到
//! bug 是「`Relaxed` 不建立 happens-before」。但你的 CPU 是 x86-64,
//! 而 x86 是 **TSO**:硬體本來就不重排 store-store、也不重排 load-load。
//! `Relaxed` 編出來跟 `Release` 是同一條 `mov`。
//! 所以這個 bug 在 x86 上**根本沒有物理表現**——跑 10⁹ 次也是綠的,
//! 直到有人拿去 ARM / RISC-V(弱記憶體序)跑,或是編譯器某次升級決定重排它。
//!
//! random fuzz 的搜尋空間是「這台機器實際會發生的 interleaving」;
//! bug 藏在「C11 **允許**、但這台機器不會做」的那一區。fuzz 永遠掃不到那裡。
//!
//! ## loom 怎麼做到的(四件事)
//!
//! **1. 型別替換。** `loom::sync::atomic::AtomicUsize`、`loom::cell::UnsafeCell`、
//!    `loom::sync::Arc` 的 API 跟 std 一模一樣,但它們是**假的**:每一次
//!    load/store/UnsafeCell 存取都會回報給 loom 的執行期。
//!    這就是 `sync_shim.rs` 必須存在的理由——被測程式碼不能直接 `use std::sync`。
//!
//! **2. thread 不是 OS thread。** `loom::thread::spawn` 開的是 green thread
//!    (loom 依賴 `generator` crate)。任一時刻**只有一條在跑**,由 loom 排程。
//!    所以執行是決定性的:給定一個排程,結果每次都一樣——bug 可重現。
//!
//! **3. 窮舉 + 回溯。** `loom::model(f)` 不是把 `f` 跑一次,是把 `f`
//!    **跑幾百幾千次**:每次在某個決策點(atomic 存取、鎖、yield)走一條
//!    沒走過的分支,DFS 遍歷整棵排程樹。獨立的操作(不同位址、兩個 load)
//!    可交換,partial-order reduction 把這種對稱分支剪掉,否則是階乘爆炸。
//!
//! **4. 模擬 C11 記憶體模型——這才是關鍵。** loom 的 atomic 變數存的不是
//!    「一個值」,是**一整段寫入歷史 + happens-before 的因果關係**。
//!    一次 `Relaxed` load,loom 會依 C11 規則算出「哪些舊值是合法可見的」,
//!    然後**真的把過期的值回給你**。x86 硬體不會這樣做,loom 會。
//!    同理,它記錄每個 `UnsafeCell` 的存取,兩次存取之間若沒有 happens-before
//!    邊、且至少一次是寫 → 直接判定 data race,當場 panic。
//!
//! 所以 loom 通過 = 在該模型與 bound 內**證明**沒有這類 bug,
//! 不是「跑很多次沒炸」。代價:狀態空間指數成長,模型必須小
//! (2 條 thread、2-3 個操作)。model 跑超過十幾秒 = 模型開太大了。
//!
//! ## 誠實的邊界
//! loom 驗的是**它模擬的那個模型**:C11 的一個子集、preemption bound 之內
//! (`LOOM_MAX_PREEMPTIONS`)、以及你**真的寫進 model 裡的那些操作**。
//! 它不會幫你檢查沒被 model 覆蓋的 API、不模擬編譯器優化、不管 UB 以外的邏輯錯。
//! loom 綠燈不等於程式沒 bug,等於「這段演算法的這組操作,在 C11 下沒有
//! interleaving/可見性層級的錯誤」。這個範圍很窄——但窄得非常值錢。

// 同一份原始碼被 include 兩次(std 一次、loom 一次)是本 example 的全部重點,
// 不是複製貼上的意外——clippy 看到的「重複」正是 sync_shim.rs 的那個機關。
#![allow(clippy::duplicate_mod)]

use std::panic::{self, AssertUnwindSafe};
use std::time::Instant;

// ─────────────────────────────────────────────────────────────────────
// 三個 flavor,兩份原始碼。#[path] include 讓「同一份演算法」用兩套
// 同步原語各實例化一次——這正是 lib 裡 sync_shim.rs 的機關。
// ─────────────────────────────────────────────────────────────────────

/// crate root 的 shim:`spsc_ring/core_impl.rs` 寫的是 `use crate::sync_shim`,
/// 在這個 example crate 裡,`crate::sync_shim` 就是這裡——接上 loom 型別。
mod sync_shim {
    pub(crate) use loom::cell::UnsafeCell;
    pub(crate) use loom::sync::Arc;
    pub(crate) use loom::sync::atomic;
}

/// 回合 3 用的**正確版**:直接 include lib 出貨的那份原始碼,一字不改。
#[allow(dead_code)]
#[path = "../src/concurrency/spsc_ring/core_impl.rs"]
mod good_spsc;

/// 回合 1:壞版 × std 型別 → 真的能開 OS thread 壓它。
///
/// `#[path = "shared"]` 把這個 inline module 的目錄脈絡指到 `examples/shared/`,
/// 底下的 `#[path = "broken_spsc.rs"]` 才找得到那份共用原始碼。
#[path = "shared"]
mod real {
    pub(crate) mod sync {
        pub(crate) use std::sync::Arc;
        pub(crate) use std::sync::atomic;

        /// 手工複製 loom 的閉包式 UnsafeCell API,讓同一份 core 編得過。
        /// std 版沒有任何記帳——零開銷,也零偵測能力。
        pub(crate) struct UnsafeCell<T>(std::cell::UnsafeCell<T>);

        impl<T> UnsafeCell<T> {
            pub(crate) fn new(v: T) -> Self {
                Self(std::cell::UnsafeCell::new(v))
            }
            pub(crate) fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
                f(self.0.get())
            }
            pub(crate) fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
                f(self.0.get())
            }
        }
    }

    #[path = "broken_spsc.rs"]
    pub(crate) mod broken;
}

/// 回合 2:壞版 × loom 型別 → 同一份 core,換一套記憶體模型。
#[path = "shared"]
mod modeled {
    pub(crate) mod sync {
        pub(crate) use loom::cell::UnsafeCell;
        pub(crate) use loom::sync::Arc;
        pub(crate) use loom::sync::atomic;
    }

    #[path = "broken_spsc.rs"]
    pub(crate) mod broken;
}

// ─────────────────────────────────────────────────────────────────────

const STRESS_ROUNDS: usize = 1_000;
const ITEMS_PER_ROUND: u64 = 2_000;

fn main() {
    println!("同一份寫壞的 SPSC(acquire/release → Relaxed),三種驗證方式。\n");

    round1_stress();
    round2_loom_broken();
    round3_loom_good();

    println!("\n══════════════════════════════════════════════════════════════");
    println!("結論:壓力測試搜的是「這台 x86 實際會發生的 interleaving」;");
    println!("      bug 藏在「C11 允許、但 x86 不會做」的那一區,fuzz 掃不到。");
    println!("      loom 直接在 C11 模型上窮舉,所以幾毫秒就抓到。");
    println!("══════════════════════════════════════════════════════════════");
}

/// 回合 1:真 OS thread,幾十萬次 push/pop。預期:綠的(什麼都抓不到)。
fn round1_stress() {
    use real::broken::channel;

    header(1, "壓力測試:真 OS thread × std atomic");
    let total = STRESS_ROUNDS as u64 * ITEMS_PER_ROUND;
    println!("  {STRESS_ROUNDS} 回合 × {ITEMS_PER_ROUND} 個元素 = {total} 次 push/pop");

    let t = Instant::now();
    let mut anomalies = 0u64;

    for _ in 0..STRESS_ROUNDS {
        let (mut tx, mut rx) = channel(2);

        let producer = std::thread::spawn(move || {
            for i in 0..ITEMS_PER_ROUND {
                let mut item = i;
                while let Err(back) = tx.push(item) {
                    item = back;
                    std::hint::spin_loop();
                }
            }
        });

        // consumer 驗 FIFO:第 i 個 pop 出來的必須是 i
        for expect in 0..ITEMS_PER_ROUND {
            loop {
                match rx.pop() {
                    Some(v) => {
                        if v != expect {
                            anomalies += 1;
                        }
                        break;
                    }
                    None => std::hint::spin_loop(),
                }
            }
        }
        producer.join().expect("producer panicked");
    }

    let dt = t.elapsed();
    if anomalies == 0 {
        println!("  → 通過。{total} 次操作,0 個異常({dt:.2?})");
        println!("    壓力測試說:這段程式碼沒問題。它錯了。");
    } else {
        // 誠實面對:release + 某些 codegen 下,壓力測試偶爾真的會抓到。
        println!("  → 抓到 {anomalies} 個異常({dt:.2?})");
        println!("    這次運氣好。重跑幾次,你會看到它多常是綠的——這才是問題所在。");
    }
}

/// 回合 2:同一份壞掉的原始碼,換 loom 型別。預期:紅的。
fn round2_loom_broken() {
    use modeled::broken::channel;

    header(2, "model checking:loom × 同一份原始碼");
    println!("  2 條 thread、容量 1、傳 2 個元素——刻意做到最小");

    let t = Instant::now();
    let outcome = run_model_quietly(|| {
        loom::model(|| {
            let (mut tx, mut rx) = channel(1);
            let producer = loom::thread::spawn(move || {
                for i in 0..2u32 {
                    let mut item = i;
                    while let Err(back) = tx.push(item) {
                        item = back;
                        loom::thread::yield_now();
                    }
                }
            });
            for expect in 0..2u32 {
                loop {
                    match rx.pop() {
                        Some(v) => {
                            assert_eq!(v, expect, "FIFO 壞了");
                            break;
                        }
                        None => loom::thread::yield_now(),
                    }
                }
            }
            producer.join().unwrap();
        });
    });
    let dt = t.elapsed();

    match outcome {
        Err(msg) => {
            println!("  → loom 抓到了({dt:.2?}):\n");
            for line in msg.lines().take(12) {
                println!("      {line}");
            }
            println!("\n    loom 排出了一組 C11 合法、但 x86 硬體不會產生的執行——");
            println!("    consumer 讀到 tail 前進,卻讀不到 producer 寫進槽位的值。");
        }
        Ok(()) => {
            println!("  → loom 沒抓到({dt:.2?})——這出乎意料,請檢查 broken_spsc.rs");
        }
    }
}

/// 回合 3:lib 出貨的正確版(`#[path]` include 同一份原始碼)。預期:綠的。
fn round3_loom_good() {
    use good_spsc::channel;

    header(3, "model checking:loom × reference 出貨的正確版");
    println!("  同樣的 model,但演算法是 src/spsc_ring/core_impl.rs 本尊");

    let t = Instant::now();
    let outcome = run_model_quietly(|| {
        loom::model(|| {
            let (mut tx, mut rx) = channel(1);
            let producer = loom::thread::spawn(move || {
                for i in 0..2u32 {
                    let mut item = i;
                    while let Err(back) = tx.push(item) {
                        item = back;
                        loom::thread::yield_now();
                    }
                }
            });
            for expect in 0..2u32 {
                loop {
                    match rx.pop() {
                        Some(v) => {
                            assert_eq!(v, expect, "FIFO 壞了");
                            break;
                        }
                        None => loom::thread::yield_now(),
                    }
                }
            }
            producer.join().unwrap();
        });
    });
    let dt = t.elapsed();

    match outcome {
        Ok(()) => {
            println!("  → 通過({dt:.2?})。所有 interleaving、所有 C11 允許的可見性結果。");
            println!("    這不是「跑很多次沒炸」,是在模型內證明沒有這類 bug。");
        }
        Err(msg) => {
            println!("  → loom 竟然抓到了 —— reference 有 bug:\n{msg}");
        }
    }
}

// ─────────────────────────────────────────────────────────────────────

/// 跑 model 並攔下 panic(loom 用 panic 回報違規);順便靜音 panic hook,
/// 免得 backtrace 把畫面洗掉。
fn run_model_quietly(f: impl FnOnce()) -> Result<(), String> {
    let prev = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result = panic::catch_unwind(AssertUnwindSafe(f));
    panic::set_hook(prev);

    result.map_err(|e| {
        if let Some(s) = e.downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = e.downcast_ref::<String>() {
            s.clone()
        } else {
            "(non-string panic payload)".to_string()
        }
    })
}

fn header(n: u32, title: &str) {
    println!("\n─── 回合 {n}:{title} ───");
}
