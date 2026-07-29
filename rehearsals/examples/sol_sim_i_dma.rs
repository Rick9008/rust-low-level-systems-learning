//! solution:sim i —— DMA dispatcher v2(R1 重做 + pipeline + cancel)。**寫完彩排才開;
//! 7/29 例外:今天的任務就是讀它**(R1 隔天,把洞的正解一次看懂)。
//!
//! R1 的洞回顧:sequential 一次一單 + 「剩餘塊 == 0 且空閒 queue 長度回到 6」當完成判定。
//! 這個判定**只在單 request 模式成立**——一旦兩單同時在飛,engine 空不空和「哪一單完成」
//! 再無關係。correct 解 = 三張表,完成判定改成 per-request 計數歸零:
//!
//! 1. `free`:誰有空(engine id 的 queue)。
//! 2. `owner`:誰在做誰的哪塊——done 只給 engine id,路由全靠這張表。
//! 3. `reqs`:每單進度(派到哪、完成幾塊、在飛幾塊、取消了沒)。
//!
//! [Trade-offs]
//! - 派工順序 FIFO 填滿(先來的單先吃滿 engine):實作最短、單內延遲最低;
//!   代價是大單會餓小單。要公平就換 round-robin(queue 輪轉),多 ~5 行。
//! - cancel 採惰性收尾:停止再派 + 在飛的塊等它自然 done 但不計數,
//!   engine 邊還邊回收。不能「立刻拔 engine」——硬體上沒有搶佔這回事。
//! - 完成判定 `done == blocks_total` 是唯一正解;任何借 engine 空閒度推斷的判定
//!   在 pipeline 下都是巧合正確。
//!
//! [Dry-Run] main() 的 scenario 2(pipeline):A=6 塊、B=1 塊同時到,lifo 完成序。
//! 派:A0..A5 佔滿 6 台 → wait → A5 done(lifo)→ 派 B0 → wait → B0 done
//! → B 計數 1/1 → **B 先 submit** → 之後 A4..A0 陸續 done → A submit。
//! sequential 版在同劇本下 B 永遠排在 A 後面——這就是兩版的分水嶺。
//!
//! 驗證:`cargo run -p rehearsals --example sol_sim_i_dma`(全綠會印 all green)。

use rehearsals::sim_i_dma::{DmaBus, DmaRequest, ENGINE_COUNT, SimBus};
use std::collections::{HashMap, VecDeque};

/// 每張 request 的進度。完成判定只看 `done == blocks_total`。
struct ReqState {
    blocks_total: u32,
    start: u64,
    next_block: u32, // 下一個還沒派出去的塊
    done: u32,       // 已完成塊數
    in_flight: u32,  // 派出去還沒回來的塊數
    cancelled: bool,
}

#[derive(Default)]
struct Dispatcher {
    free: VecDeque<u32>,                                // 表 1:誰有空
    owner: [Option<(u64, u32)>; ENGINE_COUNT as usize], // 表 2:engine -> (request, block)
    reqs: HashMap<u64, ReqState>,                       // 表 3:每單進度
    queue: VecDeque<u64>,                               // 還有塊沒派完的 request(FIFO)
}

impl Dispatcher {
    fn new() -> Self {
        let mut d = Self::default();
        d.free.extend(0..ENGINE_COUNT);
        d
    }

    fn register(&mut self, bus: &mut impl DmaBus, r: DmaRequest) {
        // boundary:0 塊的單沒有工可派,收單即完成。
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

    fn cancel(&mut self, id: u64) {
        let Some(st) = self.reqs.get_mut(&id) else {
            return;
        };
        st.cancelled = true;
        st.next_block = st.blocks_total; // 停止再派;queue 前端檢查會把它跳掉
        if st.in_flight == 0 {
            self.reqs.remove(&id); // 沒有在飛的塊,現在就能退場
        }
    }

    /// 只要還有空 engine 且還有塊沒派,就一直派(FIFO 填滿)。
    fn dispatch(&mut self, bus: &mut impl DmaBus) {
        while !self.free.is_empty() {
            // 找 queue 前端第一個還派得動的 request;派完/取消的順手清掉。
            let rid = loop {
                let Some(&rid) = self.queue.front() else {
                    return;
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

    /// done 只給 engine id → 查佔用表路由回 request,計數歸零才 submit。
    fn on_done(&mut self, bus: &mut impl DmaBus, eid: u32) {
        let (rid, _b) = self.owner[eid as usize]
            .take()
            .expect("done 來自一台沒派工的 engine");
        self.free.push_back(eid); // engine 先回收,和單的成敗無關
        let st = self.reqs.get_mut(&rid).unwrap();
        st.in_flight -= 1;
        if st.cancelled {
            // 取消的單:塊做完不計數,最後一塊回來就退場(靜默,不 submit)。
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

/// event loop:收單 → 收 cancel → 派工 → 收工;每輪把能做的做完才睡。
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
            continue; // 收工釋出了 engine,回頭再派一輪、再收單,別急著睡
        }
        if !bus.wait_event() {
            break; // 模擬結束(真面試這裡永遠 true)
        }
    }
}

fn main() {
    // scenario 1:單一 request 3 塊。
    let mut bus = SimBus::new().request_at(0, 1, 3, 100);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![1]);

    // scenario 2:pipeline(檔頭 Dry-Run 的劇本)——B 必須先 submit。
    let mut bus = SimBus::new()
        .lifo()
        .request_at(0, 1, 6, 0)
        .request_at(0, 2, 1, 100);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![2, 1]);

    // scenario 3:cancel——1 靜默退場,engine 回收給 2。
    let mut bus = SimBus::new()
        .request_at(0, 1, 6, 0)
        .cancel_at(1, 1)
        .request_at(2, 2, 3, 100);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![2]);

    // scenario 4:0 塊 boundary。
    let mut bus = SimBus::new().request_at(0, 7, 0, 500);
    run(&mut bus);
    assert_eq!(bus.submitted, vec![7]);

    println!("sol_sim_i_dma: all green");
}
