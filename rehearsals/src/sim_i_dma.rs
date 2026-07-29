//! sim i —— DMA dispatcher v2(題幹:`docs/interviews/sim-problems.md`;Phase 2 由面試官放)。
//!
//! 彩排規則:
//! - 只准讀「題目給的介面」區——那就是 R1 現場看到的東西。
//! - **SimBus 實作區跑題前不准細讀**(裡面藏著 clarify 才拿得到的 spec 答案)。
//! - 實作與你自己的測試都寫在本檔;`tests/sim_i_dma_test.rs` 跑完才開。
//! - 對照解答:`examples/sol_dma_dispatcher.rs`(寫完才開)。

use std::collections::{HashMap, HashSet, VecDeque};

// ===================== 題目給的介面(可讀)=====================

#[derive(Clone, Debug)]
pub struct DmaRequest {
    pub request_id: u64,
    pub block_nums: u32,
    pub block_start_pos: u64,
}

pub const ENGINE_COUNT: u32 = 6;

/// R1 的六個 API + Phase 2 的 cancel。真面試裡這些是 free function;
/// 這裡收進 trait 才能餵 mock,語意不變。
pub trait DmaBus {
    fn get_dma_request(&mut self) -> Option<DmaRequest>;
    /// Phase 2 才會出現 cancel;Phase 1 期間永遠回 `None`,可以先不理它。
    fn get_cancel_request(&mut self) -> Option<u64>;
    /// 把 request 的第 `block_num` 塊(位置 `block_start_pos`)派給 `engine_id`。
    fn send_dma_request_to_engine(&mut self, engine_id: u32, block_num: u32, block_start_pos: u64);
    /// 哪台 engine 剛做完。注意:**只給 engine id**。
    fn get_dma_result_done(&mut self) -> Option<u32>;
    /// 回傳 `false` = 模擬結束(真面試 = 永遠 `true`、迴圈不退)。
    fn wait_event(&mut self) -> bool;
    fn submit_dma_request_result_done(&mut self, request_id: u64);
}

// ===================== 作答區 =====================

/// 接收 request、把 blocks 派給 6 台 engine,每個 request 全部完成後 submit。
pub fn run(bus: &mut impl DmaBus) {
    todo!("彩排時實作;state 設計是考點,故意不給骨架")
}

// ============ SimBus(模擬硬體;⚠ 跑題前不准細讀)============

/// 測試用假硬體。engine 完成順序:預設照派工序(FIFO),`lifo()` 反轉。
/// 內建 oracle:派工/submit 違反協定直接 panic,錯誤當場開燈。
#[derive(Default)]
pub struct SimBus {
    tick: u64,
    lifo: bool,
    requests: VecDeque<(u64, DmaRequest)>,
    cancels: VecDeque<(u64, u64)>,
    in_flight: Vec<(u32, u64, u32)>, // (engine, request, block),按派工序
    done_ready: VecDeque<u32>,
    engine_busy: [bool; ENGINE_COUNT as usize],
    // oracle 帳
    info: HashMap<u64, (u32, u64)>, // request_id -> (block_nums, start)
    ranges: Vec<(u64, u32, u64)>,   // (start, nums, request_id);範圍必須互斥
    sent_set: HashSet<(u64, u32)>,
    done_count: HashMap<u64, u32>,
    cancelled: HashSet<u64>,
    /// 對外斷言用:成功回報的 request(按順序)。
    pub submitted: Vec<u64>,
    /// 對外斷言用:每次派工 (engine, block_num, block_start_pos)。
    pub sent_log: Vec<(u32, u32, u64)>,
}

impl SimBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// engine 完成順序改成「後派先完」——考 done 亂序路由。
    pub fn lifo(mut self) -> Self {
        self.lifo = true;
        self
    }

    /// 在第 `tick` 次 wait_event 之後投遞一個 request。
    /// 各 request 的 block 範圍必須互斥(oracle 靠位置反查歸屬)。
    pub fn request_at(mut self, tick: u64, id: u64, nums: u32, start: u64) -> Self {
        for &(s, n, _) in &self.ranges {
            let no_overlap = start + nums as u64 <= s || s + n as u64 <= start;
            assert!(no_overlap, "測試腳本錯誤:request 範圍重疊");
        }
        self.ranges.push((start, nums, id));
        self.requests.push_back((
            tick,
            DmaRequest {
                request_id: id,
                block_nums: nums,
                block_start_pos: start,
            },
        ));
        self
    }

    /// 在第 `tick` 次 wait_event 之後投遞 cancel(Phase 2)。
    pub fn cancel_at(mut self, tick: u64, id: u64) -> Self {
        self.cancels.push_back((tick, id));
        self
    }

    fn owner_of(&self, pos: u64) -> (u64, u32) {
        for &(s, n, id) in &self.ranges {
            if pos >= s && pos < s + n as u64 {
                return (id, (pos - s) as u32);
            }
        }
        panic!("派工位置 {pos} 不屬於任何 request");
    }
}

impl DmaBus for SimBus {
    fn get_dma_request(&mut self) -> Option<DmaRequest> {
        if self.requests.front().is_some_and(|(t, _)| *t <= self.tick) {
            let (_, r) = self.requests.pop_front().unwrap();
            self.info
                .insert(r.request_id, (r.block_nums, r.block_start_pos));
            self.done_count.insert(r.request_id, 0);
            return Some(r);
        }
        None
    }

    fn get_cancel_request(&mut self) -> Option<u64> {
        if self.cancels.front().is_some_and(|(t, _)| *t <= self.tick) {
            let (_, id) = self.cancels.pop_front().unwrap();
            self.cancelled.insert(id);
            return Some(id);
        }
        None
    }

    fn send_dma_request_to_engine(&mut self, engine_id: u32, block_num: u32, block_start_pos: u64) {
        assert!(engine_id < ENGINE_COUNT, "engine id {engine_id} 越界");
        let e = engine_id as usize;
        assert!(
            !self.engine_busy[e],
            "engine {engine_id} 還在忙(它的 done 還沒被收走)就再被派工"
        );
        let (rid, idx) = self.owner_of(block_start_pos);
        assert!(
            !self.cancelled.contains(&rid),
            "對已 cancel 的 request {rid} 派工"
        );
        assert_eq!(
            block_num, idx,
            "block_num 與 block_start_pos 對不上(第 {idx} 塊)"
        );
        assert!(
            self.sent_set.insert((rid, block_num)),
            "request {rid} 的第 {block_num} 塊被派了兩次"
        );
        self.engine_busy[e] = true;
        self.in_flight.push((engine_id, rid, block_num));
        self.sent_log.push((engine_id, block_num, block_start_pos));
    }

    fn get_dma_result_done(&mut self) -> Option<u32> {
        let e = self.done_ready.pop_front()?;
        self.engine_busy[e as usize] = false;
        Some(e)
    }

    fn wait_event(&mut self) -> bool {
        self.tick += 1;
        if !self.in_flight.is_empty() {
            let i = if self.lifo {
                self.in_flight.len() - 1
            } else {
                0
            };
            let (e, rid, _b) = self.in_flight.remove(i);
            self.done_ready.push_back(e);
            *self.done_count.get_mut(&rid).unwrap() += 1; // cancel 了硬體也照做完
            return true;
        }
        // 沒在飛的:還有沒到期的投遞或沒收走的 done,時間繼續走
        !self.requests.is_empty() || !self.cancels.is_empty() || !self.done_ready.is_empty()
    }

    fn submit_dma_request_result_done(&mut self, request_id: u64) {
        let (nums, _) = *self
            .info
            .get(&request_id)
            .unwrap_or_else(|| panic!("submit 了不認識的 request {request_id}"));
        assert!(
            !self.cancelled.contains(&request_id),
            "request {request_id} 已 cancel 卻被 submit"
        );
        assert!(
            !self.submitted.contains(&request_id),
            "request {request_id} 被 submit 兩次"
        );
        assert_eq!(
            self.done_count[&request_id], nums,
            "request {request_id} 還有塊沒做完就 submit"
        );
        self.submitted.push(request_id);
    }
}
