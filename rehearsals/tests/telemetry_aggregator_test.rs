//! 參考測試:telemetry_aggregator。
//!
//! 彩排時先自己寫測試;轉綠後才跑這組:
//! `cargo test -p rehearsals --test telemetry_aggregator_test -- --include-ignored`

use rehearsals::telemetry_aggregator::{Aggregator, WindowStats};

/// 聚合數學:count/sum/min/max(含負值)。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn aggregation_math() {
    let mut a = Aggregator::new(100, 4);
    assert!(a.record(10, 5));
    assert!(a.record(20, -3));
    assert!(a.record(30, 9));
    assert_eq!(
        a.stats(50),
        Some(WindowStats {
            count: 3,
            sum: 11,
            min: -3,
            max: 9
        })
    );
}

/// boundary:ts 正好落在 window 邊界——半開區間 [k*w, (k+1)*w)。
/// ts=99 屬 window 0,ts=100 屬 window 1。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn window_boundary_is_half_open() {
    let mut a = Aggregator::new(100, 4);
    assert!(a.record(99, 1));
    assert!(a.record(100, 2));
    assert_eq!(a.stats(99).unwrap().count, 1);
    assert_eq!(a.stats(100).unwrap().count, 1);
    assert_eq!(a.stats(99).unwrap().sum, 1);
    assert_eq!(a.stats(100).unwrap().sum, 2);
}

/// boundary:空 window——在保留範圍內但沒資料 → None。
/// (zero-init 陷阱:空 window 的 min/max 絕不能是 0。)
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn empty_window_is_none_not_zero() {
    let mut a = Aggregator::new(100, 4);
    assert!(a.record(50, 7)); // window 0
    assert!(a.record(350, 8)); // window 3;window 1、2 空
    assert_eq!(a.stats(150), None);
    assert_eq!(a.stats(250), None);
    assert_eq!(a.stats(50).unwrap().max, 7);
}

/// boundary:ts 跳很遠的未來——被跳過的 window 必須清掉,
/// slot 重用時不能讀到上一輪的舊資料(這題最漂亮的 bug)。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn far_future_jump_clears_skipped_windows() {
    let mut a = Aggregator::new(100, 4);
    assert!(a.record(50, 1)); // window 0(slot 0)
    assert!(a.record(150, 2)); // window 1(slot 1)
    // 跳到 window 5(slot 5 % 4 = 1,重用 window 1 的 slot)
    assert!(a.record(550, 30));
    assert_eq!(
        a.stats(550),
        Some(WindowStats {
            count: 1,
            sum: 30,
            min: 30,
            max: 30
        }),
        "slot 重用必須是乾淨的,不能混到 window 1 的舊值"
    );
    assert_eq!(a.stats(150), None, "window 1 已被淘汰");
    assert_eq!(a.stats(50), None, "window 0 超出保留範圍(5-4+1=2 起)");
    // 中間被跳過、slot 沒被重用的 window 也必須是空的
    assert_eq!(a.stats(450), None, "window 4 被跳過,視同空");
}

/// boundary:ts 落在已淘汰的過去 → record 拒絕(false)且不汙染任何 window;
/// 仍在保留範圍內的過去 → 接受。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn too_old_rejected_in_range_accepted() {
    let mut a = Aggregator::new(100, 4);
    assert!(a.record(550, 1)); // 最新 = window 5;保留 window 2..=5
    assert!(!a.record(150, 9), "window 1 已淘汰 → 拒絕");
    assert_eq!(a.stats(150), None);
    assert!(
        a.record(250, 4),
        "window 2 還在保留範圍 → 接受(亂序但沒太舊)"
    );
    assert_eq!(a.stats(250).unwrap().sum, 4);
    assert_eq!(a.stats(550).unwrap().count, 1, "舊資料不得汙染最新 window");
}
