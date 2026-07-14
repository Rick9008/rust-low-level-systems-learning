//! loom 窮舉驗證:arena lock-free stack。
//!
//! 重點劇本是 **ABA**:pop 讀了 (head, next) 之後、CAS 之前,
//! 另一執行緒完成 pop→free→alloc→push,讓同一個索引帶著不同 next 回到 head。
//! generation tag 讓這種交錯下舊 CAS 必敗——loom 會把這條路徑真的排出來走。
//! 若把 core_impl 的 gen 拿掉(或 CAS 只比 idx),這些測試會失敗。

mod sync_shim {
    pub(crate) use loom::cell::UnsafeCell;
    pub(crate) use loom::sync::atomic;
}

// 測試只用到 push/pop;capacity 等其餘 API 是 lib 的事,不算 dead code。
#[allow(dead_code)]
#[path = "../src/arena_lockfree/core_impl.rs"]
mod core_impl;

use core_impl::ArenaStack;
use loom::sync::Arc;

/// 兩個 pusher 並發:所有交錯下集合正確(不丟、不重)。
#[test]
fn loom_two_pushers_no_loss() {
    loom::model(|| {
        let s = Arc::new(ArenaStack::new(2));
        let s1 = Arc::clone(&s);
        let s2 = Arc::clone(&s);
        let t1 = loom::thread::spawn(move || s1.push(1).unwrap());
        let t2 = loom::thread::spawn(move || s2.push(2).unwrap());
        t1.join().unwrap();
        t2.join().unwrap();
        let mut got = [s.pop().unwrap(), s.pop().unwrap()];
        got.sort_unstable();
        assert_eq!(got, [1, 2]);
        assert_eq!(s.pop(), None);
    });
}

/// 兩個 popper 搶同一個 head:恰好一人得手,另一人拿到下一個或 None。
/// 驗證 CAS 競爭路徑(compare_exchange 失敗重試)不重複發放同一元素。
#[test]
fn loom_two_poppers_exactly_once() {
    loom::model(|| {
        let s = Arc::new(ArenaStack::new(2));
        s.push(1).unwrap();
        s.push(2).unwrap();
        let s1 = Arc::clone(&s);
        let s2 = Arc::clone(&s);
        let t1 = loom::thread::spawn(move || s1.pop());
        let t2 = loom::thread::spawn(move || s2.pop());
        let (a, b) = (t1.join().unwrap(), t2.join().unwrap());
        // 兩人合計恰好拿走 1 和 2 各一次
        let mut got = [a.unwrap(), b.unwrap()];
        got.sort_unstable();
        assert_eq!(got, [1, 2]);
        assert_eq!(s.pop(), None);
    });
}

/// ABA 劇本:A 執行 pop(可能卡在讀完 next、CAS 前),
/// B 同時完成 pop → 槽位回收 → push(重用同一槽位、next 已不同)。
/// 無 generation 時 A 的 CAS 會錯誤成功、結構損毀(丟失或重複元素);
/// 有 generation 時所有交錯下多重集不變量成立。
#[test]
fn loom_aba_recycle_race() {
    loom::model(|| {
        let s = Arc::new(ArenaStack::new(2));
        s.push(1).unwrap();
        let sa = Arc::clone(&s);
        let sb = Arc::clone(&s);
        let a = loom::thread::spawn(move || sa.pop());
        let b = loom::thread::spawn(move || {
            let popped = sb.pop(); // 可能搶先拿走 1(槽位回收)
            sb.push(2).unwrap(); // 極可能重用剛回收的槽位 → ABA 現場
            popped
        });
        let ra = a.join().unwrap();
        let rb = b.join().unwrap();
        // 不變量:1 恰好被 a 或 b 其中一人拿到;2 在 stack 裡或沒被拿走
        let mut taken: Vec<u32> = [ra, rb].into_iter().flatten().collect();
        let mut rest = Vec::new();
        while let Some(v) = s.pop() {
            rest.push(v);
        }
        taken.append(&mut rest);
        taken.sort_unstable();
        assert_eq!(taken, vec![1, 2]); // 不丟、不重
    });
}
