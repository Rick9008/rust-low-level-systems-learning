//! loom 窮舉驗證:MPSC ring(Vyukov 單消費退化)。
//!
//! 兩個驗證目標:①producer 側(CAS 取號 + seq 發布)在所有交錯下不丟不重;
//! ②consumer 側 head 的「非原子」宣稱——head 是 loom 的 UnsafeCell,
//! 任何與它的並發存取都會被 loom 的存取追蹤當場抓包。跑得過 = 型別系統
//! (Consumer 非 Clone + &mut self)真的把 head 圈成了私有狀態。
//!
//! 機關同 loom_spsc:`sync_shim` 換成 loom 型別,`#[path]` include
//! **同一份**核心演算法原始碼。

// loom 版 shim:名稱、API 與 lib 的 `crate::sync_shim` 完全對齊。
mod sync_shim {
    pub(crate) use loom::cell::UnsafeCell;
    pub(crate) use loom::sync::Arc;
    pub(crate) use loom::sync::atomic;
}

// 測試只用到 channel/try_push/try_pop;其餘 API 是 lib 的事,不算 dead code。
#[allow(dead_code)]
#[path = "../src/concurrency/mpsc_ring/core_impl.rs"]
mod core_impl;

use core_impl::channel;

/// 2 producers × 1 元素、cap 2:CAS 取號對撞的所有交錯下,
/// consumer 收齊兩個值、不丟不重;head 全程無並發存取。
#[test]
fn loom_mpsc_ring_two_producers_all_interleavings() {
    loom::model(|| {
        let (tx, mut rx) = channel(2);
        let mut handles = Vec::new();
        for v in [1u32, 2] {
            let tx = tx.clone();
            handles.push(loom::thread::spawn(move || {
                tx.try_push(v).unwrap(); // cap=2、各推一個,必成功
            }));
        }
        drop(tx);
        let mut got = Vec::new();
        while got.len() < 2 {
            match rx.try_pop() {
                Some(v) => got.push(v),
                None => loom::thread::yield_now(),
            }
        }
        got.sort_unstable();
        assert_eq!(got, vec![1, 2]);
        assert_eq!(rx.try_pop(), None);
        for h in handles {
            h.join().unwrap();
        }
    });
}

/// 滿載 backpressure + 第二圈重用:1 producer 推 3 個過 cap=2 的環
/// (滿了 Err 重試),單 producer ⇒ 取號序 = push 序,FIFO 嚴格成立。
#[test]
fn loom_mpsc_ring_full_backpressure_and_lap_reuse() {
    loom::model(|| {
        let (tx, mut rx) = channel(2);
        let producer = loom::thread::spawn(move || {
            for v in 0..3u32 {
                let mut item = v;
                while let Err(back) = tx.try_push(item) {
                    item = back;
                    loom::thread::yield_now(); // 滿:讓 loom 排 consumer
                }
            }
        });
        for expect in 0..3u32 {
            loop {
                match rx.try_pop() {
                    Some(v) => {
                        assert_eq!(v, expect);
                        break;
                    }
                    None => loom::thread::yield_now(),
                }
            }
        }
        assert_eq!(rx.try_pop(), None);
        producer.join().unwrap();
    });
}

/// 帶著未消費元素結束:Drop 排空在所有交錯下不洩漏、不 double-drop
/// (loom leak 檢查 + Box double-free 會炸)。
#[test]
fn loom_mpsc_ring_drop_midstream_no_leak() {
    loom::model(|| {
        let (tx, rx) = channel(2);
        let producer = loom::thread::spawn(move || {
            let _ = tx.try_push(Box::new(1u32));
            let _ = tx.try_push(Box::new(2u32));
        });
        drop(rx); // consumer 提早離場
        producer.join().unwrap();
        // Arc 歸零 → MpscRing::drop 清殘留元素
    });
}
