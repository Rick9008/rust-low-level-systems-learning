//! sim m【virtual onsite 準備題】—— engine watchdog(R1 延伸;題幹:`docs/interviews/sim-problems.md`)。
//!
//! 「題目給的介面」區一律英文(中文對照:`docs/interviews/sim-problems-zh.md`)。
//!
//! 彩排規則同 sim_i:實作+自寫測試在本檔;`tests/sim_m_watchdog_test.rs` 跑完才開。
//! 本檔自帶一份 SimBus(時間版):engine 可能 hang、可能吐 zombie done。
//! clarify 可得的 spec:一塊正常 ~[`BLOCK_MS`] 毫秒完成;timeout 值由你自己定並講出理由。

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

// ===================== 題目給的介面(可讀)=====================

#[derive(Clone, Debug)]
pub struct DmaRequest {
    pub request_id: u64,
    pub block_nums: u32,
    pub block_start_pos: u64,
}

pub const ENGINE_COUNT: u32 = 6;
/// Normal per-block processing time (ms).
pub const BLOCK_MS: u64 = 10;
pub const TIMEOUT: u64 = BLOCK_MS * 3;

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
    /// Simulation only: no more requests will ever arrive
    /// (doesn't exist in the real interview; the loop never exits there).
    fn drained(&self) -> bool;
}

// ===================== 作答區 =====================

struct DmaReqTrack {
    pub request_id: u64,
    pub block_nums: u32,
    pub block_start_pos: u64,
    pub send: u32,
    pub done: u32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Alarm {
    deadline: u64,
    strikes: u8,
    req: u64,
    block: u32,
    pos: u64,
    eng: u32,
}

/// The R1 dispatcher plus a watchdog: a hung engine must not stall a request
/// forever. You need a third kind of state — a deadline per in-flight block.
/// A zombie done (late completion from a quarantined engine) must not corrupt
/// your bookkeeping.
pub fn run(bus: &mut impl DmaBus) {
    // todo!("彩排時實作")
    // we use a queue for ready engine
    let mut ready_engine = (0..ENGINE_COUNT).collect::<VecDeque<_>>();
    // and a priority queue for dead request detection
    let mut deadline_retry = BinaryHeap::<Reverse<Alarm>>::new();
    let mut waiting_queue = VecDeque::new();
    let mut retry_queue = VecDeque::new();
    let mut map: HashMap<u64, DmaReqTrack> = HashMap::new();
    // (req_id, block_pos)
    let mut engine_doing: [Option<(u64, u64)>; ENGINE_COUNT as usize] =
        [None; ENGINE_COUNT as usize];
    while !bus.drained() || !map.is_empty() {
        if let Some(req) = bus.get_dma_request() {
            waiting_queue.push_back(req.request_id);
            map.insert(
                req.request_id,
                DmaReqTrack {
                    request_id: req.request_id,
                    block_nums: req.block_nums,
                    block_start_pos: req.block_start_pos,
                    send: 0,
                    done: 0,
                },
            );
        }

        while let Some(dead_req) = deadline_retry.peek()
            && dead_req.0.deadline <= bus.now_ms()
        {
            let Reverse(mut dead_req) = deadline_retry.pop().unwrap();
            let engine = dead_req.eng as usize;
            if engine_doing[engine].is_none() {
                continue;
            }
            let (req_id, block_pos) = engine_doing[engine].unwrap();
            if req_id != dead_req.req || block_pos != dead_req.pos {
                continue;
            }
            dead_req.strikes += 1;
            engine_doing[dead_req.eng as usize].take();
            if dead_req.strikes == 3 {
                map.remove(&dead_req.req);
                bus.submit_dma_request_error(dead_req.req);
                continue;
            }
            retry_queue.push_back(dead_req);
        }

        while let Some(done) = bus.get_dma_result_done() {
            if engine_doing[done as usize].is_none() {
                continue;
            }
            let (req_id, _block_pos) = engine_doing[done as usize].take().unwrap();
            if map.contains_key(&req_id) {
                let req_entry = map.get_mut(&req_id).unwrap();
                req_entry.done += 1;
                if req_entry.done == req_entry.block_nums {
                    bus.submit_dma_request_result_done(req_id);
                    map.remove(&req_id);
                }
            }
            ready_engine.push_back(done);
        }

        while !ready_engine.is_empty() {
            let send_id = ready_engine.pop_front().unwrap();
            if let Some(req) = waiting_queue.pop_front()
                && let Some(req_entry) = map.get_mut(&req)
            {
                bus.send_dma_request_to_engine(
                    send_id,
                    req_entry.send,
                    req_entry.block_start_pos + req_entry.send as u64,
                );
                deadline_retry.push(Reverse(Alarm {
                    deadline: bus.now_ms() + TIMEOUT,
                    strikes: 0,
                    req,
                    block: req_entry.send,
                    pos: req_entry.block_start_pos + req_entry.send as u64,
                    eng: send_id,
                }));
                engine_doing[send_id as usize] =
                    Some((req, req_entry.block_start_pos + req_entry.send as u64));
                req_entry.send += 1;
                if req_entry.send != req_entry.block_nums {
                    waiting_queue.push_back(req);
                }
                continue;
            }
            if let Some(alarm) = retry_queue.pop_front() {
                bus.send_dma_request_to_engine(send_id, alarm.block, alarm.pos);
                deadline_retry.push(Reverse(Alarm {
                    deadline: bus.now_ms() + TIMEOUT,
                    eng: send_id,
                    ..alarm
                }));
                engine_doing[send_id as usize] = Some((alarm.req, alarm.pos));
                continue;
            }
            ready_engine.push_back(send_id);
            break;
        }

        bus.wait_event_timeout(
            deadline_retry
                .peek()
                .map_or(BLOCK_MS, |req| req.0.deadline.saturating_sub(bus.now_ms()))
                .max(1),
        );
    }
}

// ============ SimBus(模擬硬體;⚠ 跑題前不准細讀)============

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

/// 時間版假硬體:虛擬時鐘,塊固定 BLOCK_MS 完成,除非被腳本指定 hang。
#[derive(Default)]
pub struct SimBus {
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

impl SimBus {
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

impl DmaBus for SimBus {
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
