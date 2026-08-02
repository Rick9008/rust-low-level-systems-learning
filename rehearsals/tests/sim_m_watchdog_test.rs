//! 參考測試:sim m(engine watchdog)。
//!
//! 彩排完才開:
//! `cargo test -p rehearsals --test sim_m_watchdog_test -- --include-ignored`
//!
//! 假設:你的 timeout 取值 < 500ms(任何「p99 塊延遲的數倍」合理值都在這內)。

use rehearsals::sim_m_watchdog::{SimBus, run};

/// 單次 hang:block 0 第一次派工就 hang(永不完成)→ 必須 timeout 後重派到別台,
/// request 照樣完成。sent_log 裡 (rid=1, block=0) 應出現兩筆。
#[test]
fn redispatch_on_single_hang() {
    let mut bus = SimBus::new()
        .request_at_ms(0, 1, 2, 0)
        .hang_once(1, 0, None);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![1]);
    let resends = bus
        .sent_log
        .iter()
        .filter(|&&(_, r, b)| r == 1 && b == 0)
        .count();
    assert_eq!(resends, 2, "hang 的塊必須被重派恰好一次");
}

/// zombie done:hang 的塊在 500ms 後仍吐出 done——那時你早已重派並 submit。
/// zombie 不准弄髒帳:不能重複 submit、不能 panic、後續 request 照常。
#[test]
fn zombie_done_is_harmless() {
    let mut bus = SimBus::new()
        .request_at_ms(0, 1, 2, 0)
        .hang_once(1, 0, Some(500))
        .request_at_ms(600, 2, 1, 100);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![1, 2]);
    assert!(bus.errors.is_empty());
}

/// retry budget:block 0 每次派工都 hang → 3 次 timeout 後放棄,
/// 走 error 路徑回報整張 request,不准無限重試。
#[test]
fn retry_budget_reports_error() {
    let mut bus = SimBus::new().request_at_ms(0, 1, 2, 0).hang_always(1, 0);
    run(&mut bus);
    assert_eq!(bus.errors, vec![1]);
    assert!(bus.submitted.is_empty(), "報錯的 request 不准又 submit");
    let tries = bus
        .sent_log
        .iter()
        .filter(|&&(_, r, b)| r == 1 && b == 0)
        .count();
    assert_eq!(tries, 3, "retry budget = 3:三次就收手");
}
