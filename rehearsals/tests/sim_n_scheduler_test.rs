//! 參考測試:sim n(priority job scheduler)。
//!
//! 彩排完才開:
//! `cargo test -p rehearsals --test sim_n_scheduler_test -- --include-ignored`

use rehearsals::sim_n_scheduler::{SimBus, run};

/// 優先權波次:5 張 job 同時到、4 台 worker——前四個派出的必須是優先權前四名
/// (同權 FIFO),第 5 張等第一台 worker 釋出。派工順序 = 你的排程決策順序。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn priority_waves() {
    let mut bus = SimBus::new()
        .job_at(0, 1, 1, &[])
        .job_at(0, 2, 9, &[])
        .job_at(0, 3, 5, &[])
        .job_at(0, 4, 9, &[])
        .job_at(0, 5, 3, &[]);
    run(&mut bus);
    let order: Vec<_> = bus.assigned_log.iter().map(|&(_, j)| j).collect();
    assert_eq!(order, vec![2, 4, 3, 5, 1], "p9(FIFO 破平手)→ p5 → p3 → p1");
}

/// 同優先權 = 到達順序(FIFO)。排程器不准把同權 job 洗亂。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn same_priority_is_fifo() {
    let mut bus = SimBus::new()
        .job_at(0, 10, 7, &[])
        .job_at(0, 11, 7, &[])
        .job_at(0, 12, 7, &[])
        .job_at(0, 13, 7, &[])
        .job_at(0, 14, 7, &[])
        .job_at(0, 15, 7, &[]);
    run(&mut bus);
    let order: Vec<_> = bus.assigned_log.iter().map(|&(_, j)| j).collect();
    assert_eq!(order, vec![10, 11, 12, 13, 14, 15]);
}

/// Phase 2 相依:C 等 A+B、D 等 C——deps 沒齊就派工,SimBus 的 oracle 會當場 panic。
/// 全部完成且 C 在 A、B 之後、D 最後。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn dag_admission() {
    let mut bus = SimBus::new()
        .job_at(0, 1, 5, &[])
        .job_at(0, 2, 5, &[])
        .job_at(0, 3, 9, &[1, 2]) // 高優先權也得等相依——priority 不能穿越 DAG
        .job_at(0, 4, 9, &[3]);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![1, 2, 3, 4]);
}

/// 相依已完成的後到者:B 到達時 A 早就做完,必須立刻可派(不能等一個永遠不會再來的事件)。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn dep_already_completed() {
    let mut bus = SimBus::new().job_at(0, 1, 5, &[]).job_at(5, 2, 5, &[1]);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![1, 2]);
}
