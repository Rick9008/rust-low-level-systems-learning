//! loom 窮舉驗證:work-stealing deque(Chase–Lev,教學版)。
//!
//! 重點交錯:**最後一件的決鬥**——owner pop「降 bottom → fence → 讀 top」
//! 與 stealer「讀 top → fence → 讀 bottom」是教科書 SB(store-buffering)
//! 形狀;把任一個 fence(SeqCst) 拿掉,loom 會當場示範 double-take。
//! (自己動手驗:改成 Acquire/Release 再跑,看它怎麼炸。)
//!
//! 機關同 loom_spsc:`sync_shim` 換成 loom 型別,`#[path]` include
//! **同一份**核心演算法原始碼。

// loom 版 shim:名稱、API 與 lib 的 `crate::sync_shim` 完全對齊。
mod sync_shim {
    // ws_deque 的槽位是 AtomicPtr,用不到 UnsafeCell——但 shim 的 API 面
    // 必須與 lib 的 `crate::sync_shim` 對齊,故保留。
    #[allow(unused_imports)]
    pub(crate) use loom::cell::UnsafeCell;
    pub(crate) use loom::sync::Arc;
    pub(crate) use loom::sync::atomic;
}

// 測試只用到 deque/push/pop/steal;其餘 API 是 lib 的事,不算 dead code。
#[allow(dead_code)]
#[path = "../src/concurrency/ws_deque/core_impl.rs"]
mod core_impl;

use core_impl::{Steal, deque};

/// 最後一件決鬥:1 個元素,owner pop vs stealer steal——
/// 所有交錯下**恰好一人**拿到(double-take 或蒸發都是 fence 壞掉的病徵)。
#[test]
fn loom_wsd_last_item_duel_exactly_one_wins() {
    loom::model(|| {
        let (mut owner, stealer) = deque(2);
        owner.push(Box::new(7u32)).unwrap();
        let thief = loom::thread::spawn(move || {
            loop {
                match stealer.steal() {
                    Steal::Item(v) => return Some(*v),
                    Steal::Empty => return None,
                    Steal::Retry => loom::thread::yield_now(), // 輸了決鬥:再看一眼
                }
            }
        });
        let mine = owner.pop().map(|b| *b);
        let stolen = thief.join().unwrap();
        assert!(
            matches!((mine, stolen), (Some(7), None) | (None, Some(7))),
            "決鬥必須恰好一人贏:owner={mine:?} stealer={stolen:?}"
        );
    });
}

/// push 與 steal 並發 + owner 收尾:2 個元素在所有交錯下不丟不重
/// (Box 值:重複取走會 double-free 直接炸)。
#[test]
fn loom_wsd_concurrent_push_steal_exactly_once() {
    loom::model(|| {
        let (mut owner, stealer) = deque::<Box<u32>>(2);
        let thief = loom::thread::spawn(move || {
            let mut got = Vec::new();
            loop {
                match stealer.steal() {
                    Steal::Item(v) => got.push(*v),
                    Steal::Retry => loom::thread::yield_now(),
                    Steal::Empty => break, // 空:owner 可能還沒 push 完,交還帳本
                }
            }
            got
        });
        owner.push(Box::new(1u32)).unwrap();
        owner.push(Box::new(2u32)).unwrap();
        let mut mine = Vec::new();
        while let Some(v) = owner.pop() {
            mine.push(*v);
        }
        let mut all = thief.join().unwrap();
        all.extend(mine);
        all.sort_unstable();
        // stealer 可能提早看到 Empty 就收工——它拿到的是子集;
        // 但「stealer + owner 的聯集」必須恰好是 {1, 2}。
        assert_eq!(all, vec![1, 2], "不丟不重:{all:?}");
    });
}

/// 帶著未取走元素結束:Drop 回收 [top, bottom),所有交錯下
/// 不洩漏、不 double-free。
#[test]
fn loom_wsd_drop_midstream_no_leak() {
    loom::model(|| {
        let (mut owner, stealer) = deque(2);
        owner.push(Box::new(1u32)).unwrap();
        owner.push(Box::new(2u32)).unwrap();
        let thief = loom::thread::spawn(move || {
            match stealer.steal() {
                Steal::Item(v) => drop(v), // 取走一個
                Steal::Empty | Steal::Retry => {}
            }
        });
        thief.join().unwrap();
        // owner 與內部 Arc 陸續 drop → WsDeque::drop 清殘餘
    });
}
