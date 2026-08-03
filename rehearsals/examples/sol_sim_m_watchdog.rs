//! solution:sim m —— engine watchdog。**寫完彩排才開。**
//!
//! 在 sim i 的三張表上加**第三種 state:時間**(每台在飛 engine 一支錶),外加三個政策決定:
//! 1. timeout 值:`5 × BLOCK_MS`——「p99 塊延遲的數倍」,太短誤殺慢引擎、太長拖住 request。
//! 2. 隔離不復用:timeout 的 engine 清 owner、**不放回 free**(嫌疑犯);它日後吐出 zombie done
//!    才證明自己活著 → 復活。zombie 免疫的關鍵一行:`owner[e] == None` 的 done 一律不碰帳。
//! 3. retry budget = 3:同一塊第三次 timeout 就放棄整張 request 走 error 路徑——無限重試
//!    會把 engine 一台台拖進黑洞。真硬體還得先問「re-execute 安全嗎?」——這裡 spec 給的答案
//!    是「隔離舊台+單一在飛」保證同塊不會同時跑兩份,重派才合法。
//!
//! 驗證:`cargo run -p rehearsals --example sol_sim_m_watchdog`

use rehearsals::sim_m_watchdog::{BLOCK_MS, DmaBus, DmaRequest, ENGINE_COUNT, SimBus};
use std::collections::{HashMap, VecDeque};

const TIMEOUT_MS: u64 = 5 * BLOCK_MS;
const MAX_TRIES: u32 = 3;

struct ReqState {
    total: u32,
    start: u64,
    done: u32,
}

#[derive(Default)]
struct Dispatcher {
    free: VecDeque<u32>,
    owner: [Option<(u64, u32)>; ENGINE_COUNT as usize],
    deadline: [Option<u64>; ENGINE_COUNT as usize], // 第三種 state:每台在飛的錶
    reqs: HashMap<u64, ReqState>,
    work: VecDeque<(u64, u32)>, // 待派的 (request, block)——首派與重派同一條隊
    tries: HashMap<(u64, u32), u32>,
}

impl Dispatcher {
    fn new() -> Self {
        let mut d = Self::default();
        d.free.extend(0..ENGINE_COUNT);
        d
    }

    fn register(&mut self, bus: &mut impl DmaBus, r: DmaRequest) {
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

    fn dispatch(&mut self, bus: &mut impl DmaBus) {
        while !self.free.is_empty() {
            let Some(&(rid, b)) = self.work.front() else {
                return;
            };
            if !self.reqs.contains_key(&rid) {
                self.work.pop_front(); // 這張已 error 退場,塊作廢
                continue;
            }
            self.work.pop_front();
            let eid = self.free.pop_front().unwrap();
            let st = &self.reqs[&rid];
            bus.send_dma_request_to_engine(eid, b, st.start + b as u64);
            self.owner[eid as usize] = Some((rid, b));
            self.deadline[eid as usize] = Some(bus.now_ms() + TIMEOUT_MS); // 上錶
        }
    }

    fn on_done(&mut self, bus: &mut impl DmaBus, eid: u32) {
        let Some((rid, _b)) = self.owner[eid as usize].take() else {
            // zombie done:隔離台遲到的完成。帳一個都不准碰——它的塊早已重派並記過帳。
            // engine 用這個 done 證明自己還活著 → 復活回 pool(政策可換:永久除役)。
            self.free.push_back(eid);
            return;
        };
        self.deadline[eid as usize] = None;
        self.free.push_back(eid);
        let Some(st) = self.reqs.get_mut(&rid) else {
            return;
        }; // 單已 error 退場
        st.done += 1;
        if st.done == st.total {
            bus.submit_dma_request_result_done(rid);
            self.reqs.remove(&rid);
            // 單退場,retry 帳跟著清——side table 每個插入點都欠一個刪除點。
            self.tries.retain(|&(r, _), _| r != rid);
        }
    }

    /// 收錶:到期的 engine 隔離(不回 free),塊重派或放棄。回傳有沒有動作。
    fn fire_timeouts(&mut self, bus: &mut impl DmaBus, now: u64) -> bool {
        let mut fired = false;
        for e in 0..ENGINE_COUNT as usize {
            if self.deadline[e].is_some_and(|d| d <= now) {
                let (rid, b) = self.owner[e].take().expect("有錶必有 owner");
                self.deadline[e] = None;
                fired = true;
                if !self.reqs.contains_key(&rid) {
                    continue; // 單已退場:塊作廢(同 dispatch),死單不准再進 tries
                }
                let t = self.tries.entry((rid, b)).or_insert(0);
                *t += 1;
                if *t >= MAX_TRIES {
                    self.reqs.remove(&rid);
                    self.tries.retain(|&(r, _), _| r != rid); // 放棄同樣要清帳
                    bus.submit_dma_request_error(rid); // 放棄整張,往上報
                } else {
                    self.work.push_back((rid, b)); // 重派(隔離舊台 ⇒ 同塊單一在飛)
                }
            }
        }
        fired
    }
}

fn run(bus: &mut impl DmaBus) {
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
        if bus.drained() && d.reqs.is_empty() {
            break;
        }
        // 睡到最近的錶到期;沒錶就用一塊的節拍當輪詢間隔。
        let ms = d
            .deadline
            .iter()
            .flatten()
            .min()
            .map_or(BLOCK_MS, |&dl| dl.saturating_sub(now).max(1));
        bus.wait_event_timeout(ms);
    }
}

fn main() {
    // scenario 1:單次 hang → timeout 重派 → 照樣完成。
    let mut bus = SimBus::new()
        .request_at_ms(0, 1, 2, 0)
        .hang_once(1, 0, None);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![1]);
    assert_eq!(
        bus.sent_log
            .iter()
            .filter(|&&(_, r, b)| r == 1 && b == 0)
            .count(),
        2
    );

    // scenario 2:zombie done 不弄髒帳,後續 request 照常。
    let mut bus = SimBus::new()
        .request_at_ms(0, 1, 2, 0)
        .hang_once(1, 0, Some(500))
        .request_at_ms(600, 2, 1, 100);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![1, 2]);
    assert!(bus.errors.is_empty());

    // scenario 3:hang 到底 → 3 次收手,error 路徑。
    let mut bus = SimBus::new().request_at_ms(0, 1, 2, 0).hang_always(1, 0);
    run(&mut bus);
    assert_eq!(bus.errors, vec![1]);
    assert!(bus.submitted.is_empty());
    assert_eq!(
        bus.sent_log
            .iter()
            .filter(|&&(_, r, b)| r == 1 && b == 0)
            .count(),
        3
    );

    println!("sol_sim_m_watchdog: all green");
}
