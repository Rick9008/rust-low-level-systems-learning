//! rehearsal a:ring_drop_oldest —— 題目見 rehearsals/README.md。
//!
//! 只給 API 簽名。結構自己設計(佔位欄位整個換掉);你自己的測試寫在本檔底部
//! `#[cfg(test)] mod tests`(模擬 CoderPad 單檔)。

// —— Part 1:單執行緒版 ——

pub struct SensorRing {
    // ↓ 佔位:動手時整個換成你的設計。
    _todo: (),
}

impl SensorRing {
    /// 恰好容納 `capacity` 筆;`capacity >= 1`。
    pub fn new(capacity: usize) -> Self {
        todo!("rehearsal")
    }

    /// 永遠成功;滿時丟最舊的一筆並計數。
    pub fn push(&mut self, value: u32) {
        todo!("rehearsal")
    }

    /// FIFO。
    pub fn pop(&mut self) -> Option<u32> {
        todo!("rehearsal")
    }

    pub fn len(&self) -> usize {
        todo!("rehearsal")
    }

    pub fn is_empty(&self) -> bool {
        todo!("rehearsal")
    }

    /// 累計被丟棄的筆數。
    pub fn dropped(&self) -> u64 {
        todo!("rehearsal")
    }
}

// —— Part 2:SPSC 版(恰好一個 producer 執行緒、一個 consumer 執行緒)——

pub struct Producer {
    _todo: (),
}

pub struct Consumer {
    _todo: (),
}

pub fn channel(capacity: usize) -> (Producer, Consumer) {
    todo!("rehearsal")
}

impl Producer {
    /// 永遠成功;滿時丟最舊的一筆並計數。
    pub fn push(&mut self, value: u32) {
        todo!("rehearsal")
    }

    /// 累計被丟棄的筆數。
    pub fn dropped(&self) -> u64 {
        todo!("rehearsal")
    }
}

impl Consumer {
    /// FIFO;空 → None(不 block)。
    pub fn pop(&mut self) -> Option<u32> {
        todo!("rehearsal")
    }
}
