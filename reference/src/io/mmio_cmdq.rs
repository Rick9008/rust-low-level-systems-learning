// ⚠⚠ 防雷:本檔是 sim l(MMIO command queue)的解法。計時場排在 8/2——
// 在那之前不要讀本檔(含註解與測試)。自定規則:跑題前不開 oracle/sol。

//! # mmio_cmdq —— MMIO command queue:doorbell + completion ring(sim l 教學版)
//!
//! ## [Clarify]
//! 題幹:`docs/interviews/sim-problems.md` sim l(彩排 harness:`rehearsals/src/sim_l_mmio.rs`)。
//! 一句話:**這是一條 SPSC ring,只是消費者是硬體**。隱藏 spec:
//! - device 是另一個「讀者」,它看不見你的程式序——沒柵欄它可能讀到寫一半的 slot;
//! - `SubmitHead`/`CompTail` 是唯讀(device 的進度),`Doorbell` 唯寫且只能前進;
//! - ring 滿了 `Err(Full)` 立刻回,不等——backpressure 是呼叫端的事;
//! - Phase 2:completion 亂序回來,靠 descriptor 自帶的 tag 路由。
//!
//! ## [Abstract]
//! driver 端 state 只要**兩個單調序號 + 容量**:
//! `tail`(我提交到哪)、`comp_head`(我收完成收到哪)。
//! 滿的判定用序號差 `tail - head == cap`(head 讀 `SubmitHead` 暫存器),
//! 不用 `%` 比較——滿/空不混淆,head/tail 座標系的老朋友
//! (同 [`crate::ds::ring_buffer`] 的口訣,這裡跨到硬體邊界)。
//!
//! ## [Iterate]
//! V0 in-order:completion 按提交序回,`comp_head` 順序掃 → Phase 2 lifo:
//! 同一段程式碼**不用改**——tag 在 descriptor 裡,`on_done(tag, payload)` 天生
//! 路由;真硬體 completion 只回 tag 時才需要 driver 自建 `tag → cmd` 表(講出差異)。
//!
//! ## [Trade-offs]
//! - **submit 鐵律:填 descriptor → `barrier()` → 敲 doorbell**。柵欄不是為了
//!   編譯器好看——MMIO 寫入與普通記憶體寫入在匯流排上可以亂序,device 收到
//!   doorbell 時 descriptor 可能還在路上。這是 store-release 的硬體版。
//! - 每筆一次 barrier+doorbell vs 批次(填 N 筆 → 一次柵欄 → doorbell 一次跳 N):
//!   doorbell 是 MMIO 寫 = uncached、貴(~百 ns 到 µs);高吞吐走批次。
//!   本實作每筆一敲,是教學形狀(規則先站穩再省)。
//! - poll 順序:先讀 `CompTail` 再讀 slot——方向相反的同一條鐵律(load-acquire
//!   的硬體版):先知道 device 寫到哪,才准碰那之前的 slot。
//!
//! ## [Dry-Run]
//! 測試 `full_then_backpressure_then_reuse` 的手 trace(cap=2):
//! submit(1)、submit(2) 佔滿 → submit(3) 讀 SubmitHead=0,tail−head=2==cap →
//! `Err(Full)` 不動任何 state → device step 消費 1 筆 → poll 收 1 筆 →
//! submit(3) 此時 head=2,tail−head=1 < 2 → 進 slot(2%2=0,wrap 重用)→ 全收齊。
//!
//! 對照:彩排解答 `rehearsals/examples/sol_sim_l_mmio.rs`(同設計,單檔面試版)。

use std::collections::{HashSet, VecDeque};

// ===================== 題目給的介面(與 sim l 相同,英文保留)=====================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Descriptor {
    pub tag: u32,
    pub payload: u64,
}

#[derive(Clone, Copy, Debug)]
pub enum Reg {
    /// Submission index the device has consumed up to (read-only).
    SubmitHead,
    /// Write the new submission tail here to notify the device (write-only).
    Doorbell,
    /// Completion index the device has written up to (read-only).
    CompTail,
}

/// Returned when the submission ring is full — backpressure is the caller's problem.
#[derive(Debug, PartialEq, Eq)]
pub struct Full;

/// The simulated accelerator. `step()` = one device tick (consume
/// submissions, post one completion). 內建協定 oracle:descriptor 寫入後、
/// 敲 doorbell 前沒放 `barrier()` **當場 panic**——device 看不見你的程式序。
pub struct Device {
    cap: usize,
    submit_slots: Vec<Option<Descriptor>>,
    comp_slots: Vec<Option<Descriptor>>,
    submit_head: u64,
    doorbell: u64,
    comp_tail: u64,
    comp_consumed: u64,
    unbarriered: HashSet<usize>,
    pending: VecDeque<Descriptor>,
    lifo: bool,
}

impl Device {
    pub fn new(cap: usize) -> Self {
        Self {
            cap,
            submit_slots: vec![None; cap],
            comp_slots: vec![None; cap],
            submit_head: 0,
            doorbell: 0,
            comp_tail: 0,
            comp_consumed: 0,
            unbarriered: HashSet::new(),
            pending: VecDeque::new(),
            lifo: false,
        }
    }

    /// Phase 2: out-of-order completions (last submitted finishes first).
    pub fn lifo(mut self) -> Self {
        self.lifo = true;
        self
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn mmio_read(&self, reg: Reg) -> u64 {
        match reg {
            Reg::SubmitHead => self.submit_head,
            Reg::Doorbell => self.doorbell,
            Reg::CompTail => self.comp_tail,
        }
    }

    pub fn mmio_write(&mut self, reg: Reg, val: u64) {
        match reg {
            Reg::Doorbell => {
                assert!(
                    self.unbarriered.is_empty(),
                    "敲 doorbell 前必須 barrier():device 可能讀到寫一半的 descriptor"
                );
                assert!(val >= self.doorbell, "doorbell 只能前進");
                assert!(
                    val - self.submit_head <= self.cap as u64,
                    "doorbell 超出 ring 容量:未消費區間塞不下"
                );
                self.doorbell = val;
            }
            _ => panic!("{reg:?} 是唯讀暫存器"),
        }
    }

    /// Write a descriptor into submission slot `idx` (you compute idx = seq % cap).
    pub fn slot_write(&mut self, idx: usize, d: Descriptor) {
        assert!(idx < self.cap, "slot index 越界");
        self.submit_slots[idx] = Some(d);
        self.unbarriered.insert(idx);
    }

    /// Take the descriptor out of completion slot `idx` (destructive read = consumed).
    pub fn comp_slot_read(&mut self, idx: usize) -> Descriptor {
        assert!(idx < self.cap, "slot index 越界");
        self.comp_consumed += 1;
        self.comp_slots[idx]
            .take()
            .expect("completion slot 是空的:讀過頭了")
    }

    /// Memory barrier toward the device — only after this are your earlier
    /// `slot_write`s guaranteed visible to it.
    pub fn barrier(&mut self) {
        self.unbarriered.clear();
    }

    /// One device tick; returns whether it did anything (for test loops).
    pub fn step(&mut self) -> bool {
        let mut worked = false;
        // 一口氣消費 doorbell 以前的所有 submission(真硬體就是這樣 slurp 的)。
        while self.submit_head < self.doorbell {
            let idx = (self.submit_head % self.cap as u64) as usize;
            let d = self.submit_slots[idx]
                .take()
                .expect("device 讀到空 slot:doorbell 敲過頭");
            self.pending.push_back(d);
            self.submit_head += 1;
            worked = true;
        }
        // 完成一個。
        let done = if self.lifo {
            self.pending.pop_back()
        } else {
            self.pending.pop_front()
        };
        if let Some(d) = done {
            assert!(
                self.comp_tail - self.comp_consumed < self.cap as u64,
                "completion ring 滿了還在寫:driver 沒在收"
            );
            let idx = (self.comp_tail % self.cap as u64) as usize;
            assert!(self.comp_slots[idx].is_none(), "completion 覆蓋未收的完成");
            self.comp_slots[idx] = Some(d);
            self.comp_tail += 1;
            worked = true;
        }
        worked
    }
}

// ===================== 實作 =====================

/// driver 端 state:兩個單調序號 + 容量,一個欄位都不多。
/// `tail` = 我提交到哪;`comp_head` = 我收完成收到哪。
/// head 不用存——它活在 device 的 `SubmitHead` 暫存器裡,要用就讀。
pub struct Driver {
    cap: u64,
    tail: u64,
    comp_head: u64,
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

/// Submit one command. On a full ring return `Err(Full)` immediately — never wait.
/// 鐵律順序:滿判定(序號差)→ 填 slot → `barrier()` → tail 前進 → 敲 doorbell。
pub fn submit(dev: &mut Device, drv: &mut Driver, tag: u32, payload: u64) -> Result<(), Full> {
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

/// Collect completions, invoking `on_done(tag, payload)` per entry.
/// 方向相反的同一條鐵律:**先讀 `CompTail`**(device 寫到哪),再碰那之前的 slot。
/// 亂序(Phase 2)不用改碼——tag 在 descriptor 裡,天生路由。
pub fn poll_completions(dev: &mut Device, drv: &mut Driver, on_done: &mut dyn FnMut(u32, u64)) {
    let tail = dev.mmio_read(Reg::CompTail);
    while drv.comp_head < tail {
        let idx = (drv.comp_head % drv.cap) as usize;
        let d = dev.comp_slot_read(idx);
        on_done(d.tag, d.payload);
        drv.comp_head += 1;
    }
}

// ===================== 測試 =====================

#[cfg(test)]
mod tests {
    use super::*;

    /// in-order 基本流手 trace(cap=8):submit(1,100)→slot0、barrier、doorbell=1;
    /// (2,200)→slot1、doorbell=2;(3,300)→slot2、doorbell=3 → device step 到沒事做
    /// (消費 3 筆、完成 3 筆)→ poll:CompTail=3,comp_head 0→3 依序收
    /// (1,100)(2,200)(3,300)。
    #[test]
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

    /// 檔頭 [Dry-Run] 的劇本:滿載 backpressure + slot wrap 重用(cap=2)。
    /// Err(Full) 必須不動任何 state——之後重試要能成功。
    #[test]
    fn full_then_backpressure_then_reuse() {
        let mut dev = Device::new(2);
        let mut drv = Driver::new(2);
        submit(&mut dev, &mut drv, 1, 10).unwrap();
        submit(&mut dev, &mut drv, 2, 20).unwrap();
        assert_eq!(submit(&mut dev, &mut drv, 3, 30), Err(Full));
        dev.step();
        let mut got = Vec::new();
        poll_completions(&mut dev, &mut drv, &mut |tag, p| got.push((tag, p)));
        submit(&mut dev, &mut drv, 3, 30).unwrap(); // wrap:seq 2 → slot 0
        while dev.step() {}
        poll_completions(&mut dev, &mut drv, &mut |tag, p| got.push((tag, p)));
        let mut tags: Vec<_> = got.iter().map(|&(t, _)| t).collect();
        tags.sort_unstable();
        assert_eq!(tags, vec![1, 2, 3]);
    }

    /// Phase 2 亂序 completion(lifo):同一段 driver 碼不改,tag 自帶路由——
    /// 每筆 (tag, payload) 配對仍正確,順序不保證(這題順序本來就不是 contract)。
    #[test]
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

    /// 鐵律的反例示範:填了 slot、沒 barrier 就敲 doorbell → device oracle 當場抓。
    /// (這條測的是「教訓」本身:沒有柵欄,device 可能讀到寫一半的 descriptor。)
    #[test]
    #[should_panic(expected = "敲 doorbell 前必須 barrier()")]
    fn doorbell_without_barrier_is_caught() {
        let mut dev = Device::new(4);
        dev.slot_write(0, Descriptor { tag: 1, payload: 1 });
        dev.mmio_write(Reg::Doorbell, 1); // 少了 dev.barrier() —— panic
    }

    /// 連續 wrap 壓力(cap=2,5 輪 submit→step→poll):序號單調、slot 反覆重用,
    /// doorbell 只前進;每輪收回自己的 tag。
    #[test]
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
