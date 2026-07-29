//! solution:sim l —— MMIO command queue(doorbell + completion ring)。**寫完彩排才開。**
//!
//! 本題一句話:**這是一條 SPSC ring,只是消費者是硬體**。三個要點:
//! 1. submit 鐵律:填 descriptor → `barrier()` → 才敲 doorbell。device 是另一個「讀者」,
//!    它看不見你的程式序——沒柵欄它可能讀到寫一半的 slot(Device oracle 會當場抓)。
//! 2. 滿的判定:`tail - head == cap`(head 從 SubmitHead 暫存器讀,是 device 消費進度);
//!    用序號差不用 % 比較,滿/空不混淆——head/tail 座標系的老朋友。
//! 3. completion 側方向相反:先讀 CompTail(device 的寫進度),再讀 slot 內容;
//!    亂序靠 descriptor 自帶的 tag 路由,driver 不需要 in-flight 表(completion 回帶全 payload;
//!    真硬體只回 tag 時就要自己記 `tag → cmd`——講出這個差異)。
//!
//! 驗證:`cargo run -p rehearsals --example sol_sim_l_mmio`

use rehearsals::sim_l_mmio::{Descriptor, Device, Full, Reg};

/// driver 端 state:兩個單調序號 + 容量。tail = 我提交到哪;comp_head = 我收完成收到哪。
struct SolDriver {
    cap: u64,
    tail: u64,
    comp_head: u64,
}

impl SolDriver {
    fn new(cap: usize) -> Self {
        Self {
            cap: cap as u64,
            tail: 0,
            comp_head: 0,
        }
    }
}

fn submit(dev: &mut Device, drv: &mut SolDriver, tag: u32, payload: u64) -> Result<(), Full> {
    let head = dev.mmio_read(Reg::SubmitHead);
    if drv.tail - head == drv.cap {
        return Err(Full); // 立刻回,不等——backpressure 是呼叫端的事
    }
    let idx = (drv.tail % drv.cap) as usize;
    dev.slot_write(idx, Descriptor { tag, payload });
    dev.barrier(); // 鐵律:填完 → 柵欄 → 敲鈴
    drv.tail += 1;
    dev.mmio_write(Reg::Doorbell, drv.tail);
    Ok(())
}

fn poll_completions(dev: &mut Device, drv: &mut SolDriver, on_done: &mut dyn FnMut(u32, u64)) {
    let tail = dev.mmio_read(Reg::CompTail); // 先看 device 寫到哪(acquire 方向)
    while drv.comp_head < tail {
        let idx = (drv.comp_head % drv.cap) as usize;
        let d = dev.comp_slot_read(idx);
        on_done(d.tag, d.payload);
        drv.comp_head += 1;
    }
}

fn main() {
    // scenario 1:in-order 基本流。
    let mut dev = Device::new(8);
    let mut drv = SolDriver::new(8);
    for tag in 1..=3u32 {
        submit(&mut dev, &mut drv, tag, u64::from(tag) * 100).unwrap();
    }
    while dev.step() {}
    let mut got = Vec::new();
    poll_completions(&mut dev, &mut drv, &mut |tag, p| got.push((tag, p)));
    assert_eq!(got, vec![(1, 100), (2, 200), (3, 300)]);

    // scenario 2:滿載 backpressure。
    let mut dev = Device::new(2);
    let mut drv = SolDriver::new(2);
    submit(&mut dev, &mut drv, 1, 10).unwrap();
    submit(&mut dev, &mut drv, 2, 20).unwrap();
    assert_eq!(submit(&mut dev, &mut drv, 3, 30), Err(Full));
    dev.step();
    let mut got = Vec::new();
    poll_completions(&mut dev, &mut drv, &mut |tag, p| got.push((tag, p)));
    submit(&mut dev, &mut drv, 3, 30).unwrap();
    while dev.step() {}
    poll_completions(&mut dev, &mut drv, &mut |tag, p| got.push((tag, p)));
    let mut tags: Vec<_> = got.iter().map(|&(t, _)| t).collect();
    tags.sort_unstable();
    assert_eq!(tags, vec![1, 2, 3]);

    // scenario 3:Phase 2 亂序 completion,tag 路由。
    let mut dev = Device::new(8).lifo();
    let mut drv = SolDriver::new(8);
    for tag in 1..=4u32 {
        submit(&mut dev, &mut drv, tag, u64::from(tag) * 7).unwrap();
    }
    while dev.step() {}
    let mut got = Vec::new();
    poll_completions(&mut dev, &mut drv, &mut |tag, p| got.push((tag, p)));
    assert_eq!(got.len(), 4);
    for &(tag, p) in &got {
        assert_eq!(p, u64::from(tag) * 7);
    }

    println!("sol_sim_l_mmio: all green");
}
