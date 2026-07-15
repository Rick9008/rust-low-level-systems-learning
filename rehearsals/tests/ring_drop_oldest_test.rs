//! 參考測試:ring_drop_oldest。
//!
//! 彩排時先自己寫測試(寫在 src/ring_drop_oldest.rs 底部);自己的測試轉綠後
//! 才跑這組對照,看漏了哪一類邊界:
//! `cargo test -p rehearsals --test ring_drop_oldest_test -- --include-ignored`

use rehearsals::ring_drop_oldest::{channel, SensorRing};
use std::thread;

/// boundary:空 buffer——pop 回 None、len/is_empty/dropped 全零。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn empty_ring() {
    let mut r = SensorRing::new(4);
    assert_eq!(r.pop(), None);
    assert_eq!(r.len(), 0);
    assert!(r.is_empty());
    assert_eq!(r.dropped(), 0);
}

/// boundary:單元素容量——第二次 push 就觸發 drop-oldest,留下的是新值。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn capacity_one_overwrites_oldest() {
    let mut r = SensorRing::new(1);
    r.push(1);
    assert_eq!(r.len(), 1);
    r.push(2); // 滿:丟 1、收 2
    assert_eq!(r.dropped(), 1);
    assert_eq!(r.pop(), Some(2));
    assert_eq!(r.pop(), None);
    assert_eq!(r.dropped(), 1); // pop 不影響計數
}

/// boundary:滿時 drop 計數——容量恰好是上限(不許偷偷上取 2 的冪)。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn full_drops_oldest_and_counts() {
    let mut r = SensorRing::new(3);
    for v in 1..=5 {
        r.push(v); // 4 丟 1、5 丟 2
    }
    assert_eq!(r.dropped(), 2);
    assert_eq!(r.len(), 3);
    assert_eq!(r.pop(), Some(3));
    assert_eq!(r.pop(), Some(4));
    assert_eq!(r.pop(), Some(5));
    assert!(r.is_empty());
}

/// boundary:wrap 跨界——push/pop 交錯讓實體索引繞回開頭,
/// 之後的 FIFO 順序與 drop 計數都必須不受影響。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn wrap_across_boundary() {
    let mut r = SensorRing::new(4);
    for v in 1..=4 {
        r.push(v); // 滿
    }
    assert_eq!(r.pop(), Some(1));
    assert_eq!(r.pop(), Some(2));
    r.push(5); // 這兩筆落在繞回去的位置
    r.push(6);
    assert_eq!(r.dropped(), 0); // 有空位,不該丟
    for expect in 3..=6 {
        assert_eq!(r.pop(), Some(expect));
    }
    assert!(r.is_empty());

    // 從 wrap 過的起點再灌爆一次:drop 行為在 wrap 後照常。
    for v in 7..=11 {
        r.push(v); // 11 丟 7
    }
    assert_eq!(r.dropped(), 1);
    for expect in 8..=11 {
        assert_eq!(r.pop(), Some(expect));
    }
}

/// Part 2(SPSC):一產一消並發跑。不變量:
/// 消費序列嚴格遞增(FIFO + drop-oldest 不重排)、最後一筆必到
/// (沒有 push 跟在它後面,不可能被丟)、收到數 + 丟棄數 = 總 push 數。
#[test]
#[ignore = "參考測試:彩排完成後再開"]
fn spsc_concurrent_no_loss_no_reorder() {
    const N: u32 = 10_000;
    let (mut tx, mut rx) = channel(64);

    let producer = thread::spawn(move || {
        for i in 0..N {
            tx.push(i);
        }
        tx.dropped()
    });

    let mut got = Vec::new();
    loop {
        match rx.pop() {
            Some(v) => {
                got.push(v);
                if v == N - 1 {
                    break; // 最後一筆 push 的值,FIFO 下它彈出時 ring 已空
                }
            }
            None => thread::yield_now(),
        }
    }
    let dropped = producer.join().unwrap();

    assert!(got.windows(2).all(|w| w[0] < w[1]), "消費序列必須嚴格遞增");
    assert_eq!(*got.last().unwrap(), N - 1);
    assert_eq!(got.len() as u64 + dropped, u64::from(N));
}
