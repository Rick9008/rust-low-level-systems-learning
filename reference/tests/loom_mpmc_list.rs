//! loom 窮舉驗證:MPMC list(Michael–Scott,教學版)。
//!
//! 重點交錯:①兩個 producer 搶「唯一的 null next」+ tail 落後時的 help
//! ②兩個 consumer 對同一元素的 head CAS 決勝(偷看副本不得 double-drop)。
//! 教學版不回收退休節點,所以不存在 UAF 交錯——這正是它能被 loom
//! 直接驗的原因(工業版的 epoch 邏輯 loom 驗不動)。
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

// 測試只用到 new/push/try_pop;其餘 API 是 lib 的事,不算 dead code。
#[allow(dead_code)]
#[path = "../src/concurrency/mpmc_list/core_impl.rs"]
mod core_impl;

use core_impl::MpmcList;
use loom::sync::Arc;

/// 2 producers × 1 元素:接鏈 CAS 對撞 + tail help 的所有交錯下,
/// consumer 收齊兩個值、不丟不重。
#[test]
fn loom_msq_two_producers_all_interleavings() {
    loom::model(|| {
        let q = Arc::new(MpmcList::new());
        let mut handles = Vec::new();
        for v in [1u32, 2] {
            let q = Arc::clone(&q);
            handles.push(loom::thread::spawn(move || {
                q.push(v); // unbounded:必成功
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
        assert_eq!(got, vec![1, 2]);
        assert_eq!(q.try_pop(), None);
        for h in handles {
            h.join().unwrap();
        }
    });
}

/// 1 元素、2 consumers 決鬥:恰好一人拿到(偷看的 bitwise copy 在
/// CAS 輸家手上必須無害消失——Box 值讓 double-drop 直接炸)。
#[test]
fn loom_msq_two_consumers_exactly_one_wins() {
    loom::model(|| {
        let q = Arc::new(MpmcList::new());
        q.push(Box::new(7u32));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let q = Arc::clone(&q);
            handles.push(loom::thread::spawn(move || q.try_pop().map(|b| *b)));
        }
        let a = handles.pop().unwrap().join().unwrap();
        let b = handles.pop().unwrap().join().unwrap();
        // 恰一人 Some(7):兩人都拿到=重複消費;都沒拿到=元素蒸發。
        assert!(
            matches!((a, b), (Some(7), None) | (None, Some(7))),
            "duel 結果必須恰好一人贏:{a:?} / {b:?}"
        );
    });
}

/// 帶著未消費元素結束:Drop 的兩段式回收(退休鏈只收 Box、活值連值收)
/// 在所有交錯下不洩漏、不 double-drop。
#[test]
fn loom_msq_drop_midstream_no_leak() {
    loom::model(|| {
        let q = Arc::new(MpmcList::new());
        let q2 = Arc::clone(&q);
        let producer = loom::thread::spawn(move || {
            q2.push(Box::new(1u32));
            q2.push(Box::new(2u32));
        });
        let _ = q.try_pop(); // 可能撿到 0/1 個;撿到的節點退休
        producer.join().unwrap();
        // Arc 歸零 → MpmcList::drop 兩段式清場
    });
}
