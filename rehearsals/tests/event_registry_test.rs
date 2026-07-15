//! 參考測試:event_registry。
//!
//! 彩排時先自己寫測試;轉綠後才跑這組:
//! `cargo test -p rehearsals --test event_registry_test -- --include-ignored`

use rehearsals::event_registry::{After, Registry};
use std::cell::RefCell;
use std::rc::Rc;

/// boundary:未知 id——dispatch 是 no-op,回 0,不 panic。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn unknown_id_is_noop() {
    let mut r = Registry::new();
    assert_eq!(r.dispatch(42, 7), 0);
    assert_eq!(r.handler_count(42), 0);
}

/// 同一 id 多個 handler:依註冊順序執行,payload 正確傳入,id 之間互相隔離。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn handlers_fire_in_registration_order() {
    let mut r = Registry::new();
    let log: Rc<RefCell<Vec<(u32, u64)>>> = Rc::new(RefCell::new(Vec::new()));

    let l1 = Rc::clone(&log);
    r.register(
        7,
        Box::new(move |p| {
            l1.borrow_mut().push((1, p));
            After::Keep
        }),
    );
    let l2 = Rc::clone(&log);
    r.register(
        7,
        Box::new(move |p| {
            l2.borrow_mut().push((2, p));
            After::Keep
        }),
    );
    let l3 = Rc::clone(&log);
    r.register(
        8,
        Box::new(move |p| {
            l3.borrow_mut().push((3, p));
            After::Keep
        }),
    );

    assert_eq!(r.dispatch(7, 42), 2);
    assert_eq!(*log.borrow(), vec![(1, 42), (2, 42)]); // 順序 + payload;id 8 沒被叫
    assert_eq!(r.handler_count(7), 2);
    assert_eq!(r.handler_count(8), 1);
}

/// boundary:dispatch 中途 unregister——handler 以回傳值 Remove 自我移除,
/// 這一輪仍算執行,下一輪起消失;其餘 handler 不受影響。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn remove_takes_effect_after_this_dispatch() {
    let mut r = Registry::new();
    let hits: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));

    let h1 = Rc::clone(&hits);
    r.register(
        1,
        Box::new(move |_| {
            h1.borrow_mut().push("once");
            After::Remove // 跑一次就自我移除
        }),
    );
    let h2 = Rc::clone(&hits);
    r.register(
        1,
        Box::new(move |_| {
            h2.borrow_mut().push("keep");
            After::Keep
        }),
    );

    assert_eq!(r.dispatch(1, 0), 2); // 兩個都跑(Remove 這輪仍算)
    assert_eq!(r.handler_count(1), 1);
    assert_eq!(r.dispatch(1, 0), 1); // once 已消失
    assert_eq!(*hits.borrow(), vec!["once", "keep", "keep"]);
}

/// 有狀態 handler(FnMut):跨多次 dispatch 累積狀態。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn stateful_handler_accumulates() {
    let mut r = Registry::new();
    let sum = Rc::new(RefCell::new(0u64));
    let s = Rc::clone(&sum);
    r.register(
        3,
        Box::new(move |p| {
            *s.borrow_mut() += p;
            After::Keep
        }),
    );
    r.dispatch(3, 10);
    r.dispatch(3, 32);
    assert_eq!(*sum.borrow(), 42);
}

/// 規模 sanity:一千個 id 各掛一個 handler,各 dispatch 一次,互不串音。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn thousand_ids_isolated() {
    let mut r = Registry::new();
    let count = Rc::new(RefCell::new(vec![0u32; 1000]));
    for id in 0..1000u32 {
        let c = Rc::clone(&count);
        r.register(
            id,
            Box::new(move |_| {
                c.borrow_mut()[id as usize] += 1;
                After::Keep
            }),
        );
    }
    for id in 0..1000u32 {
        assert_eq!(r.dispatch(id, 0), 1);
    }
    assert!(count.borrow().iter().all(|&c| c == 1));
}
