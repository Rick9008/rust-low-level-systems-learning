//! # dma_dispatcher —— bus 驅動的 DMA dispatcher(sim i 教學版)
//!
//! ## [Clarify]
//! 題幹:`docs/interviews/sim-problems.md` sim i(彩排 harness:`rehearsals/src/sim_i_dma.rs`)。
//! 6 台 DMA engine,request 進來拆塊派工;done 事件**只帶 engine id**;
//! 每單全部塊完成才回報。隱藏 spec(clarify 才拿得到):
//! - 多單可同時在飛(pipeline)——完成回報照「誰先做完」,不是誰先進來;
//! - cancel(Phase 2)對 in-flight 塊不可搶佔——硬體沒有「拔線」這回事;
//! - engine 做完一塊就空,空了就該再餵——閒置 engine = 浪費頻寬。
//!
//! ## [Abstract]
//! 三張表,一張都不能省:
//! 1. **`free`**:誰有空(engine id queue)。
//! 2. **`owner`**:engine → (request, block)——done 只給 engine id,路由全靠它。
//! 3. **`reqs`**:每單進度(派到哪、完成幾塊、在飛幾塊、取消了沒)。
//!
//! 完成判定唯一正解:**per-request `done == blocks_total` 計數歸零**。
//! R1 的洞就在這:「剩餘塊 == 0 且空閒 queue 回到 6」只在單 request 模式碰巧成立,
//! 兩單同時在飛時 engine 空不空與「哪一單完成」再無關係。
//!
//! ## [Iterate]
//! V0 sequential(一次一單,R1 現場版)→ V1 pipeline(三張表 + 計數歸零)
//! → V2 cancel(惰性收尾:停止再派,在飛塊等自然 done、不計數、engine 照回收)。
//!
//! ## [Trade-offs]
//! - 派工 FIFO 填滿:先到的單先吃滿 engine,實作最短、單內延遲最低;
//!   代價是大單餓小單。要公平換 round-robin(queue 輪轉),多 ~5 行。
//! - cancel 惰性 vs 立即:立即退場要「假裝塊沒發生」,但硬體會照做完並回 done,
//!   不等它就會把 done 路由到已消失的單——惰性(等 in-flight 歸零才退場)才對。
//! - 事件迴圈「收工後不睡」:done 釋出了 engine,先回頭再派/再收單,
//!   否則每輪只推進一件事,吞吐掉一半。
//!
//! ## [Dry-Run]
//! 測試 `pipeline_small_request_overtakes` 的手 trace(lifo 完成序):
//! A=6 塊、B=1 塊同時到 → A0..A5 佔滿 6 台 → A5 先 done → engine 還 → 派 B0
//! → B0 done → B 計數 1/1 → **B 先 submit** → A4..A0 陸續回 → A submit。
//! sequential 版同劇本 B 永遠壓在 A 後面——這一步就是兩版的分水嶺。
//!
//! 對照:彩排解答 `rehearsals/examples/sol_sim_i_dma.rs`(同設計,單檔面試版)。

use std::collections::{HashMap, HashSet, VecDeque};

// ===================== 題目給的介面(與 sim i 相同,英文保留)=====================

#[derive(Clone, Debug)]
pub struct DmaRequest {
    pub request_id: u64,
    pub block_nums: u32,
    pub block_start_pos: u64,
}

pub const ENGINE_COUNT: u32 = 6;

/// The six R1 APIs plus Phase 2's `cancel`. In the real interview these are
/// free functions; a trait here so tests can swap in a mock.
pub trait DmaBus {
    /// Pull the next incoming request, if any.
    fn get_dma_request(&mut self) -> Option<DmaRequest>;
    /// Phase 2 only; always `None` during Phase 1.
    fn get_cancel_request(&mut self) -> Option<u64>;
    /// Dispatch one block of a request (index `block_num`, at `block_start_pos`) to `engine_id`.
    fn send_dma_request_to_engine(&mut self, engine_id: u32, block_num: u32, block_start_pos: u64);
    /// Which engine just finished. Note: **you only get the engine id.**
    fn get_dma_result_done(&mut self) -> Option<u32>;
    /// Block until something happens. Returns `false` when the simulation is exhausted.
    fn wait_event(&mut self) -> bool;
    /// Report a request once ALL of its blocks are done.
    fn submit_dma_request_result_done(&mut self, request_id: u64);
}

// ===================== 實作 =====================

/// 表 3 的一列:每張 request 的進度。完成判定只看 `done == blocks_total`。
struct ReqState {
    blocks_total: u32,
    start: u64,
    /// 下一個還沒派出去的塊(== blocks_total 表示派完或已取消停派)。
    next_block: u32,
    done: u32,
    in_flight: u32,
    cancelled: bool,
}

/// 三張表 + 待派 queue。所有操作 O(1)(dispatch 的 queue 清理攤銷 O(1):
/// 每單最多被 pop 一次)。
pub struct Dispatcher {
    /// 表 1:誰有空。
    free: VecDeque<u32>,
    /// 表 2:engine → (request, block)。done 只帶 engine id,路由全靠這張表。
    owner: [Option<(u64, u32)>; ENGINE_COUNT as usize],
    /// 表 3:每單進度。
    reqs: HashMap<u64, ReqState>,
    /// 還有塊沒派完的 request(FIFO 填滿;要公平改輪轉)。
    queue: VecDeque<u64>,
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

    /// 收單:入表、入待派 queue。boundary:0 塊的單沒有工可派,收單即完成。
    pub fn register(&mut self, bus: &mut impl DmaBus, r: DmaRequest) {
        if r.block_nums == 0 {
            bus.submit_dma_request_result_done(r.request_id);
            return;
        }
        self.reqs.insert(
            r.request_id,
            ReqState {
                blocks_total: r.block_nums,
                start: r.block_start_pos,
                next_block: 0,
                done: 0,
                in_flight: 0,
                cancelled: false,
            },
        );
        self.queue.push_back(r.request_id);
    }

    /// cancel:停止再派;在飛的塊等自然 done(on_done 負責收尾)。
    /// 不認識的 id(已完成/從沒來過)是 no-op——cancel 天生要冪等。
    pub fn cancel(&mut self, id: u64) {
        let Some(st) = self.reqs.get_mut(&id) else {
            return;
        };
        st.cancelled = true;
        st.next_block = st.blocks_total; // 這單從此派不出塊
        if st.in_flight == 0 {
            self.reqs.remove(&id); // 沒有在飛的塊,現在就能退場
        }
    }

    /// 只要還有空 engine 且還有塊沒派,就一直派(FIFO 填滿)。
    pub fn dispatch(&mut self, bus: &mut impl DmaBus) {
        while !self.free.is_empty() {
            // queue 前端第一個還派得動的單;派完/取消的順手清掉(攤銷 O(1))。
            let rid = loop {
                let Some(&rid) = self.queue.front() else {
                    return; // 沒工可派,留著空 engine 等下一單
                };
                match self.reqs.get(&rid) {
                    Some(st) if !st.cancelled && st.next_block < st.blocks_total => break rid,
                    _ => {
                        self.queue.pop_front();
                    }
                }
            };
            let eid = self.free.pop_front().unwrap();
            let st = self.reqs.get_mut(&rid).unwrap();
            let b = st.next_block;
            bus.send_dma_request_to_engine(eid, b, st.start + b as u64);
            self.owner[eid as usize] = Some((rid, b)); // 派工當下就記路由
            st.next_block += 1;
            st.in_flight += 1;
        }
    }

    /// done 只帶 engine id → 查 `owner` 路由回 request;計數歸零才 submit。
    /// engine 回收與單的成敗無關——先還再說。
    pub fn on_done(&mut self, bus: &mut impl DmaBus, eid: u32) {
        let (rid, _b) = self.owner[eid as usize]
            .take()
            .expect("done 來自一台沒派工的 engine");
        self.free.push_back(eid);
        let st = self.reqs.get_mut(&rid).unwrap();
        st.in_flight -= 1;
        if st.cancelled {
            // 取消的單:塊做完不計數;最後一塊回來就靜默退場(不 submit)。
            if st.in_flight == 0 {
                self.reqs.remove(&rid);
            }
            return;
        }
        st.done += 1;
        if st.done == st.blocks_total {
            bus.submit_dma_request_result_done(rid);
            self.reqs.remove(&rid);
        }
    }
}

/// event loop:收單 → 收 cancel → 派工 → 收工;**收工後不睡**
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
            break; // 模擬耗盡(真面試這裡永遠 true)
        }
    }
}

// ===================== Mock(測試/教學 harness)=====================

/// 測試用假硬體(與彩排 `SimBus` 同協定)。engine 完成序預設照派工序
/// (FIFO),[`MockBus::lifo`] 反轉成「後派先完」——考 done 亂序路由。
/// 內建 oracle:違反協定(重複派塊、對忙碌 engine 派工、對已取消單派工、
/// submit 不完整的單…)當場 panic,錯誤顯性化。
#[derive(Default)]
pub struct MockBus {
    tick: u64,
    lifo: bool,
    requests: VecDeque<(u64, DmaRequest)>,
    cancels: VecDeque<(u64, u64)>,
    in_flight: Vec<(u32, u64, u32)>, // (engine, request, block),按派工序
    done_ready: VecDeque<u32>,
    engine_busy: [bool; ENGINE_COUNT as usize],
    // oracle 帳
    info: HashMap<u64, (u32, u64)>, // request_id -> (block_nums, start)
    ranges: Vec<(u64, u32, u64)>,   // (start, nums, request_id);範圍互斥
    sent_set: HashSet<(u64, u32)>,
    done_count: HashMap<u64, u32>,
    cancelled: HashSet<u64>,
    /// 對外斷言用:成功回報的 request(按順序)。
    pub submitted: Vec<u64>,
    /// 對外斷言用:每次派工 (engine, block_num, block_start_pos)。
    pub sent_log: Vec<(u32, u32, u64)>,
}

impl MockBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// engine 完成順序改成「後派先完」。
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

impl DmaBus for MockBus {
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

// ===================== 測試 =====================

#[cfg(test)]
mod tests {
    use super::*;

    /// 手 trace(單一 request 3 塊,FIFO 完成序):
    /// 輪 1:收單 id=1(3 塊,start=100)→ dispatch:塊 0→engine0(pos100)、
    ///   塊 1→engine1(pos101)、塊 2→engine2(pos102),free=[3,4,5] 但無工可派
    ///   → 無 done → wait:engine0 的塊完成,done_ready=[0]。
    /// 輪 2:on_done(0):owner[0]=(1,0) → done=1/3,不 submit → progressed,continue。
    /// 輪 3–4:同理 engine1、engine2 → done=3/3 → submit(1)。
    /// sent_log 順序驗證派工是「填滿式」一輪派完,不是一輪一塊。
    #[test]
    fn single_request_lifecycle() {
        let mut bus = MockBus::new().request_at(0, 1, 3, 100);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![1]);
        assert_eq!(bus.sent_log, vec![(0, 0, 100), (1, 1, 101), (2, 2, 102)]);
    }

    /// 檔頭 [Dry-Run] 的劇本:lifo 完成序,A=6 塊(id1)、B=1 塊(id2)同時到。
    /// A 佔滿 6 台 → A5 先回(lifo)→ engine5 還 → B0 派上 engine5 → B0 先回
    /// → B 計數 1/1 → **B 先 submit**,之後 A4..A0 陸續回 → A submit。
    /// sequential(一次一單)版在同劇本下 submitted 會是 [1, 2]——分水嶺就在這一步。
    #[test]
    fn pipeline_small_request_overtakes() {
        let mut bus = MockBus::new()
            .lifo()
            .request_at(0, 1, 6, 0)
            .request_at(0, 2, 1, 100);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![2, 1]);
    }

    /// 8 塊 > 6 台:前 6 塊佔滿,第 6、7 塊等 engine 釋出才派(等待 queue 生效),
    /// 全部 8 塊都派出、單一 submit。驗 dispatch 的「有空才派、空了再補」。
    #[test]
    fn more_blocks_than_engines_queue_and_reuse() {
        let mut bus = MockBus::new().request_at(0, 1, 8, 0);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![1]);
        assert_eq!(bus.sent_log.len(), 8);
    }

    /// cancel in-flight(sol scenario 3):id1(6 塊)全部在飛時 cancel →
    /// 停止再派、在飛塊自然 done 不計數、engine 邊還邊回收給 id2 →
    /// id1 靜默退場(不 submit),submitted 只有 [2]。
    /// oracle 同時驗證:cancel 之後沒有任何 id1 的塊再被派工。
    #[test]
    fn cancel_in_flight_silent_exit_engines_recycled() {
        let mut bus = MockBus::new()
            .request_at(0, 1, 6, 0)
            .cancel_at(1, 1)
            .request_at(2, 2, 3, 100);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![2]);
    }

    /// cancel 還在排隊、一塊都沒派的單:同輪「收單 → 收 cancel → 派工」順序下,
    /// id1 在 dispatch 前就被取消 → 一塊都不派(sent_log 全是 id2 的 pos≥100)。
    /// 順帶驗 cancel 不認識的 id(99)是 no-op 不 panic——冪等。
    #[test]
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

    /// boundary:0 塊的單。沒有工可派,收單即完成——不進表、不佔 queue。
    /// (register 的第一個分支;漏掉它的實作會讓這單永遠不 submit,模擬耗盡後斷言炸。)
    #[test]
    fn zero_block_request_completes_immediately() {
        let mut bus = MockBus::new().request_at(0, 7, 0, 500);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![7]);
        assert!(bus.sent_log.is_empty());
    }
}
