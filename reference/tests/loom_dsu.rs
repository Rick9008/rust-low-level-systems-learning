//! loom 窮舉驗證:lock-free DSU。
//!
//! 重點劇本:
//! 1. 同一對節點雙 union——link CAS 的 expected value(root 自指)必須保證
//!    恰好一人成功;若把 compare_exchange 改成 blind store,這裡會抓到雙贏。
//! 2. 鏈式 union 併發——連通性收斂、components 精確(每條邊只成功合併一次)。
//! 3. find(path halving)與 union 交錯——halving CAS 失敗被安全忽略
//!    (它只是 hint),find 回傳值永遠留在自己的集合裡。
//! 免 generation tag 的理由(parent 單調、root 資格一去不返)由這些交錯實測。

mod sync_shim {
    pub(crate) use loom::sync::atomic;
}

// 測試只用到部分 API;其餘(len/is_empty 等)是 lib 的事,不算 dead code。
#[allow(dead_code)]
#[path = "../src/ds_sync/dsu_lockfree/core_impl.rs"]
mod core_impl;

use core_impl::DsuLockFree;
use loom::sync::Arc;

/// 兩執行緒 union 同一對:所有交錯下恰好一人真的合併。
#[test]
fn loom_racing_unions_exactly_one_wins() {
    loom::model(|| {
        let d = Arc::new(DsuLockFree::new(2));
        let d1 = Arc::clone(&d);
        let d2 = Arc::clone(&d);
        let t1 = loom::thread::spawn(move || d1.union(0, 1));
        let t2 = loom::thread::spawn(move || d2.union(0, 1));
        let (a, b) = (t1.join().unwrap(), t2.join().unwrap());
        assert!(a ^ b, "恰好一個 union 真的發生合併");
        assert!(d.connected(0, 1));
        assert_eq!(d.components(), 1);
    });
}

/// 鏈式 union 併發:0-1 與 1-2 兩條邊都必成(端點不相交於同集合),
/// 收斂後全連通、components == 1。
#[test]
fn loom_chain_unions_converge() {
    loom::model(|| {
        let d = Arc::new(DsuLockFree::new(3));
        let d1 = Arc::clone(&d);
        let d2 = Arc::clone(&d);
        let t1 = loom::thread::spawn(move || d1.union(0, 1));
        let t2 = loom::thread::spawn(move || d2.union(1, 2));
        assert!(t1.join().unwrap());
        assert!(t2.join().unwrap());
        assert!(d.connected(0, 2));
        assert!(d.connected(0, 1) && d.connected(1, 2));
        assert_eq!(d.components(), 1);
    });
}

/// find 與 union 交錯:先造一條鏈讓 find 有 halving 可做,
/// 再讓 union(0,2) 同時改結構。不變量:find 回傳的節點永遠在 0 的集合裡
/// (集合只合不拆,「曾是 0 的根」不可能漂到別的集合)。
#[test]
fn loom_find_races_union_halving_harmless() {
    loom::model(|| {
        let d = Arc::new(DsuLockFree::new(3));
        assert!(d.union(0, 1)); // 併發開始前:{0,1} {2}
        let df = Arc::clone(&d);
        let du = Arc::clone(&d);
        let tf = loom::thread::spawn(move || df.find(0));
        let tu = loom::thread::spawn(move || du.union(0, 2));
        let root_seen = tf.join().unwrap();
        assert!(tu.join().unwrap());
        assert!(d.connected(root_seen, 0), "find 的回傳留在 0 的集合裡");
        assert!(d.connected(0, 2));
        assert_eq!(d.components(), 1);
    });
}
