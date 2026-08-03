// ⚠⚠ 防雷:本檔是 sim m(engine watchdog)的填空版,spec 註解含解法方向。
// 計時場排在 8/2(7/31 改制,自 8/8 前移)——在那之前不要讀。自定規則:跑題前不開 oracle/sol。

//! drill:engine_watchdog —— 填含時間軸的四個轉移函式(sim m 的填空版)。
//!
//! 已給:題目介面與 `MockBus`(借 reference 的)、`Dispatcher` 結構
//! (sim i 三張表 + `deadline` 錶 + retry 帳)、`run` 事件迴圈與睡眠計算。
//! 要填:`register` / `dispatch` / `on_done` / `fire_timeouts`。
//!
//! 核心不變量:
//! - 每台在飛 engine 一支錶(`deadline[e]`);有 owner 必有錶,隔離台兩者皆無;
//! - timeout 的 engine **隔離不回 free**;它的 zombie done 一帳都不准碰
//!   (`owner[e] == None` 的 done 只當生存證明,把 engine 收回 pool);
//! - 同一塊第 [`MAX_TRIES`] 次 timeout → 放棄整張 request 走 error;
//! - 首派與重派走同一條 work 隊;已 error 退場的單,殘塊在 dispatch 作廢。
//!
//! 設計取捨(timeout 值怎麼定、為何隔離、budget 為何要有)見 reference
//! 同名模組檔頭(**跑過 sim m 之後**再讀)。

use reference::io::engine_watchdog::{BLOCK_MS, DmaBus, DmaRequest, ENGINE_COUNT};
use std::collections::{HashMap, VecDeque};

/// timeout 政策:p99 塊延遲的數倍(理由要講得出來)。
pub const TIMEOUT_MS: u64 = 5 * BLOCK_MS;
/// 同一塊的 retry budget。
pub const MAX_TRIES: u32 = 3;

pub struct ReqState {
    pub total: u32,
    pub start: u64,
    pub done: u32,
}

pub struct Dispatcher {
    pub free: VecDeque<u32>,
    pub owner: [Option<(u64, u32)>; ENGINE_COUNT as usize],
    /// 第三種 state:時間——每台在飛 engine 的錶。
    pub deadline: [Option<u64>; ENGINE_COUNT as usize],
    pub reqs: HashMap<u64, ReqState>,
    /// 待派的 (request, block):首派與重派同一條隊。
    pub work: VecDeque<(u64, u32)>,
    pub tries: HashMap<(u64, u32), u32>,
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

    /// spec:收單。0 塊 → 直接 submit 不進表;其餘建 `ReqState`,
    /// 塊 0..nums 全部展開 push 進 `work`。
    pub fn register(&mut self, bus: &mut impl DmaBus, r: DmaRequest) {
        todo!("spec: 0 塊即完成;其餘入表 + 塊展開進 work")
    }

    /// spec:派工。有空 engine 且 work 有塊就派;`reqs` 查不到的 rid(已 error
    /// 退場)把塊 pop 掉作廢;派出時記 `owner[eid]` **並上錶**
    /// (`bus.now_ms() + TIMEOUT_MS`)。
    pub fn dispatch(&mut self, bus: &mut impl DmaBus) {
        todo!("spec: 同 sim i 的派工 + 上錶;error 單的殘塊作廢")
    }

    /// spec:收工。`owner[eid].take()`:
    /// - `None` → **zombie**:帳一個都不碰,engine push 回 `free`(生存證明),return;
    /// - `Some((rid, b))` → 清錶、還 engine;`reqs` 查不到(已 error)就 return;
    ///   否則 `done += 1`,`done == total` → submit + 移出 `reqs`
    ///   **並清掉 `tries` 裡這張單的所有鍵**(side table 插入點欠刪除點,漏了是慢漏)。
    pub fn on_done(&mut self, bus: &mut impl DmaBus, eid: u32) {
        todo!("spec: zombie 免疫一行;正常路徑計數歸零才 submit")
    }

    /// spec:收錶。掃 6 台:`deadline[e] <= now` 的——`owner[e].take()` 拿回 (rid, b)、
    /// 清錶、**不還 free**(隔離);rid 已不在 `reqs`(死單)→ 塊作廢,不記 `tries`;
    /// 否則 `tries` +1:達 [`MAX_TRIES`] → `reqs.remove` + **清掉 `tries` 這張單的鍵**
    /// 再 `submit_dma_request_error`(整張放棄);否則塊 push 回 `work` 重派。
    /// 回傳這輪有沒有錶響(讓 run 決定要不要睡)。
    pub fn fire_timeouts(&mut self, bus: &mut impl DmaBus, now: u64) -> bool {
        todo!("spec: 到期→隔離+重派;第 MAX_TRIES 次→error;回傳有無動作")
    }

    pub fn idle(&self) -> bool {
        self.reqs.is_empty()
    }

    /// 下次該醒的時刻:最近的錶;沒錶就用一塊的節拍當輪詢間隔。(已給)
    pub fn next_sleep_ms(&self, now: u64) -> u64 {
        self.deadline
            .iter()
            .flatten()
            .min()
            .map_or(BLOCK_MS, |&dl| dl.saturating_sub(now).max(1))
    }
}

/// event loop(已給):三種事件源(request、done、錶到期);
/// 睡的長度 = 到最近的錶,不是睡到天亮。
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
            continue;
        }
        if bus.drained() && d.idle() {
            break;
        }
        bus.wait_event_timeout(d.next_sleep_ms(now));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reference::io::engine_watchdog::MockBus;

    /// 沒人 hang:watchdog 不誤傷,4 塊 4 筆派工、零重派。
    #[test]
    #[ignore = "drill:填完四個轉移後拔掉"]
    fn happy_path_no_false_kill() {
        let mut bus = MockBus::new()
            .request_at_ms(0, 1, 3, 0)
            .request_at_ms(0, 2, 1, 100);
        run(&mut bus);
        assert_eq!(bus.submitted.len(), 2);
        assert!(bus.errors.is_empty());
        assert_eq!(bus.sent_log.len(), 4, "沒 hang 不該有任何重派");
    }

    /// 塊 0 首派 hang → 錶響隔離重派 → 照樣完成;(1, 塊0) 派了兩次。
    #[test]
    #[ignore = "drill:填完 fire_timeouts 後拔掉"]
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

    /// zombie done(500ms 遲到的完成)不弄髒帳;隔離台復活,600ms 的 req2 照常。
    #[test]
    #[ignore = "drill:填完 on_done 的 zombie 分支後拔掉"]
    fn zombie_done_does_not_corrupt_books() {
        let mut bus = MockBus::new()
            .request_at_ms(0, 1, 2, 0)
            .hang_once(1, 0, Some(500))
            .request_at_ms(600, 2, 1, 100);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![1, 2]);
        assert!(bus.errors.is_empty());
    }

    /// 每次派都 hang → 第 3 次收手走 error;恰好 3 筆派工(1 首派 + 2 重派)。
    #[test]
    #[ignore = "drill:填完 fire_timeouts 的 budget 分支後拔掉"]
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

    /// boundary:0 塊即完成。
    #[test]
    #[ignore = "drill:填完 register 後拔掉"]
    fn zero_block_request_completes_immediately() {
        let mut bus = MockBus::new().request_at_ms(0, 7, 0, 500);
        run(&mut bus);
        assert_eq!(bus.submitted, vec![7]);
        assert!(bus.sent_log.is_empty());
    }
}
