//! loom 窮舉驗證:SPSC ring。
//!
//! 這不是 fuzz——`loom::model` 把閉包重跑到「所有可能的執行緒交錯 +
//! C11 記憶體模型允許的所有可見性結果」全部覆蓋為止。
//! 模型刻意小(2 個元素、容量 1/2):狀態空間隨操作數指數成長,
//! 小模型已足以打出 acquire/release 寫錯、槽位讀寫競爭等所有一步錯誤。
//!
//! 機關:`sync_shim` 在這裡換成 loom 型別,再以 `#[path]` include
//! **同一份**核心演算法原始碼——loom 驗的就是 lib 出貨的那份邏輯。

// loom 版 shim:名稱、API 與 lib 的 `crate::sync_shim` 完全對齊。
mod sync_shim {
    pub(crate) use loom::cell::UnsafeCell;
    pub(crate) use loom::sync::Arc;
    pub(crate) use loom::sync::atomic;
}

// 測試只用到 channel/push/pop;其餘 API 是 lib 的事,不算 dead code。
#[allow(dead_code)]
#[path = "../src/concurrency/spsc_ring/core_impl.rs"]
mod core_impl;

use core_impl::channel;

/// 容量 1(最緊的 backpressure):producer 推 2 個元素、consumer 拉 2 個。
/// 所有交錯下:順序不亂、不丟、不重、滿時 Err 後重試能成功。
#[test]
fn loom_spsc_cap1_two_items_all_interleavings() {
    loom::model(|| {
        let (mut tx, mut rx) = channel(1);
        let producer = loom::thread::spawn(move || {
            for i in 0..2u32 {
                let mut item = i;
                loop {
                    match tx.push(item) {
                        Ok(()) => break,
                        Err(back) => {
                            item = back;
                            loom::thread::yield_now(); // 滿:讓 loom 排 consumer
                        }
                    }
                }
            }
        });
        for expect in 0..2u32 {
            loop {
                match rx.pop() {
                    Some(v) => {
                        assert_eq!(v, expect); // FIFO 且值完整
                        break;
                    }
                    None => loom::thread::yield_now(),
                }
            }
        }
        assert_eq!(rx.pop(), None); // 全部收完必為空
        producer.join().unwrap();
    });
}

/// 容量 2 + 非 Copy 型別(Box):同時驗 happens-before 的資料可見性
/// (Box 內容跨執行緒移交)與 Drop 正確性(loom 會檢查洩漏)。
#[test]
fn loom_spsc_cap2_boxed_values() {
    loom::model(|| {
        let (mut tx, mut rx) = channel(2);
        let producer = loom::thread::spawn(move || {
            tx.push(Box::new(41)).unwrap(); // cap=2,前兩次 push 必成功
            let mut item = Box::new(42);
            loop {
                match tx.push(item) {
                    Ok(()) => break,
                    Err(back) => {
                        item = back;
                        loom::thread::yield_now();
                    }
                }
            }
        });
        let mut got = Vec::new();
        while got.len() < 2 {
            match rx.pop() {
                Some(v) => got.push(*v),
                None => loom::thread::yield_now(),
            }
        }
        assert_eq!(got, vec![41, 42]);
        producer.join().unwrap();
    });
}

/// 帶著未消費元素結束:Drop 清理在所有交錯下都不洩漏、不 double-drop
/// (loom 的 leak 檢查 + Box 的 double-free 會直接炸)。
#[test]
fn loom_spsc_drop_midstream_no_leak() {
    loom::model(|| {
        let (mut tx, rx) = channel(2);
        let producer = loom::thread::spawn(move || {
            let _ = tx.push(Box::new(1)); // 可能成功也可能滿(consumer 不拉)
            let _ = tx.push(Box::new(2));
        });
        drop(rx); // consumer 提早離場
        producer.join().unwrap();
        // tx 在 producer 執行緒結尾 drop;SpscRing::drop 負責清掉殘留元素
    });
}
