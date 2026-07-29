//! sim l —— MMIO command queue:doorbell + completion ring(題幹:`docs/interviews/sim-problems.md`)。
//!
//! 彩排規則同 sim_i:實作+自寫測試在本檔;`tests/sim_l_mmio_test.rs` 跑完才開。
//! Device 內建協定 oracle:**descriptor 寫入後、敲 doorbell 前沒放 `barrier()` 會當場 panic**
//! ——這正是本題的最大看點(device 是另一個「讀者」,它看不見你程式序)。

use std::collections::{HashSet, VecDeque};

// ===================== 題目給的介面(可讀)=====================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Descriptor {
    pub tag: u32,
    pub payload: u64,
}

#[derive(Clone, Copy, Debug)]
pub enum Reg {
    /// device 已消費到的 submission index(唯讀)。
    SubmitHead,
    /// 你寫入新的 submission tail 通知 device(唯寫)。
    Doorbell,
    /// device 已寫到的 completion index(唯讀)。
    CompTail,
}

/// 「回傳給呼叫端處理」的滿載錯誤。
#[derive(Debug, PartialEq, Eq)]
pub struct Full;

/// 模擬的加速器。`step()` = device 動一拍(消費 submission、產生一個 completion)。
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

    /// Phase 2:completion 亂序(後進先完)。
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

    /// 把 descriptor 寫進 submission ring 第 `idx` 槽(idx = 序號 % cap 由你算)。
    pub fn slot_write(&mut self, idx: usize, d: Descriptor) {
        assert!(idx < self.cap, "slot index 越界");
        self.submit_slots[idx] = Some(d);
        self.unbarriered.insert(idx);
    }

    /// 讀走 completion ring 第 `idx` 槽(破壞性讀 = 你收走了)。
    pub fn comp_slot_read(&mut self, idx: usize) -> Descriptor {
        assert!(idx < self.cap, "slot index 越界");
        self.comp_consumed += 1;
        self.comp_slots[idx]
            .take()
            .expect("completion slot 是空的:讀過頭了")
    }

    /// 對 CPU 的寫入立柵欄——之後 device 才保證看得見前面的 slot_write。
    pub fn barrier(&mut self) {
        self.unbarriered.clear();
    }

    /// device 動一拍;回傳這拍有沒有做事(測試迴圈用)。
    pub fn step(&mut self) -> bool {
        let mut worked = false;
        // 一口氣消費 doorbell 以前的所有 submission(真硬體就是這樣slurp的)。
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

// ===================== 作答區 =====================

/// driver 端 state 由你設計(欄位彩排時自己加)。
pub struct Driver {
    // 你的欄位
}

impl Driver {
    pub fn new(cap: usize) -> Self {
        todo!("彩排時實作")
    }
}

/// 提交一個命令。ring 滿 → `Err(Full)` 立刻回,不等。
pub fn submit(dev: &mut Device, drv: &mut Driver, tag: u32, payload: u64) -> Result<(), Full> {
    todo!("彩排時實作")
}

/// 收 completion,逐筆回呼 `on_done(tag, payload)`。Phase 2 completion 會亂序。
pub fn poll_completions(dev: &mut Device, drv: &mut Driver, on_done: &mut dyn FnMut(u32, u64)) {
    todo!("彩排時實作")
}
