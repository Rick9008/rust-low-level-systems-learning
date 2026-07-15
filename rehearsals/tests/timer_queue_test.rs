//! 參考測試:timer_queue。
//!
//! 彩排時先自己寫測試;轉綠後才跑這組:
//! `cargo test -p rehearsals --test timer_queue_test -- --include-ignored`

use rehearsals::timer_queue::TimerQueue;

/// boundary:空 queue——next_deadline None、pop_due 空、不 panic。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn empty_queue() {
    let mut q = TimerQueue::new();
    assert_eq!(q.next_deadline(), None);
    assert_eq!(q.pop_due(1_000), Vec::<u64>::new());
    assert!(q.is_empty());
}

/// boundary:同一 deadline 的 tie-break——依 (deadline, id) 排序;
/// next_deadline 回的是最早的那個(park 目標)。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn ordering_and_tie_break() {
    let mut q = TimerQueue::new();
    q.schedule(2, 100, 1_000);
    q.schedule(1, 100, 1_000); // 與 id 2 同 deadline
    q.schedule(3, 50, 1_000);
    assert_eq!(q.next_deadline(), Some(50));
    assert_eq!(q.pop_due(100), vec![3, 1, 2]); // deadline 序;同 deadline 依 id
    assert_eq!(q.len(), 3, "週期任務收割後仍在排程中");
}

/// 週期重排不飄移:下一個 deadline = 舊 deadline + interval,不是 now + interval。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn periodic_reschedule_no_drift() {
    let mut q = TimerQueue::new();
    q.schedule(7, 100, 100);
    assert_eq!(q.pop_due(100), vec![7]);
    assert_eq!(q.next_deadline(), Some(200));
    assert_eq!(q.pop_due(250), vec![7], "晚了 50ms 才收割");
    assert_eq!(
        q.next_deadline(),
        Some(300),
        "從舊 deadline 起算 → 300,不是 350"
    );
}

/// boundary:now 落後很多——錯過的週期在同一次 pop_due 裡補發。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn catch_up_fires_missed_periods() {
    let mut q = TimerQueue::new();
    q.schedule(9, 100, 100);
    assert_eq!(q.pop_due(399), vec![9, 9, 9], "100/200/300 三次都要補");
    assert_eq!(q.next_deadline(), Some(400));
}

/// boundary:deadline 已在過去——排進去就立刻到期。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn deadline_in_past_fires_immediately() {
    let mut q = TimerQueue::new();
    q.schedule(5, 0, 1_000);
    assert_eq!(q.pop_due(0), vec![5], "deadline <= now 就要觸發");
    assert_eq!(q.next_deadline(), Some(1_000));
}

/// 多 timer 不同 interval 交錯:收割順序照合併後的 (deadline, id)。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn interleaved_intervals() {
    let mut q = TimerQueue::new();
    q.schedule(1, 100, 100); // 100, 200, 300 ⋯
    q.schedule(2, 150, 300); // 150, 450 ⋯
    assert_eq!(q.pop_due(300), vec![1, 2, 1, 1]); // 100(1), 150(2), 200(1), 300(1)
    assert_eq!(q.next_deadline(), Some(400)); // id1 下次 400;id2 下次 450
}
