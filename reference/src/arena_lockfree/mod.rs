//! # arena_lockfree —— generation-tagged index 的 lock-free stack
//!
//! ## [Clarify]
//! 解決:多執行緒共享的 LIFO free-list / 物件池(纜線是 MPMC),
//! push/pop 無鎖。同時示範 lock-free 世界的第一大坑:**ABA**,
//! 以及 index + generation 的標準解法。
//! Constraints:容量固定(bounded,滿了 Err);索引空間 u32(MAX 當 NIL),
//! generation u32——2^32 次操作內的 ABA 免疫(取捨見下)。
//!
//! ## [Abstract]
//! 元素型別泛型;「滿了怎麼辦」還給 caller。回收策略只做 free list
//! (epoch / hazard pointer 是 production 級答案,聲明提及不實作)。
//!
//! ## [Iterate]
//! 演進線:Treiber stack 裸指標版(有 ABA + use-after-free 兩個雷)→
//! **arena 索引版**(use-after-free 消失:索引永遠在 bounds 內)→
//! **+ generation tag**(ABA 消失)。本模組直接是第三形態,
//! ABA 的攻擊劇本完整註在 `pop()` 內與 loom 測試。
//!
//! ## [Trade-offs]
//! - index 換 pointer:解掉回收安全(索引永遠有效,最多指到「內容已換人」
//!   的槽位——由 gen 擋掉),這正是 Rust 學 lock-free 先走 arena 的原因。
//! - (gen:32|idx:32) 打包進一個 AtomicU64:單字 CAS 原子地同時比對兩者。
//!   gen 迴繞(2^32 次)理論上可 ABA——一條執行緒要剛好卡在
//!   CAS 前睡到 42 億次操作發生,實務接受;不接受就 128-bit CAS 或 epoch。
//! - lock-free ≠ wait-free:競爭下單執行緒可能重試多次,但整體必有進展
//!   (每次 CAS 失敗 ⇔ 別人成功了)。
//! - 時間攤銷 O(1);空間 O(cap)(slot = value + next + padding)。
//!
//! ## [Dry-Run]
//! 單執行緒:LIFO trace、滿/空、槽位回收重用。並發:煙霧測試 +
//! **loom 窮舉**(`tests/loom_arena.rs`):雙 popper 搶同一個 head、
//! pop 與「pop→push 回收重用」交錯——正是 ABA 的觸發劇本。
//!
//! Production 對照:crossbeam(epoch-based reclamation)、
//! crossbeam::queue::ArrayQueue(bounded MPMC)。

mod core_impl;

pub use core_impl::ArenaStack;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// [Dry-Run] LIFO trace(cap=4):
    ///   push(1): 佔槽 0,head=(g1,0)     push(2): 佔槽 1,next=0,head=(g2,1)
    ///   pop→2(頂),槽 1 回 free 鏈      pop→1   pop→None(空,idx=NIL)
    #[test]
    fn lifo_order_roundtrip() {
        let s = ArenaStack::new(4);
        s.push(1).unwrap();
        s.push(2).unwrap();
        assert_eq!(s.pop(), Some(2));
        assert_eq!(s.pop(), Some(1));
        assert_eq!(s.pop(), None);
    }

    /// boundary:滿 → Err 歸還;pop 一個之後空位回收,再 push 成功。
    /// 這同時驗證 free 鏈的回收路徑(pop → free_slot → alloc_slot 重用)。
    #[test]
    fn boundary_full_then_recycle_slot() {
        let s = ArenaStack::new(2);
        s.push(10).unwrap();
        s.push(20).unwrap();
        assert_eq!(s.push(30), Err(30)); // 滿
        assert_eq!(s.pop(), Some(20));
        s.push(40).unwrap(); // 重用剛回收的槽位
        assert_eq!(s.pop(), Some(40));
        assert_eq!(s.pop(), Some(10));
    }

    /// boundary:cap=1 退化——每次操作都在滿/空邊界上。
    #[test]
    fn boundary_cap_one() {
        let s = ArenaStack::new(1);
        s.push(1).unwrap();
        assert_eq!(s.push(2), Err(2));
        assert_eq!(s.pop(), Some(1));
        assert_eq!(s.pop(), None);
        s.push(3).unwrap();
        assert_eq!(s.pop(), Some(3));
    }

    /// 並發煙霧測試:4 執行緒 × 1000 push,全收齊、無重複。
    /// (窮舉版證明在 tests/loom_arena.rs。)
    #[test]
    fn concurrent_push_pop_no_loss_no_dup() {
        let s = Arc::new(ArenaStack::new(4096));
        let handles: Vec<_> = (0..4u32)
            .map(|t| {
                let s = Arc::clone(&s);
                thread::spawn(move || {
                    for i in 0..1000 {
                        s.push(t * 1000 + i).unwrap();
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        let mut got = Vec::new();
        while let Some(v) = s.pop() {
            got.push(v);
        }
        got.sort_unstable();
        assert_eq!(got, (0..4000).collect::<Vec<_>>());
    }

    /// boundary:帶著元素 drop——stack 鏈上的值全部要被 drop,不洩漏。
    #[test]
    fn boundary_drop_with_items_no_leak() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        #[derive(Debug)]
        struct CountDrop(Arc<AtomicUsize>);
        impl Drop for CountDrop {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }

        let drops = Arc::new(AtomicUsize::new(0));
        {
            let s = ArenaStack::new(4);
            s.push(CountDrop(Arc::clone(&drops))).unwrap();
            s.push(CountDrop(Arc::clone(&drops))).unwrap();
            drop(s.pop()); // 1 個正常消費
            assert_eq!(drops.load(Ordering::Relaxed), 1);
        } // drop 剩餘 1 個
        assert_eq!(drops.load(Ordering::Relaxed), 2);
    }
}
