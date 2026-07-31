// ⚠⚠ 防雷:本檔是 sim l(MMIO command queue)的填空版,spec 註解含解法方向。
// 本檔就是 8/2 lite 場材料(7/31 改制):開跑即用、開跑前不要讀。自定規則:跑題前不開 oracle/sol。

//! drill:mmio_cmdq —— 填 submit 與 poll_completions(sim l 的填空版)。
//!
//! 已給:`Device` / `Descriptor` / `Reg` / `Full`(借 reference 的,題目給定件)、
//! `Driver` 的欄位設計(兩個單調序號 + 容量)。
//! 要填:`submit` / `poll_completions` 兩個函式。
//!
//! 核心不變量:
//! - **鐵律:填 descriptor → `barrier()` → 才敲 doorbell**——device 是另一個
//!   讀者,看不見你的程式序(Device oracle 抓違規,當場 panic);
//! - 滿判定用序號差 `tail - head == cap`(head 讀 `SubmitHead`),不用 `%` 比較;
//! - 滿了 `Err(Full)` 立刻回、**不動任何 state**——之後重試要能成功;
//! - poll 方向相反的同一條鐵律:先讀 `CompTail`,再碰那之前的 slot。
//!
//! 設計取捨見 reference 同名模組檔頭(**跑過 sim l 之後**再讀)。

use reference::io::mmio_cmdq::{Descriptor, Device, Full, Reg};

/// driver 端 state(已給):`tail` = 我提交到哪;`comp_head` = 我收完成收到哪。
/// head 不用存——它活在 device 的 `SubmitHead` 暫存器裡,要用就讀。
pub struct Driver {
    pub cap: u64,
    pub tail: u64,
    pub comp_head: u64,
}

impl Driver {
    pub fn new(cap: usize) -> Self {
        Self {
            cap: cap as u64,
            tail: 0,
            comp_head: 0,
        }
    }
}

/// spec:提交一筆。讀 `SubmitHead` → 序號差滿判定(滿 → `Err(Full)`,啥都不動)
/// → `slot_write(tail % cap, ..)` → `barrier()` → `tail += 1` → 敲 `Doorbell`(寫新 tail)。
/// 順序錯一步,oracle 就 panic。
pub fn submit(dev: &mut Device, drv: &mut Driver, tag: u32, payload: u64) -> Result<(), Full> {
    todo!("spec: 滿判定(序號差)→ 填 slot → barrier → 敲 doorbell")
}

/// spec:收完成。先讀 `CompTail`(device 寫到哪)→ `comp_head` 追到 tail 為止,
/// 逐 slot `comp_slot_read(comp_head % cap)` → `on_done(tag, payload)`。
/// 亂序(Phase 2)不用改碼——tag 在 descriptor 裡,天生路由。
pub fn poll_completions(dev: &mut Device, drv: &mut Driver, on_done: &mut dyn FnMut(u32, u64)) {
    todo!("spec: 先讀 CompTail 再收 slot;comp_head 單調前進")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// in-order 基本流:3 筆依序提交、device 跑完、poll 依序收回。
    #[test]
    #[ignore = "drill:填完 submit/poll_completions 後拔掉"]
    fn in_order_submit_and_complete() {
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

    /// 滿載 backpressure + wrap 重用(cap=2):Err(Full) 不動 state,重試要成功。
    #[test]
    #[ignore = "drill:填完後拔掉"]
    fn full_then_backpressure_then_reuse() {
        let mut dev = Device::new(2);
        let mut drv = Driver::new(2);
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
    }

    /// Phase 2 亂序 completion(lifo):(tag, payload) 配對仍正確,順序不是 contract。
    #[test]
    #[ignore = "drill:填完後拔掉"]
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
    }

    /// 連續 wrap(cap=2,5 輪):序號單調、slot 反覆重用、doorbell 只前進。
    #[test]
    #[ignore = "drill:填完後拔掉"]
    fn sustained_wrap_around() {
        let mut dev = Device::new(2);
        let mut drv = Driver::new(2);
        for tag in 1..=5u32 {
            submit(&mut dev, &mut drv, tag, u64::from(tag)).unwrap();
            while dev.step() {}
            let mut got = Vec::new();
            poll_completions(&mut dev, &mut drv, &mut |t, p| got.push((t, p)));
            assert_eq!(got, vec![(tag, u64::from(tag))]);
        }
    }
}
