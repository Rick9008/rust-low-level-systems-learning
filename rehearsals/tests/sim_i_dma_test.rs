//! 參考測試:sim i(DMA dispatcher v2)。
//!
//! 彩排時先自己寫測試(寫在 src/sim_i_dma.rs 底部);自己的測試轉綠後
//! 才跑這組對照:
//! `cargo test -p rehearsals --test sim_i_dma_test -- --include-ignored`

use rehearsals::sim_i_dma::{SimBus, run};

/// boundary:單一 request 3 塊——每塊派對位置、做完才 submit 一次。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn single_request_three_blocks() {
    let mut bus = SimBus::new().request_at(0, 1, 3, 100);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![1]);
    let mut sent: Vec<_> = bus.sent_log.iter().map(|&(_, b, p)| (b, p)).collect();
    sent.sort_unstable();
    assert_eq!(sent, vec![(0, 100), (1, 101), (2, 102)]);
}

/// pipeline 證明:A 佔滿 6 台時 B 進場,B 的塊必須插隊跑(sequential 版過不了)。
/// lifo 完成序 → B 的塊先做完 → **B 先 submit**。R1 的 `queue.len()==6` 判定在這裡爆。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn pipeline_two_requests_interleave() {
    let mut bus = SimBus::new()
        .lifo()
        .request_at(0, 1, 6, 0) // A:塞滿全部 engine
        .request_at(0, 2, 1, 100); // B:一塊,第一台 engine 釋出就該上
    run(&mut bus);
    assert_eq!(
        bus.submitted,
        vec![2, 1],
        "B 必須先完成——request 之間要 pipeline"
    );
}

/// done 只給 engine id:三個 request 交錯在飛,路由靠你自己的佔用表。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn done_routing_three_requests() {
    let mut bus = SimBus::new()
        .lifo()
        .request_at(0, 1, 4, 0)
        .request_at(0, 2, 4, 100)
        .request_at(1, 3, 2, 200);
    run(&mut bus);
    let mut s = bus.submitted.clone();
    s.sort_unstable();
    assert_eq!(s, vec![1, 2, 3]);
}

/// Phase 2 cancel:取消的 request 永不 submit,它的 engine 要能回收給下一單。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn cancel_reclaims_engines() {
    let mut bus = SimBus::new()
        .request_at(0, 1, 6, 0)
        .cancel_at(1, 1)
        .request_at(2, 2, 3, 100);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![2], "cancel 的 1 不准出現;2 必須完成");
}

/// boundary:0 塊的 request——不派工,直接 submit(clarify 挖得到的 spec)。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn zero_block_request_submits_immediately() {
    let mut bus = SimBus::new().request_at(0, 7, 0, 500);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![7]);
    assert!(bus.sent_log.is_empty());
}
