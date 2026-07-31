// ⚠⚠ 防雷:本檔是 sim m(engine watchdog)的解法。計時場排在 8/2(7/31 改制,自 8/8 前移)——
// 在那之前不要讀本檔(含註解與測試)。自定規則:跑題前不開 oracle/sol。

//! # engine_watchdog —— engine timeout 看門狗(sim m 教學版;R1 直系延伸)
//!
//! ## [Clarify]
//! 題幹:`docs/interviews/sim-problems.md` sim m(彩排 harness:`rehearsals/src/sim_m_watchdog.rs`)。
//! sim i 的 dispatcher,但 engine 可能 hang、可能吐 zombie done(隔離後遲到的完成)。
//! clarify 可得的 spec:
//! - 一塊正常 ~[`BLOCK_MS`] 毫秒完成;timeout 值**由你定並講出理由**;
//! - hang 的 request 不能永遠卡住;放棄的條件也是你的政策(這裡:同塊 3 次);
//! - 關鍵前提問題:「**re-execute 這塊安全嗎?**」——spec 給的答案是行為冪等,
//!   且「隔離舊台 + 單一在飛」保證同塊不會同時跑兩份,重派才合法。
//!
//! ## [Abstract]
//! 在 sim i 的三張表上加**第三種 state:時間**——每台在飛 engine 一支錶
//! (`deadline[e]`)。event loop 的事件源從兩種(request、done)變三種
//! (request、done、**錶到期**),`wait_event` 也升級成 `wait_event_timeout`:
//! 睡到最近的錶到期,不是睡到天亮。
//!
//! ## [Iterate]
//! V0 = sim i(沒有時間概念,hang = 永久卡死)→ V1 上錶 + timeout 重派
//! → V2 隔離 + zombie 免疫 → V3 retry budget → error 路徑。
//!
//! ## [Trade-offs]
//! - **timeout 值 `5 × BLOCK_MS`**:「p99 塊延遲的數倍」量級——太短誤殺慢引擎
//!   (誤殺 = 白白重做 + 好引擎被隔離),太長拖住整張 request 的尾延遲。
//! - **隔離不復用**:timeout 的 engine 清 owner、**不放回 free**(嫌疑犯)。
//!   它日後吐出 zombie done 才證明自己活著 → 復活回 pool(政策可換:永久除役)。
//! - **zombie 免疫的關鍵一行**:`owner[e] == None` 的 done 一律不碰帳——
//!   那塊早已重派並記過帳,zombie 只當「engine 生存證明」用。
//! - **retry budget = 3**:同一塊第三次 timeout 就放棄整張 request 走 error——
//!   無限重試會把 engine 一台台拖進黑洞(每次重派都可能再 hang 一台)。
//!
//! ## [Dry-Run]
//! 測試 `hang_once_timeout_and_redispatch` 的手 trace:req1(2 塊),塊 0 首派
//! 會 hang:t=0 派塊 0→e0(錶 50)、塊 1→e1(錶 50)→ t=10 塊 1 done,e1 還、
//! 塊 0 沒動靜 → t=50 錶響:e0 隔離、塊 0 重派 e1(新錶 t=110)→ t=60 塊 0
//! done → 計數 2/2 → submit。sent_log 裡塊 0 出現兩筆——重派有據可查。
//!
//! 對照:彩排解答 `rehearsals/examples/sol_sim_m_watchdog.rs`(同設計,單檔面試版)。

use std::collections::{HashMap, HashSet, VecDeque};

// ===================== 題目給的介面(與 sim m 相同,英文保留)=====================

#[derive(Clone, Debug)]
pub struct DmaRequest {
    pub request_id: u64,
    pub block_nums: u32,
    pub block_start_pos: u64,
}

pub const ENGINE_COUNT: u32 = 6;

/// Normal per-block processing time (ms).
pub const BLOCK_MS: u64 = 10;

/// The R1 APIs with `wait_event` upgraded to a timeout variant, plus a clock
/// and an error path.
pub trait DmaBus {
    fn now_ms(&self) -> u64;
    fn get_dma_request(&mut self) -> Option<DmaRequest>;
    fn send_dma_request_to_engine(&mut self, engine_id: u32, block_num: u32, block_start_pos: u64);
    fn get_dma_result_done(&mut self) -> Option<u32>;
    /// Sleep until an event arrives or `ms` elapse, whichever comes first.
    /// Waking guarantees nothing — poll for dones yourself.
    fn wait_event_timeout(&mut self, ms: u64);
    fn submit_dma_request_result_done(&mut self, request_id: u64);
    /// Give up on a whole request and report it upstream
    /// (Phase 2: after 3 timeouts on the same block).
    fn submit_dma_request_error(&mut self, request_id: u64);
    /// Simulation only: no more requests will ever arrive.
    fn drained(&self) -> bool;
}

// ===================== 實作 =====================

/// timeout 政策:p99 塊延遲的數倍。太短誤殺慢引擎,太長拖住 request。
pub const TIMEOUT_MS: u64 = 5 * BLOCK_MS;
/// 同一塊的 retry budget:第三次 timeout 放棄整張 request。
pub const MAX_TRIES: u32 = 3;

struct ReqState {
    total: u32,
    start: u64,
    done: u32,
}

/// sim i 的三張表 + 第三種 state(每台在飛 engine 的錶)+ retry 帳。
pub struct Dispatcher {
    free: VecDeque<u32>,
    owner: [Option<(u64, u32)>; ENGINE_COUNT as usize],
    /// 第三種 state:時間。有 owner 的 engine 必有錶;隔離的 engine 兩者皆無。
    deadline: [Option<u64>; ENGINE_COUNT as usize],
    reqs: HashMap<u64, ReqState>,
    /// 待派的 (request, block)——首派與重派走同一條隊,dispatch 不用分知道。
    work: VecDeque<(u64, u32)>,
    tries: HashMap<(u64, u32), u32>,
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
            deadline: [None; ENGINE_COUNT as usize],
            reqs: HashMap::new(),
            work: VecDeque::new(),
            tries: HashMap::new(),
        }
    }

    /// 收單:塊展開進 work 隊。boundary:0 塊即完成(同 sim i)。
    pub fn register(&mut self, bus: &mut impl DmaBus, r: DmaRequest) {
        if r.block_nums == 0 {
            bus.submit_dma_request_result_done(r.request_id);
            return;
        }
        self.reqs.insert(
            r.request_id,
            ReqState {
                total: r.block_nums,
                start: r.block_start_pos,
                done: 0,
            },
        );
        for b in 0..r.block_nums {
            self.work.push_back((r.request_id, b));
        }
    }

    /// 派工 = sim i + **上錶**(`now + TIMEOUT_MS`)。
    /// 已 error 退場的單,殘塊在這裡作廢(查 reqs 不在就跳)。
    pub fn dispatch(&mut self, bus: &mut impl DmaBus) {
        while !self.free.is_empty() {
            let Some(&(rid, b)) = self.work.front() else {
                return;
            };
            if !self.reqs.contains_key(&rid) {
                self.work.pop_front();
                continue;
            }
            self.work.pop_front();
            let eid = self.free.pop_front().unwrap();
            let st = &self.reqs[&rid];
            bus.send_dma_request_to_engine(eid, b, st.start + b as u64);
            self.owner[eid as usize] = Some((rid, b));
            self.deadline[eid as usize] = Some(bus.now_ms() + TIMEOUT_MS);
        }
    }

    /// 收工 = sim i + **zombie 免疫**:`owner[e] == None` 的 done 一律不碰帳
    /// (那塊早已重派並記過帳),只把 engine 當「活著的證明」收回 pool。
    pub fn on_done(&mut self, bus: &mut impl DmaBus, eid: u32) {
        let Some((rid, _b)) = self.owner[eid as usize].take() else {
            self.free.push_back(eid); // zombie:隔離台復活(政策可換:永久除役)
            return;
        };
        self.deadline[eid as usize] = None;
        self.free.push_back(eid);
        let Some(st) = self.reqs.get_mut(&rid) else {
            return; // 單已 error 退場,塊的完成不再有意義
        };
        st.done += 1;
        if st.done == st.total {
            bus.submit_dma_request_result_done(rid);
            self.reqs.remove(&rid);
        }
    }

    /// 收錶:到期的 engine **隔離**(不回 free),塊重派或(第 [`MAX_TRIES`] 次)
    /// 放棄整張 request 走 error。回傳有沒有動作(讓 event loop 決定要不要睡)。
    pub fn fire_timeouts(&mut self, bus: &mut impl DmaBus, now: u64) -> bool {
        let mut fired = false;
        for e in 0..ENGINE_COUNT as usize {
            if self.deadline[e].is_some_and(|d| d <= now) {
                let (rid, b) = self.owner[e].take().expect("有錶必有 owner");
                self.deadline[e] = None;
                fired = true;
                let t = self.tries.entry((rid, b)).or_insert(0);
                *t += 1;
                if *t >= MAX_TRIES {
                    if self.reqs.remove(&rid).is_some() {
                        bus.submit_dma_request_error(rid);
                    }
                } else {
                    self.work.push_back((rid, b)); // 隔離舊台 ⇒ 同塊單一在飛,重派合法
                }
            }
        }
        fired
    }

    fn idle(&self) -> bool {
        self.reqs.is_empty()
    }

    /// 下次該醒的時刻:最近的錶;沒錶就用一塊的節拍當輪詢間隔。
    fn next_sleep_ms(&self, now: u64) -> u64 {
        self.deadline
            .iter()
            .flatten()
            .min()
            .map_or(BLOCK_MS, |&dl| dl.saturating_sub(now).max(1))
    }
}

/// event loop:三種事件源(request、done、錶到期)。
/// 睡的長度 = 到最近的錶,**不是睡到天亮**——watchdog 的醒法是設計的一部分。
pub fn run(bus: &mut impl DmaBus) {
    let mut d = Dispatcher::new();
    loop {
        while let Some(r) = bus.get_dma_request() {
            d.register(bus, r);
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
        let now = bus.now_ms();
        if d.fire_timeouts(bus, now) {
            continue; // 重派的塊下一輪 dispatch
        }
        if bus.drained() && d.idle() {
            break; // 模擬耗盡(真面試這裡永遠不退)
        }
        bus.wait_event_timeout(d.next_sleep_ms(now));
    }
}

// ===================== Mock(時間版假硬體)=====================

struct Flight {
    engine: u32,
    rid: u64,
    block: u32,
    due: Option<u64>, // None = hang(永不完成);zombie = 很晚的 Some
}

struct Hang {
    always: bool,
    zombie: Option<u64>, // Some(delay) = hang 的那次會在 delay 後吐出 zombie done
    fired: bool,
}

/// 時間版假硬體(與彩排 `SimBus` 同協定):虛擬時鐘,塊固定 [`BLOCK_MS`] 完成,
/// 除非被腳本指定 hang。oracle:zombie 的重複完成只計一次,submit 騙不了帳。
#[derive(Default)]
pub struct MockBus {
    now: u64,
    requests: VecDeque<(u64, DmaRequest)>, // (到達時刻 ms, request)
    in_flight: Vec<Flight>,
    done_ready: VecDeque<u32>,
    engine_busy: [bool; ENGINE_COUNT as usize],
    hangs: HashMap<(u64, u32), Hang>,
    // oracle 帳
    info: HashMap<u64, (u32, u64)>,
    ranges: Vec<(u64, u32, u64)>,
    done_set: HashSet<(u64, u32)>,
    done_cnt: HashMap<u64, u32>,
    /// 對外斷言用:成功回報的 request(按順序)。
    pub submitted: Vec<u64>,
    /// 對外斷言用:報錯的 request。
    pub errors: Vec<u64>,
    /// 對外斷言用:每次派工 (engine, request, block)——重派會出現同塊多筆。
    pub sent_log: Vec<(u32, u64, u32)>,
}

impl MockBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// 在第 `ms` 毫秒投遞一個 request(block 範圍必須互斥)。
    pub fn request_at_ms(mut self, ms: u64, id: u64, nums: u32, start: u64) -> Self {
        for &(s, n, _) in &self.ranges {
            let no_overlap = start + nums as u64 <= s || s + n as u64 <= start;
            assert!(no_overlap, "測試腳本錯誤:request 範圍重疊");
        }
        self.ranges.push((start, nums, id));
        self.requests.push_back((
            ms,
            DmaRequest {
                request_id: id,
                block_nums: nums,
                block_start_pos: start,
            },
        ));
        self
    }

    /// 這塊「第一次被派工」時 hang;`zombie` = Some(delay) 表示 delay 毫秒後仍會吐 done。
    pub fn hang_once(mut self, rid: u64, block: u32, zombie: Option<u64>) -> Self {
        self.hangs.insert(
            (rid, block),
            Hang {
                always: false,
                zombie,
                fired: false,
            },
        );
        self
    }

    /// 這塊每次被派工都 hang(考 retry budget → error 路徑)。
    pub fn hang_always(mut self, rid: u64, block: u32) -> Self {
        self.hangs.insert(
            (rid, block),
            Hang {
                always: true,
                zombie: None,
                fired: false,
            },
        );
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

    /// 把所有到期的完成投進 done_ready;回傳有沒有投遞。
    fn deliver_due(&mut self) -> bool {
        let now = self.now;
        let mut any = false;
        let mut i = 0;
        while i < self.in_flight.len() {
            if self.in_flight[i].due.is_some_and(|d| d <= now) {
                let f = self.in_flight.remove(i);
                self.done_ready.push_back(f.engine);
                // zombie 的重複完成只計一次——dedup 是 oracle 的事,騙不了帳。
                if self.done_set.insert((f.rid, f.block)) {
                    *self.done_cnt.get_mut(&f.rid).unwrap() += 1;
                }
                any = true;
            } else {
                i += 1;
            }
        }
        any
    }
}

impl DmaBus for MockBus {
    fn now_ms(&self) -> u64 {
        self.now
    }

    fn get_dma_request(&mut self) -> Option<DmaRequest> {
        if self.requests.front().is_some_and(|(t, _)| *t <= self.now) {
            let (_, r) = self.requests.pop_front().unwrap();
            self.info
                .insert(r.request_id, (r.block_nums, r.block_start_pos));
            self.done_cnt.insert(r.request_id, 0);
            return Some(r);
        }
        None
    }

    fn send_dma_request_to_engine(&mut self, engine_id: u32, block_num: u32, block_start_pos: u64) {
        assert!(engine_id < ENGINE_COUNT, "engine id {engine_id} 越界");
        let e = engine_id as usize;
        assert!(
            !self.engine_busy[e],
            "engine {engine_id} 還在忙(hang 中或 done 未收)就再被派工"
        );
        let (rid, idx) = self.owner_of(block_start_pos);
        assert_eq!(
            block_num, idx,
            "block_num 與 block_start_pos 對不上(第 {idx} 塊)"
        );
        let due = match self.hangs.get_mut(&(rid, block_num)) {
            Some(h) if h.always || !h.fired => {
                h.fired = true;
                h.zombie.map(|z| self.now + z)
            }
            _ => Some(self.now + BLOCK_MS),
        };
        self.engine_busy[e] = true;
        self.in_flight.push(Flight {
            engine: engine_id,
            rid,
            block: block_num,
            due,
        });
        self.sent_log.push((engine_id, rid, block_num));
    }

    fn get_dma_result_done(&mut self) -> Option<u32> {
        let e = self.done_ready.pop_front()?;
        self.engine_busy[e as usize] = false;
        Some(e)
    }

    fn wait_event_timeout(&mut self, ms: u64) {
        if self.deliver_due() {
            return; // 已有到期事件,立即醒
        }
        let target = self.now + ms;
        // 下一個會發生的事:最早的完成(含 zombie)或下一張 request 到達。
        let mut earliest = self
            .in_flight
            .iter()
            .filter_map(|f| f.due)
            .filter(|&d| d > self.now)
            .min();
        if let Some((t, _)) = self.requests.front() {
            let t = *t;
            if t > self.now {
                earliest = Some(earliest.map_or(t, |e| e.min(t)));
            }
        }
        self.now = match earliest {
            Some(e) if e <= target => e,
            _ => target,
        };
        self.deliver_due();
    }

    fn submit_dma_request_result_done(&mut self, request_id: u64) {
        let (nums, _) = *self
            .info
            .get(&request_id)
            .unwrap_or_else(|| panic!("submit 了不認識的 request {request_id}"));
        assert!(
            !self.submitted.contains(&request_id),
            "request {request_id} 被 submit 兩次"
        );
        assert!(
            !self.errors.contains(&request_id),
            "request {request_id} 已報錯又 submit"
        );
        assert_eq!(
            self.done_cnt[&request_id], nums,
            "request {request_id} 還有塊沒真正完成就 submit(zombie 騙不了帳)"
        );
        self.submitted.push(request_id);
    }

    fn submit_dma_request_error(&mut self, request_id: u64) {
        assert!(
            self.info.contains_key(&request_id),
            "報錯了不認識的 request {request_id}"
        );
        assert!(
            !self.submitted.contains(&request_id),
            "request {request_id} 已 submit 又報錯"
        );
        assert!(
            !self.errors.contains(&request_id),
            "request {request_id} 報錯兩次"
        );
        self.errors.push(request_id);
    }

    fn drained(&self) -> bool {
        self.requests.is_empty()
    }
}

// ===================== 測試 =====================

#[cfg(test)]
mod tests {
    use super::*;

    /// 沒人 hang 的快樂路徑:watchdog 不該誤傷任何人(錶上了但沒響)。
    #[test]
    fn happy_path_no_false_kill() {
        let mut bus = MockBus::new()
            .request_at_ms(0, 1, 3, 0)
            .request_at_ms(0, 2, 1, 100);
        run(&mut bus);
        assert_eq!(bus.submitted.len(), 2);
        assert!(bus.errors.is_empty());
        assert_eq!(bus.sent_log.len(), 4, "沒 hang 不該有任何重派");
    }

    /// 檔頭 [Dry-Run] 的劇本:塊 0 首派 hang(無 zombie)→ 錶響隔離重派 →
    /// 照樣完成。sent_log 裡 (1, 塊0) 出現兩筆——重派有據可查。
    #[test]
    fn hang_once_timeout_and_redispatch() {
        let mut bus = MockBus::new()
            .request_at_ms(0, 1, 2, 0)
            .hang_once(1, 0, None);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![1]);
        assert!(bus.errors.is_empty());
        assert_eq!(
            bus.sent_log
                .iter()
                .filter(|&&(_, r, b)| r == 1 && b == 0)
                .count(),
            2
        );
    }

    /// zombie 免疫:塊 0 hang 到 500ms 才吐 done(此時早已重派完成、單已 submit)。
    /// zombie 不准弄髒帳(不重複計數、不 double-submit);隔離台用它復活,
    /// 600ms 的 req2 照常跑完。
    #[test]
    fn zombie_done_does_not_corrupt_books() {
        let mut bus = MockBus::new()
            .request_at_ms(0, 1, 2, 0)
            .hang_once(1, 0, Some(500))
            .request_at_ms(600, 2, 1, 100);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![1, 2]);
        assert!(bus.errors.is_empty());
    }

    /// retry budget:塊 0 每次派都 hang → 第 3 次收手,整張走 error 路徑;
    /// 不 submit、sent_log 恰好 3 筆(1 首派 + 2 重派)——無限重試是黑洞。
    #[test]
    fn hang_always_exhausts_retry_budget_to_error() {
        let mut bus = MockBus::new().request_at_ms(0, 1, 2, 0).hang_always(1, 0);
        run(&mut bus);
        assert_eq!(bus.errors, vec![1]);
        assert!(bus.submitted.is_empty());
        assert_eq!(
            bus.sent_log
                .iter()
                .filter(|&&(_, r, b)| r == 1 && b == 0)
                .count(),
            MAX_TRIES as usize
        );
    }

    /// boundary:0 塊的單收單即完成(同 sim i,watchdog 版也不能漏)。
    #[test]
    fn zero_block_request_completes_immediately() {
        let mut bus = MockBus::new().request_at_ms(0, 7, 0, 500);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![7]);
        assert!(bus.sent_log.is_empty());
    }
}
