// ============================================================
// 批改(Claude,2026-07-24 晚;wheel 第一版,11 個 compile error,回家修)
// scratch 不進 gate,broken 不影響 workspace。逐條:
//
// 【編譯錯】
//  L50  const SLOTS = 256;           → const SLOTS: usize = 256;(缺型別)
//  L64  vec![Vec::new(); SLOTS]      → Vec<TimeEntry> 非 Clone,vec![x;n] 要 Clone。
//                                       改 (0..SLOTS).map(|_| Vec::new()).collect()
//  L74/111/119  u64 當 slice 下標     → 下標要 usize:(x as usize) & (SLOTS-1)
//  L88  now_ms                       → self.now_ms(不在 scope)
//  L91  for entry in self.slots[..]  → &self.slots[..](&self 不能 move 出元素)
//  L93/97/98/112/113  entry.round    → entry.rounds(欄位名打錯)
//  L93 vs L96  3-tuple vs 2-tuple    → 解構對不齊(而且 now_id 沒用到)
//  L111 .extract_if(..|entry|..)     → .extract_if(.., |entry| ..)(range 後缺逗號)
//  L114 let mut fires = ..collect()  → 標型別 let mut fires: Vec<TimeEntry> = ..
//  L119 push(fire_e)                 → fire_e 是 &mut(iter_mut);要 push 擁有值。
//                                       for mut fire_e in fires { fire_e.rounds=..; slots[..].push(fire_e); }
//
// 【邏輯洞(編過也會錯)】
//  1. ★ len 從沒 +1:schedule 要 self.len += 1。否則 is_empty 永遠 true、
//     next_deadline 永遠 early-return None。這個最致命。
//  2. rounds 算法 L77 用 first_at/SLOTS = 假設 now_ms==0;正解
//     rounds = (first_at - self.now_ms) / SLOTS(相對當前時鐘)。
//  3. next_deadline 的「最近」要比【觸發時間】= delta + rounds*SLOTS,
//     不是只比 rounds(delta 小但 rounds 大 ≠ 最早)。現在只比 round 會挑錯。
//  4. tick off-by-one:pop_due 是 `while now_ms<n { 處理 slot(now_ms+1); now_ms+=1 }`
//     → 跳過 tick 0(deadline=0 的 timer 在 pop_due(0) 不觸發)。heap 版是 deadline<=now
//     都觸發;決定 now_ms 是「已處理到」還是「下一個要處理」,對齊 heap 語意。
//
// 方向全對(絕對 slot=first_at%SLOTS ✓、pop_due 推進落點 ✓、rounds ✓、
// extract_if 單趟 fire+decrement ✓、sort_by_key ✓、drift-free 重排 ✓)。
// 就是第一版手滑 + len 沒記 + next_deadline 比較鍵。修完:
//   rustc --edition 2024 --crate-type lib --emit=metadata -o /dev/null scratch/timer_queue2.rs
// ============================================================

//! timer_queue2 —— timing wheel 版(同一個 public 合約,內部換成雜湊輪)。
//!
//! 目的:親手體會 wheel 的 O(1) 插入 vs heap 的 O(log n),以及 wheel 在
//! `next_deadline`(park-until-next)這件事上的**先天弱點**。
//!
//! 編譯檢查(在 repo 根目錄):
//!   rustc --edition 2024 --crate-type lib --emit=metadata -o /dev/null scratch/timer_queue2.rs
//!
//! ─────────────────────────────────────────────────────────────
//! 設計(單層 hashed wheel + rounds):
//!
//!   const SLOTS: usize = 256;   // 輪一圈 = 256 個 tick;tick = 1 邏輯 ms
//!
//!   - 每格是一串 Entry;Entry 至少要 { id, interval, rounds }。
//!   - `rounds` = 「還要繞幾整圈才輪到我觸發」。一圈只能表示 SLOTS ms 內的未來,
//!     所以 SLOTS ms 以外的 deadline 靠 rounds 記剩幾圈。
//!   - 維護 `now_ms`(輪自己的邏輯時鐘);當前格 = (now_ms % SLOTS)。
//!
//! 每個方法要做的事:
//!   schedule(id, first_at, interval):
//!     delay = first_at - now_ms;  slot = first_at % SLOTS;  rounds = delay / SLOTS
//!     → 把 Entry 塞進 slots[slot]
//!   pop_due(now):
//!     把 now_ms **一 tick 一 tick 推進到 now**;每步落在 slot = now_ms % SLOTS,
//!     掃那一格:rounds == 0 → 觸發(收 id + 用「舊 deadline + interval」重排、重新插回),
//!               rounds > 0  → rounds -= 1(這圈還沒輪到它)
//!   next_deadline():  ← 弱點在這,見坑 3
//!   len / is_empty:  自己記一個計數,或掃全格(慢)
//! ─────────────────────────────────────────────────────────────
//!
//! 坑(wheel 專屬,寫的時候盯著):
//!   1. rounds:別忘了「同一格但不同圈」——只有 rounds==0 的才觸發,其餘 -=1。
//!   2. now_ms 會跳:pop_due(now) 的 now 是任意值,你得把輪的時鐘從 self.now_ms
//!      推到 now。跳很遠 → O(elapsed_ticks),這就是 wheel 對「跳躍式邏輯時鐘」的
//!      先天代價(本練習可接受;真實系統 now 一 tick 一 tick 來)。
//!   3. next_deadline 是弱點:wheel 沒有 O(1) peek。你得從當前格往前掃、還要考慮
//!      rounds,才找得到最近的觸發 → O(SLOTS) 甚至更差。**這就是這場的 punchline:
//!      親手感受為什麼 park-until-next 用 heap 贏。**
//!   4. 排序:同一 tick 一起觸發的多個 Entry,回傳前要依 id 排(spec 的 (deadline, id))。
//!   5. 週期重排 + 追補:觸發後用「舊 deadline + interval」重新算 slot/rounds 插回;
//!      因為你是一 tick 一 tick 推進,若 old+interval 仍 <= now,之後的 tick 會再撞到它
//!      → 追補多次自然發生(不用特別處理)。

struct TimeEntry {
    id: u64,
    interval: u64,
    rounds: u64,
}

const SLOTS = 256;

pub struct TimerWheel {
    // ↓ 佔位:動手時換成 slots: Vec<Vec<Entry>>、now_ms、len 計數等。
    // _todo: (),
    slots: Vec<Vec<TimeEntry>>,
    now_ms: u64,
    len: usize,
}

impl TimerWheel {
    pub fn new() -> Self {
        // todo!("wheel: 建 SLOTS 個空格 + now_ms = 0")
        Self {
            slots: vec![Vec::new(); SLOTS],
            now_ms: 0,
            len: 0,
        }
    }

    /// 排一個週期任務:第一次在 `first_at_ms`,之後每 `interval_ms` 一次。
    /// `interval_ms >= 1`;id 唯一性由 caller 負責。
    pub fn schedule(&mut self, id: u64, first_at_ms: u64, interval_ms: u64) {
        // todo!("wheel: 算 slot = first_at % SLOTS、rounds = (first_at - now_ms) / SLOTS,塞進去")
        self.slots[first_at_ms & (SLOTS - 1)].push(TimeEntry {
            id,
            interval: interval_ms,
            rounds: first_at_ms / SLOTS,
        })
    }

    /// 下一個 deadline;沒有任何 timer → None。
    /// ⚠ 坑 3:wheel 沒 O(1) peek——往前掃格 + 算 rounds。
    pub fn next_deadline(&self) -> Option<u64> {
        // todo!("wheel: 從當前格往前掃最近的觸發(感受它為什麼比 heap 慢)")
        if self.is_empty() {
            return None;
        }
        let time = now_ms;
        let mut min_id = None;
        for delta in 0..256 {
            for entry in self.slots[(delta + time) & (SLOTS - 1)] {
                if min_id.is_none() {
                    min_id = Some((entry.id, entry.round, delta + time + entry.round * SLOTS));
                    continue;
                }
                let (now_id, now_round) = min_id.unwrap();
                if entry.round < now_round {
                        min_id = Some((entry.id, entry.round, delta + time + entry.round * SLOTS));
                }
            }
        }
        Some(min_id.unwrap().2)
    }

    /// 收割所有 deadline <= now 的觸發,依 (deadline, id) 排序回傳 id。
    /// 觸發後以「舊 deadline + interval」重排(不飄移);now 落後很多 → 補發多次。
    pub fn pop_due(&mut self, now_ms: u64) -> Vec<u64> {
        // todo!("wheel: 把 now_ms 一 tick 一 tick 推到 now;每步掃當前格,rounds==0 觸發、否則 -=1")
        let mut res = Vec::new();
        while self.now_ms < now_ms { 
            let mut fires = self.slots[(self.now_ms + 1) & (SLOTS - 1)].extract_if(..|entry| {
                if entry.round == 0 {true}
                else { entry.round -= 1; false}
            }).collect();
            fires.sort_unstable_by_key(|e| e.id);
            res.extend(fires.iter().map(|e| e.id));
            for fire_e in fires.iter_mut() {
                fire_e.rounds = (fire_e.interval) / SLOTS;
                self.slots[(self.now_ms + 1 + fire_e.interval) & (SLOTS - 1)].push(fire_e);    
            }
            self.now_ms += 1;
        }
        res
    }

    /// 目前排程中的 timer 數。
    pub fn len(&self) -> usize {
        // todo!("wheel")
        self.len
    }

    pub fn is_empty(&self) -> bool {
        // todo!("wheel")
        self.len == 0
    }
}

impl Default for TimerWheel {
    fn default() -> Self {
        Self::new()
    }
}
