//! loom 窮舉驗證:MPSC list(Vyukov)。
//!
//! 重點交錯:producer 的「swap 佔位 → store 發布」兩步之間(縫)。
//! 單執行緒測試永遠看不到 `Inconsistent`;loom 會把「producer 停在縫裡、
//! consumer 此刻來 pop」的世界線也走一遍——本檔案就是為它而寫。
//!
//! 機關同 loom_spsc:`sync_shim` 換成 loom 型別,`#[path]` include
//! **同一份**核心演算法原始碼。

// loom 版 shim:名稱、API 與 lib 的 `crate::sync_shim` 完全對齊。
mod sync_shim {
    pub(crate) use loom::cell::UnsafeCell;
    pub(crate) use loom::sync::Arc;
    pub(crate) use loom::sync::atomic;
}

// 測試只用到 channel/push/pop;其餘 API 是 lib 的事,不算 dead code。
#[allow(dead_code)]
#[path = "../src/concurrency/mpsc_list/core_impl.rs"]
mod core_impl;

use core_impl::{PopResult, channel};

/// 2 producers × 1 元素:所有交錯(含縫)下 consumer 收齊兩個值,
/// 不丟不重;Inconsistent 只是重試訊號,絕不終態。
#[test]
fn loom_mpsc_two_producers_all_interleavings() {
    loom::model(|| {
        let (tx, mut rx) = channel();
        let mut handles = Vec::new();
        for v in [1u32, 2] {
            let tx = tx.clone();
            handles.push(loom::thread::spawn(move || {
                tx.push(v);
            }));
        }
        let mut got = Vec::new();
        while got.len() < 2 {
            match rx.pop() {
                PopResult::Item(v) => got.push(v),
                // Empty 或縫:讓 loom 把 producer 排進來走完發布那一步。
                PopResult::Empty | PopResult::Inconsistent => loom::thread::yield_now(),
            }
        }
        got.sort_unstable();
        assert_eq!(got, vec![1, 2]);
        assert_eq!(rx.pop(), PopResult::Empty); // 收完必回 Empty(不是縫)
        for h in handles {
            h.join().unwrap();
        }
    });
}

/// 單 producer 推 2 個 Box:FIFO + 跨執行緒資料可見性
/// (Box 內容經 next 的 Release→Acquire 邊移交)。
#[test]
fn loom_mpsc_fifo_boxed_values() {
    loom::model(|| {
        let (tx, mut rx) = channel();
        let producer = loom::thread::spawn(move || {
            tx.push(Box::new(41));
            tx.push(Box::new(42));
        });
        let mut got = Vec::new();
        while got.len() < 2 {
            match rx.pop() {
                PopResult::Item(v) => got.push(*v),
                PopResult::Empty | PopResult::Inconsistent => loom::thread::yield_now(),
            }
        }
        assert_eq!(got, vec![41, 42]); // 單 producer:嚴格 FIFO
        producer.join().unwrap();
    });
}

/// 帶著未消費元素(含卡在縫裡的)結束:Drop 沿鏈回收,所有交錯下
/// 不洩漏、不 double-free(loom leak 檢查 + Box double-free 會炸)。
#[test]
fn loom_mpsc_drop_midstream_no_leak() {
    loom::model(|| {
        let (tx, mut rx) = channel();
        let producer = loom::thread::spawn(move || {
            tx.push(Box::new(1u32));
            tx.push(Box::new(2u32));
        });
        let _ = rx.pop(); // 可能 Item/Empty/Inconsistent 都行
        producer.join().unwrap();
        // rx 與內部 Arc 陸續 drop → MpscList::drop 清整條鏈
    });
}
