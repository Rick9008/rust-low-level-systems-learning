//! 參考測試:fd_registry。
//!
//! 彩排時先自己寫測試;轉綠後才跑這組:
//! `cargo test -p rehearsals --test fd_registry_test -- --include-ignored`

use rehearsals::fd_registry::{FdRegistry, Token};

/// 核心 boundary:fd 回收重用——舊 token 必死、新 token 必活、
/// stale 操作不影響現任住戶。這就是題幹描述的那個 bug。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn fd_reuse_stale_token_dies() {
    let mut r = FdRegistry::new();
    let t1 = r.register(5, "A");
    assert_eq!(r.unregister(t1), Some("A"));

    let t2 = r.register(5, "B"); // kernel 把 5 發給新連線
    assert_eq!(r.get(t1), None, "過期 token 不准查到新住戶");
    assert_eq!(r.get(t2), Some(&"B"));
    assert_eq!(r.unregister(t1), None, "stale unregister 是 no-op");
    assert_eq!(r.len(), 1);
    assert_eq!(r.get(t2), Some(&"B"), "現任住戶不受 stale 操作影響");
}

/// token 經 u64 往返(這就是 epoll_event.data 的旅程)後仍解析。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn token_survives_u64_roundtrip() {
    let mut r = FdRegistry::new();
    let t = r.register(3, 30);
    let raw: u64 = t.to_raw();
    assert_eq!(r.get(Token::from_raw(raw)), Some(&30));
}

/// boundary:空表 / 從未登記的 slot,偽造 token 一律安全回 None。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn forged_tokens_are_safe() {
    let mut r: FdRegistry<i32> = FdRegistry::new();
    assert_eq!(r.get(Token::from_raw(5)), None);
    assert_eq!(r.unregister(Token::from_raw(5)), None);

    r.register(10, 1); // 表增長後,fd 3 仍從未登記
    assert_eq!(r.get(Token::from_raw(3)), None);
    assert_eq!(r.len(), 1);
}

/// dispatch 端用 get_mut 就地改狀態。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn get_mut_mutates_in_place() {
    let mut r = FdRegistry::new();
    let t = r.register(7, vec![1, 2]);
    r.get_mut(t).unwrap().push(3);
    assert_eq!(r.get(t), Some(&vec![1, 2, 3]));
}

/// 規模 sanity:千 fd 高 churn——偶數位換代後舊 token 全滅、新的全活,
/// 奇數位第一代不受影響。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn thousand_fd_churn() {
    let mut r = FdRegistry::new();
    let gen0: Vec<Token> = (0..1000).map(|fd| r.register(fd, fd)).collect();
    for fd in (0..1000).step_by(2) {
        assert_eq!(r.unregister(gen0[fd]), Some(fd));
    }
    let gen1: Vec<Token> = (0..1000)
        .step_by(2)
        .map(|fd| r.register(fd, fd + 10_000))
        .collect();

    assert_eq!(r.len(), 1000);
    for fd in (0..1000).step_by(2) {
        assert_eq!(r.get(gen0[fd]), None);
    }
    for (i, fd) in (0..1000).step_by(2).enumerate() {
        assert_eq!(r.get(gen1[i]), Some(&(fd + 10_000)));
    }
    for fd in (1..1000).step_by(2) {
        assert_eq!(r.get(gen0[fd]), Some(&fd));
    }
}
