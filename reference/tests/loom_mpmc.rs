//! loom 窮舉驗證:MPMC ring(Vyukov)。
//!
//! 模型刻意小(每 producer 1 個元素、cap 2):CAS 重試迴圈讓狀態空間
//! 比 SPSC 大得多,3 條執行緒 × 2 個元素已足以打出
//! seq 發布順序寫錯、CAS 取號與槽位寫入的競爭等所有一步錯誤。
//!
//! 機關同 loom_spsc:`sync_shim` 換成 loom 型別,`#[path]` include
//! **同一份**核心演算法原始碼。

// loom 版 shim:名稱、API 與 lib 的 `crate::sync_shim` 完全對齊。
mod sync_shim {
    pub(crate) use loom::cell::UnsafeCell;
    #[allow(unused_imports)]
    pub(crate) use loom::sync::Arc;
    pub(crate) use loom::sync::atomic;
}

// 測試只用到 new/try_push/try_pop;其餘 API 是 lib 的事,不算 dead code。
#[allow(dead_code)]
#[path = "../src/concurrency/mpmc_ring/core_impl.rs"]
mod core_impl;

use core_impl::MpmcRing;
use loom::sync::Arc;

/// 2 producers × 1 元素、cap 2(不觸滿):所有交錯下 consumer 收齊
/// 兩個值、不丟不重——驗「CAS 取號 + seq 發布」的核心交接。
#[test]
fn loom_mpmc_two_producers_all_interleavings() {
    loom::model(|| {
        let q = Arc::new(MpmcRing::new(2));
        let mut handles = Vec::new();
        for v in [1u32, 2] {
            let q = Arc::clone(&q);
            handles.push(loom::thread::spawn(move || {
                q.try_push(v).unwrap(); // cap=2、各推一個,必成功
            }));
        }
        let mut got = Vec::new();
        while got.len() < 2 {
            match q.try_pop() {
                Some(v) => got.push(v),
                None => loom::thread::yield_now(),
            }
        }
        got.sort_unstable();
        assert_eq!(got, vec![1, 2]); // 不丟不重
        assert_eq!(q.try_pop(), None);
        for h in handles {
            h.join().unwrap();
        }
    });
}

/// 1 producer 推 2、2 consumers 各搶:兩人拿到的必須是不相交的分割
/// ——驗 head 端 CAS 取號不會讓同一元素被讀兩次。
#[test]
fn loom_mpmc_two_consumers_disjoint() {
    loom::model(|| {
        let q = Arc::new(MpmcRing::new(2));
        q.try_push(Box::new(1u32)).unwrap();
        q.try_push(Box::new(2u32)).unwrap();
        let mut handles = Vec::new();
        for _ in 0..2 {
            let q = Arc::clone(&q);
            handles.push(loom::thread::spawn(move || {
                loop {
                    if let Some(v) = q.try_pop() {
                        return *v;
                    }
                    loom::thread::yield_now();
                }
            }));
        }
        let a = handles.pop().unwrap().join().unwrap();
        let b = handles.pop().unwrap().join().unwrap();
        let mut got = [a, b];
        got.sort_unstable();
        assert_eq!(got, [1, 2]); // 不相交且齊全(Box 同時驗資料可見性)
    });
}

/// 滿時的 backpressure + 第二圈槽位重用(seq += cap):1 producer 推 3 個
/// 過 cap=2 的環(滿了 Err 重試),consumer 依序全收。
/// 多 producer 的競爭已由上面兩測覆蓋;這裡刻意只開 2 條執行緒——
/// 3 執行緒 × 滿載重試迴圈會讓 loom 狀態空間爆掉(模型要小的鐵律)。
#[test]
fn loom_mpmc_full_backpressure_and_lap_reuse() {
    loom::model(|| {
        let q = Arc::new(MpmcRing::new(2));
        let q2 = Arc::clone(&q);
        let producer = loom::thread::spawn(move || {
            for v in 0..3u32 {
                let mut item = v;
                while let Err(back) = q2.try_push(item) {
                    item = back;
                    loom::thread::yield_now(); // 滿:讓 loom 排 consumer
                }
            }
        });
        // 單 producer ⇒ 取號順序 = push 順序,FIFO 必須嚴格成立。
        for expect in 0..3u32 {
            loop {
                match q.try_pop() {
                    Some(v) => {
                        assert_eq!(v, expect);
                        break;
                    }
                    None => loom::thread::yield_now(),
                }
            }
        }
        assert_eq!(q.try_pop(), None);
        producer.join().unwrap();
    });
}

/// 帶著未消費元素結束:Drop 排空在所有交錯下不洩漏、不 double-drop
/// (loom 的 leak 檢查 + Box 的 double-free 會直接炸)。
#[test]
fn loom_mpmc_drop_midstream_no_leak() {
    loom::model(|| {
        let q = Arc::new(MpmcRing::new(2));
        let q2 = Arc::clone(&q);
        let producer = loom::thread::spawn(move || {
            let _ = q2.try_push(Box::new(1u32));
            let _ = q2.try_push(Box::new(2u32));
        });
        let _ = q.try_pop(); // 可能撿到 0/1 個
        producer.join().unwrap();
        // 兩個 Arc 陸續 drop → MpmcRing::drop 清殘留元素
    });
}
