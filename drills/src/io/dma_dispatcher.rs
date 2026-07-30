//! drill:dma_dispatcher —— 填三張表的狀態轉移(sim i 的填空版)。
//!
//! 已給:題目介面與 MockBus(借 reference 的,harness 不算解法)、
//! `Dispatcher` 結構(三張表)、`run` 事件迴圈(迴圈紀律當閱讀材料)。
//! 要填:`register` / `cancel` / `dispatch` / `on_done` 四個轉移函式。
//!
//! 核心不變量(R1 的洞就在這):
//! - 完成判定**只看 per-request `done == blocks_total`**——任何借 engine
//!   空閒度推斷的判定,在兩單同時在飛時都只是巧合正確;
//! - done 事件只帶 engine id,路由靠 `owner` 表(派工當下就要記);
//! - cancel 不可搶佔在飛的塊(硬體沒有拔線),只能惰性收尾。
//!
//! 設計取捨與手 trace 見 reference 同名模組檔頭(**跑過 sim i 之後**再讀)。

use reference::io::dma_dispatcher::{DmaBus, DmaRequest, ENGINE_COUNT};
use std::collections::{HashMap, VecDeque};

/// 表 3 的一列:每張 request 的進度。
pub struct ReqState {
    pub blocks_total: u32,
    pub start: u64,
    /// 下一個還沒派出去的塊(== blocks_total 表示派完或已停派)。
    pub next_block: u32,
    pub done: u32,
    pub in_flight: u32,
    pub cancelled: bool,
}

/// 三張表 + 待派 queue。
pub struct Dispatcher {
    /// 表 1:誰有空。
    pub free: VecDeque<u32>,
    /// 表 2:engine → (request, block)。done 只帶 engine id,路由全靠這張表。
    pub owner: [Option<(u64, u32)>; ENGINE_COUNT as usize],
    /// 表 3:每單進度。
    pub reqs: HashMap<u64, ReqState>,
    /// 還有塊沒派完的 request(FIFO 填滿)。
    pub queue: VecDeque<u64>,
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            free: (0..ENGINE_COUNT).collect(),
            owner: [None; ENGINE_COUNT as usize],
            reqs: HashMap::new(),
            queue: VecDeque::new(),
        }
    }

    /// spec:收單。boundary——`block_nums == 0` 的單沒有工可派,
    /// **收單即完成**(直接 `submit_dma_request_result_done`,不進表);
    /// 其餘建 `ReqState` 入 `reqs`,request_id 排進 `queue` 尾。
    pub fn register(&mut self, bus: &mut impl DmaBus, r: DmaRequest) {
        todo!("spec: 0 塊即完成;其餘入表 + 入待派 queue")
    }

    /// spec:取消。不認識的 id(已完成/從沒來過)是 no-op——cancel 要冪等。
    /// 已知的單:標 `cancelled`、讓它從此派不出塊;`in_flight == 0` 的
    /// 現在就退場(移出 `reqs`),否則留給 `on_done` 收最後一塊。
    pub fn cancel(&mut self, id: u64) {
        todo!("spec: 冪等 no-op;停止再派;in_flight==0 才退場")
    }

    /// spec:派工。只要還有空 engine **且**還有塊沒派,就一直派(FIFO 填滿:
    /// queue 前端的單先吃滿)。queue 前端派完/已取消的順手 pop 掉。
    /// 派一塊 = `send_dma_request_to_engine(eid, b, start + b)` +
    /// **派工當下記 `owner[eid]`** + `next_block`/`in_flight` 前進。
    pub fn dispatch(&mut self, bus: &mut impl DmaBus) {
        todo!("spec: 有空有工就派;owner 當下記;前端清掉派不動的單")
    }

    /// spec:收工。done 只帶 engine id → `owner[eid].take()` 路由回 (request, block);
    /// **engine 先回收**(push 回 `free`,與單的成敗無關)。
    /// cancelled 的單:塊不計數,最後一塊回來(`in_flight == 0`)靜默退場、不 submit。
    /// 正常單:`done += 1`,`done == blocks_total` 才 submit + 退場。
    pub fn on_done(&mut self, bus: &mut impl DmaBus, eid: u32) {
        todo!("spec: owner 路由;engine 先還;cancelled 靜默收尾;計數歸零才 submit")
    }
}

/// event loop(已給):收單 → 收 cancel → 派工 → 收工;**收工後不睡**
/// (done 釋出了 engine,回頭再派一輪、再收單,別急著 wait)。
pub fn run(bus: &mut impl DmaBus) {
    let mut d = Dispatcher::new();
    loop {
        while let Some(r) = bus.get_dma_request() {
            d.register(bus, r);
        }
        while let Some(id) = bus.get_cancel_request() {
            d.cancel(id);
        }
        d.dispatch(bus);
        let mut progressed = false;
        while let Some(e) = bus.get_dma_result_done() {
            d.on_done(bus, e);
            progressed = true;
        }
        if progressed {
            continue;
        }
        if !bus.wait_event() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reference::io::dma_dispatcher::MockBus;

    /// 單一 request 3 塊:塊 0/1/2 依序派上 engine 0/1/2,三塊全回才 submit。
    #[test]
    #[ignore = "drill:填完 register/dispatch/on_done 後拔掉"]
    fn single_request_lifecycle() {
        let mut bus = MockBus::new().request_at(0, 1, 3, 100);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![1]);
        assert_eq!(bus.sent_log, vec![(0, 0, 100), (1, 1, 101), (2, 2, 102)]);
    }

    /// pipeline 分水嶺:lifo 完成序,A=6 塊、B=1 塊同時到 → B 必須先 submit。
    /// (sequential 版會是 [1, 2]——這條測試就是在打 R1 的洞。)
    #[test]
    #[ignore = "drill:填完後拔掉"]
    fn pipeline_small_request_overtakes() {
        let mut bus = MockBus::new()
            .lifo()
            .request_at(0, 1, 6, 0)
            .request_at(0, 2, 1, 100);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![2, 1]);
    }

    /// 8 塊 > 6 台:後兩塊要等 engine 釋出才派得出去。
    #[test]
    #[ignore = "drill:填完後拔掉"]
    fn more_blocks_than_engines_queue_and_reuse() {
        let mut bus = MockBus::new().request_at(0, 1, 8, 0);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![1]);
        assert_eq!(bus.sent_log.len(), 8);
    }

    /// cancel in-flight:停止再派、在飛塊自然 done 不計數、engine 回收給下一單,
    /// 被取消的單靜默退場(不 submit)。
    #[test]
    #[ignore = "drill:填完 cancel/on_done 後拔掉"]
    fn cancel_in_flight_silent_exit_engines_recycled() {
        let mut bus = MockBus::new()
            .request_at(0, 1, 6, 0)
            .cancel_at(1, 1)
            .request_at(2, 2, 3, 100);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![2]);
    }

    /// cancel 搶在第一次派工前(同輪收單→收 cancel→派工):一塊都不准派;
    /// 順帶驗 cancel 不認識的 id(99)是 no-op。
    #[test]
    #[ignore = "drill:填完 cancel 後拔掉"]
    fn cancel_before_any_dispatch_and_unknown_id_noop() {
        let mut bus = MockBus::new()
            .request_at(0, 1, 3, 0)
            .cancel_at(0, 1)
            .cancel_at(0, 99)
            .request_at(0, 2, 2, 100);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![2]);
        assert!(bus.sent_log.iter().all(|&(_, _, pos)| pos >= 100));
    }

    /// boundary:0 塊的單收單即完成,不派任何工。
    #[test]
    #[ignore = "drill:填完 register 後拔掉"]
    fn zero_block_request_completes_immediately() {
        let mut bus = MockBus::new().request_at(0, 7, 0, 500);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![7]);
        assert!(bus.sent_log.is_empty());
    }
}
