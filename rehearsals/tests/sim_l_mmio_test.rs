//! 參考測試:sim l(MMIO command queue)。
//!
//! 彩排完才開:
//! `cargo test -p rehearsals --test sim_l_mmio_test -- --include-ignored`
//!
//! 注意:barrier 協定不用在這裡測——Device 的 oracle 會在你漏 barrier 的瞬間 panic,
//! 所以任何一個測試能跑完,協定就是對的。

use rehearsals::sim_l_mmio::{Device, Driver, Full, poll_completions, submit};

/// 基本流:submit 3 → device 動 → completion 依序回、payload 對上 tag。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn inorder_roundtrip() {
    let mut dev = Device::new(8);
    let mut drv = Driver::new(8);
    for tag in 1..=3u32 {
        submit(&mut dev, &mut drv, tag, u64::from(tag) * 100).unwrap();
    }
    while dev.step() {}
    let mut got = Vec::new();
    poll_completions(&mut dev, &mut drv, &mut |tag, p| got.push((tag, p)));
    assert_eq!(got, vec![(1, 100), (2, 200), (3, 300)]);
}

/// ring 滿:cap 2 塞第 3 個必須立刻 Err(Full);device 消化後再 submit 要成功。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn full_ring_backpressure() {
    let mut dev = Device::new(2);
    let mut drv = Driver::new(2);
    submit(&mut dev, &mut drv, 1, 10).unwrap();
    submit(&mut dev, &mut drv, 2, 20).unwrap();
    assert_eq!(
        submit(&mut dev, &mut drv, 3, 30),
        Err(Full),
        "滿了必須立刻回 Err"
    );
    dev.step(); // device 消費 + 完成一個
    let mut got = Vec::new();
    poll_completions(&mut dev, &mut drv, &mut |tag, p| got.push((tag, p)));
    submit(&mut dev, &mut drv, 3, 30).expect("空間釋出後要能再收");
    while dev.step() {}
    poll_completions(&mut dev, &mut drv, &mut |tag, p| got.push((tag, p)));
    let mut tags: Vec<_> = got.iter().map(|&(t, _)| t).collect();
    tags.sort_unstable();
    assert_eq!(tags, vec![1, 2, 3]);
}

/// Phase 2 亂序:lifo device——completion 順序反轉,路由靠 tag,payload 不能張冠李戴。
#[test]
#[ignore = "sim 參考測試:跑完彩排才開"]
fn out_of_order_completions_route_by_tag() {
    let mut dev = Device::new(8).lifo();
    let mut drv = Driver::new(8);
    for tag in 1..=4u32 {
        submit(&mut dev, &mut drv, tag, u64::from(tag) * 7).unwrap();
    }
    while dev.step() {}
    let mut got = Vec::new();
    poll_completions(&mut dev, &mut drv, &mut |tag, p| got.push((tag, p)));
    assert_eq!(got.len(), 4);
    for &(tag, p) in &got {
        assert_eq!(p, u64::from(tag) * 7, "tag {tag} 的 payload 路由錯了");
    }
    assert_ne!(
        got.iter().map(|&(t, _)| t).collect::<Vec<_>>(),
        vec![1, 2, 3, 4],
        "lifo device 下 completion 不該按提交序回來(否則你排序了?)"
    );
}
