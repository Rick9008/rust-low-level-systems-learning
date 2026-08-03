# rehearsals —— 計時彩排(模擬 CoderPad 條件)

九題計時彩排。題目是面試 prompt 風格:只給場景、contract、規模,**不給任何做法提示**。
環境約束照 [`docs/coderpad-constraints.md`](../docs/coderpad-constraints.md)。
題 d 用 tokio(pad 實測清單有);其餘只准 std。

> **virtual onsite(R2)準備題:sim 系列(i–n)**——R1(2026-07-28)實測題型的 spec-heavy 模擬
> (長英文 spec 埋洞 + provided API + 實作一個 fn,Claude 當面試官、Phase 2 驗收後才放)。
> 題幹(英文):[`docs/interviews/sim-problems.md`](../docs/interviews/sim-problems.md),
> 中文對照 `sim-problems-zh.md`|harness:`src/sim_{i,j,k,l,m,n}_*.rs`
> (上半「題目給的介面」是英文、可讀;**SimBus 區跑題前不准細讀**)|
> 參考測試 `tests/sim_*_test.rs` 與解答 `examples/sol_sim_*.rs` 六份全備(寫完才開;
> 19/19 參考測試已用解答驗過)。日程:[`docs/interviews/README.md`](../docs/interviews/README.md)。

定位:**題 a–d 是主菜**,全程 45 分鐘照 protocol 跑。**題 e–h 對應題型預測的
Q4–Q7**,預設做 recognition 練習——讀題 → 30 秒定界宣言 → 口述 arc 與 trade-off
——想全程寫也照 protocol 跑,骨架與參考測試都在。
**題 e2(fd_registry)是 JD 點名的 sleeper**,值得完整 45 分鐘跑一次。

## 進度狀態表(2026-07-30 掃描)

| 題 | 檔案 | 定位 | 檔內現況(todo!/自測) | 計時場排程 | 洞筆記 |
|---|---|---|---|---|---|
| a | ring_drop_oldest | 全場主菜 | 9 todo!/3 自測 | 已跑(衝刺期) | |
| b | pool_graceful_shutdown | 全場主菜 | 2 todo!/3 自測 | 已跑(衝刺期) | |
| c | frame_parser_heartbeat | 全場主菜 | 全填/2 自測 | 已跑(衝刺期) | |
| d | tokio_frame_server | 全場主菜(tokio) | 1 todo!/0 自測 | 已跑?(檔內無自測,重打時自查) | |
| e | event_registry | 認題 Q4 | 1 todo!/1 自測 | recognition | |
| e2 | fd_registry | JD sleeper,值得全場 | 1 todo!/3 自測 | 已跑(衝刺期) | |
| f | telemetry_aggregator | 認題 Q5 | 1 todo!/2 自測 | recognition | |
| g | bounded_channel | 認題 Q6 | 4 todo!/2 自測 | recognition | |
| h | timer_queue | 認題 Q7 | 6 todo!/1 自測 | recognition | |
| sim i | sim_i_dma | R2 模擬(i-lite 30m) | 骨架 | 7/30(sol 已開) | |
| sim j | sim_j_isr | R2 模擬全場 | ✅ 已作答(P1+P2 + 自測×2) | 7/31 ✅ 錶內,oracle 2/2 | |
| sim k | sim_k_fanin | R2 模擬 **lite**(7/31 改制:改跑 drills 填空,本 harness 備而不用) | 骨架 | 8/1 | |
| sim l | sim_l_mmio | R2 模擬 **lite**(同上;實跑走 drills 填空,本 harness 備而不用) | 骨架 | 8/2 ✅ 深夜 lite,drill 4/4 綠 | |
| sim m | sim_m_watchdog | R2 模擬全場(自 8/8 前移;認題輪取消) | ✅ 已作答(場後修帳+Alarm struct 重構;自測 0=已記洞) | 8/2 ✅ 65m(+20),場後修至參考測試 3/3 綠 | |
| sim n | sim_n_scheduler | R2 模擬 **lite**(改跑 drills 填空;自 8/9 前移) | 骨架 | **✅ 8/3 晚**(clarify 改制版) | drill 5/5 綠;帳:cards 8/3 §九 |
| sim o | (無 harness;走 `drills/src/ds/boot_planner.rs` 填空) | algo 系首發(8/2 深夜新增):Kahn 分層+DAG 最長路徑+環回報 | ✅ 已填(8 測全綠,含自寫紅測 ×2) | 8/3 ✅ 不計時三輪拉鋸(帳:cards_2026-08-03 §四) | |

「檔內現況」照掃描時點(重打後自行更新);「已跑」出自衝刺期紀錄(`../SCHEDULE.md`,已結案)。
sim 排程對照 [`docs/interviews/README.md`](../docs/interviews/README.md) 的逐日計畫。
**sim 六題的解答本與挖空版已入庫 `reference`/`drills`(2026-07-30)——場次前一樣不准開**
(各檔頂有防雷 banner)。**7/31 改制**:k/l/n 降 lite,其 **drills 填空版就是 lite 場材料(開跑即用)**;
reference 答案本一律場後才開。全場題(j/m)跑完後,複打走填空層:

| sim | reference(答案本)/ drills(填空)模組 | 開放時點 |
|---|---|---|
| i | `io/dma_dispatcher` | 已開(sol 7/29 已讀) |
| j | `concurrency/isr_pipeline` | ✅ 已開(7/31 場後;複打排 8/3 洞複掃) |
| k | `concurrency/percpu_fanin` | drills=8/1 lite 場開跑即用;reference=場後 |
| l | `io/mmio_cmdq` | drills=8/3 lite 場開跑即用(自 8/2 滑);reference=場後 |
| m | `io/engine_watchdog` | ✅ 已開(8/2 場後) |
| n | `concurrency/job_scheduler` | drills=8/3 lite 場開跑即用;reference=場後 |
| o | `ds/boot_planner` | drills=8/3 開機槽即用;reference=場後(algo 系,無 sim harness/防雷分頁) |

複打流程:`cargo test -p drills <模組名> -- --include-ignored` 看紅 → 讀函式上的 spec
註解填 `todo!()` → 拔 `#[ignore]` 轉綠;卡住才 diff `reference` 同名模組
(檔頭 5-pillar 詳解與 Dry-Run 手 trace 也留到那時再讀)。

## 規則

1. **計時 45 分鐘一題,照下方〈45 分鐘 protocol〉切段。** 時間到就停筆,誠實記錄做到哪。
2. **模擬單檔:** 實作寫在 `src/<name>.rs`,**你自己的測試也寫在同檔底部的
   `#[cfg(test)] mod tests`**——CoderPad 就是全部擠一個 buffer。
3. **彩排時自己寫測試。** 先想 boundary、自己出測項。
   **先在紙上 dry-run,再 `cargo test`**(Run 按鈕紀律,字面意思)。
4. 你自己的測試轉綠之後,**才**跑參考測試對照:

   ```sh
   cargo test -p rehearsals --test <name>_test -- --include-ignored
   ```

   參考測試含刻意建構的 boundary case,預設 `#[ignore]`(workspace 保持綠)。
   對照重點不是過不過,是:**你的測試漏了哪一類邊界?** 漏掉的那類
   (wrap?空?重複操作?欄位切斷點?)下次要在動手前就想到。
5. 開始前不要偷看 `tests/`,也不要看 `reference/` 對應模組。寫完隨便你 diff。

## 45 分鐘 protocol

| 時間 | 做什麼 | 紀律 |
|---|---|---|
| 0–5 | **Clarify,大聲問** | 鎖 contract(滿了怎辦?單雙執行緒?shutdown 語意?),問完不准中途改設計 |
| 5–10 | **Skeleton** | struct + 全部簽名 + `todo!()`,先求能編譯——這 5 分鐘決定後面不用 refactor |
| 10–30 | **Core,一次一個函式** | 邊寫邊講不變量(「len = tail − head」「worker 醒來先查 stop」) |
| 30–40 | **自己點名 boundary + dry-run** | 空/單元素/滿/wrap/切斷點;先在註解手 trace,**再**按 Run(pad 一次 ~7 秒,省著用) |
| 40–45 | **Trade-offs 收尾** | drop-oldest vs backpressure、Mutex vs atomics、規模轉折點在哪(→ epoll/tokio) |

每次彩排記錄各段實際花的分鐘數:第一次通常 core 爆,第二次通常 debug 爆,
第三次收斂。**設計在家裡決定,場上只是重放**——每題一個 canonical 設計練到反射
(ring = head+len;pool = `Arc<(Mutex<State>, Condvar)>` + `VecDeque`;
framer = `Vec<u8>` 累積 + drain 迴圈)。

## 大題定界:escalation ladder

Prompt 看起來像「build a runtime」(thread pool + executor + reactor 全都要)時,
考的是**定界**,不是手速。開場 30 秒先說:

> 「45 分鐘我先做單執行緒的 `block_on` + 一個 `Delay` future,因為 Waker 協議是
> 整個 runtime 的心臟;多 task spawn 和 IO reactor 我講架構不寫——時間剩我補 spawn。」

然後照階梯走——**一顆用寫的,其餘用講的**:

| 階段 | 形式 | 內容 |
|---|---|---|
| 1(25–30 分)| 寫 | `block_on` + `Waker` + park/unpark + `Delay`(那條計時 thread 就是微型 reactor) |
| 2(~3 分)| 講 | 多 task:run queue + `Arc<Task>`,wake = 把自己 push 回 queue——thread pool 骨架換 payload |
| 3(~3 分)| 講 | 真 IO:reactor thread + epoll + interest table;std 沒有多路等待原語,這層是 tokio(mio→epoll)接手 |
| 4(~2 分)| 收 | 轉折點:何時要 work-stealing、何時 readiness 不夠要 completion(io_uring) |

階段 3 的「講」配一個 stub 就夠(Abstract the Noise,詳見
[`docs/coderpad-constraints.md`](../docs/coderpad-constraints.md)):

```rust
trait Poller {
    fn register(&mut self, fd: RawFd, token: u64);
    fn wait(&mut self, out: &mut Vec<(u64, Ready)>, timeout_ms: i32) -> usize;
}
```

加一句「底層是 epoll,`epoll_event.data` 的 u64 就是我的 token,要的話可以展開」,
然後回到主結構——**不要現場手搓 FFI**。
整條梯子的可執行版在 `reference/src/runtime/mini_runtime.rs`(階段 2 的 run queue +
階段 3 的 reactor;V0 O(n) scan → V1 epoll,runtime 一行不改)。

背景知識見 [`docs/async/async-runtime-anatomy.md`](../docs/async/async-runtime-anatomy.md);
邊講邊用的數字在 [`docs/cost-model.md`](../docs/cost-model.md)。

## 當天認題(掃描表)

貫穿所有題的一句話:**「串流是無限的,你的記憶體不是。界在哪?」**
每題都是同一題的變形——差別只在「界」是什麼(容量、執行緒數、buffer、id 空間、
window、capacity、heap)。

| 聽到這個 | 題 | 第一個 clarify |
|---|---|---|
| "continuous stream" / "most recent N" | a(Q1) | **滿了怎麼辦?** |
| "concurrently" / "health checks" / "no external libraries" | b(Q2) | 幾條 thread?shutdown 語意? |
| "byte stream" / "protocol" / "frames" | c(Q3) | len 含不含 header?max frame size? |
| "event id" / "handlers" / "thousands of signals" | e(Q4) | **id 密集還是稀疏?** |
| "fd" / "handle recycling" / "stale event" / "connections come and go" | e2(Q4 進階) | **fd 會回收嗎?unregister 後佇列裡的舊 event 怎麼辦?** |
| "can't store them all" / "aggregate" / "windows" | f(Q5) | window 多大?timestamp 會亂序嗎? |
| "producers block when full" | g(Q6) | capacity?close 語意? |
| "periodic" / "interval" / "what runs next" | h(Q7) | 幾個 timer?精度? |

認不出來就問。開場永遠是:*"Before I start, let me make sure I understand the constraints."*

**彩排時題幹讀英文版:[`PROMPTS_EN.md`](PROMPTS_EN.md)**(面試是英文的——
認題、clarify、定界、trade-off 全程英文;下方中文版當對照與出處)。

問什麼、答案怎麼變成設計:[`docs/clarify-playbook.md`](../docs/clarify-playbook.md)
(互動版 [`docs/artifacts/clarify_playbook.html`](../docs/artifacts/clarify_playbook.html))。
專門練這一步的六張情境卡:[`clarify-cards.md`](clarify-cards.md),每張 5 分鐘;
答案在 `clarify-answers.md`,**寫完才開**。

## 對答案

每題有一份**已對參考測試驗證全綠**的 solution:`examples/sol_<name>.rs`。
**寫完(或時間到)才開**——信任模型跟 challenges 不偷看 `reference/` 相同。
solution 檔頂端註明該題的 canonical 設計與關鍵取捨;可直接跑 smoke:

```sh
cargo run -p rehearsals --example sol_ring_drop_oldest
```

題 a–c 在 `reference/` 另有教學版近親(`ring_buffer` / `thread_pool` /
`hw_bridge` framer),contract 略有差異(reference 的 ring 是滿了拒收,
不是 drop-oldest),diff 時注意語意差。

---

## 題目 a:ring_drop_oldest

感測器以固定頻率產生讀數(`u32`),下游消費速度不穩定,偶爾停頓。
你要寫一個固定容量的 buffer 接在中間。

需求:
- 容量固定,恰好容納 `capacity` 筆(`capacity >= 1`)。
- **滿的時候不能 block、也不能拒收新讀數**:丟掉最舊的一筆,讓新讀數進來。
- 被丟掉的筆數要累計,監控系統會定期讀這個數字。
- 取出端是 FIFO。

**Part 1:** 單執行緒版(`SensorRing`)。
**Part 2:** 生產者與消費者各在自己的執行緒上(恰好一產一消),
把它做成執行緒安全的版本(`channel(capacity) -> (Producer, Consumer)`)。

API 簽名在 `src/ring_drop_oldest.rs`。

## 題目 b:pool_graceful_shutdown

服務啟動後要並發地對數百台設備做 health check;每個 check 是一個阻塞呼叫,
交給固定數量的 worker 執行緒消化。

需求:
- `new(workers)` 起固定 worker 數的 thread pool;`submit(job)` 丟任務進去。
- 服務收到終止訊號時呼叫 `shutdown()`,要求 **graceful**:
  - 所有**已被接受**(submit 回 `Ok`)的任務都必須執行完;
  - `shutdown()` 回傳時,上述任務保證已完成;
  - `shutdown()` 之後的 `submit` 一律拒絕(回 `Err(Rejected)`);
  - `shutdown()` 可能被呼叫多次(訊號處理常見),必須安全。

std-only(`std::thread` / `std::sync`)。API 簽名在 `src/pool_graceful_shutdown.rs`。

## 題目 c:frame_parser_heartbeat

裝置透過 TCP 傳送 frame,wire format:

```text
[u32 len(big-endian)][payload:len bytes]
```

`len` 是 payload 的 byte 數;`len == 0` 的 frame 是 **heartbeat**(沒有 payload)。

TCP 是 byte stream,沒有 message 邊界:一次 `read` 拿到的 bytes 可能只有
半個 frame,也可能一次夾好幾個 frame。

需求:寫一個 incremental parser:`feed(&[u8])` 吃進這次新到的 bytes,
回傳**這次新完成**的所有 frame(依 stream 順序)。heartbeat 也要如實回報。
假設 stream 格式正確(信任的 peer,不需處理 malformed)。

API 簽名在 `src/frame_parser_heartbeat.rs`。

## 題目 d:tokio_frame_server

(唯一用 crate 的一題:tokio。)

裝置閘道器:多台裝置同時透過 TCP 連上來,講題目 c 的協定——
`[u32 len(BE)][payload]`,`len == 0` 是 heartbeat。

需求:
- 用 tokio 寫 server:`serve(listener, idle_timeout)`,服務到 listener 出錯為止。
- 每條連線並發服務、互相獨立。
- 收到 data frame → 原封不動 echo 回去(同 wire format)。
- 收到 heartbeat → 不回應。
- 一條連線超過 `idle_timeout` 沒有任何 bytes 進來 → 關閉該連線。
  heartbeat 算流量,能讓閒置的裝置連線活著——這就是它存在的目的。
- TCP 照樣沒有 message 邊界(半個 / 多個 frame),題目 c 的功課這裡要重用。

API 簽名在 `src/tokio_frame_server.rs`。

## 題目 e:event_registry(Q4)

硬體訊號帶著 event id 進來。要一個 registry:掛 handler、事件進來就 dispatch。
幾千種 id、高事件率。

需求:
- `register(id, handler)`:同一 id 可掛多個;`dispatch(id, payload)` 依**註冊順序**
  執行該 id 的所有 handler,回傳執行個數。
- handler 執行完回報去留(`After::Keep` / `After::Remove`)——`Remove` 從此不再被呼叫。
- 未知 id 是 no-op。
- dispatch 進行中不會有人 register(caller 保證)。

【clarify points——動手前先自答】
- id 密集還是稀疏?這決定你選什麼結構、trade-off 是什麼。
- 「dispatch 中途 unregister」為什麼在 Rust 裡特別麻煩?這份 API 用什麼方式繞開?

API 簽名在 `src/event_registry.rs`。

## 題目 e2:fd_registry(Q4 進階——JD 點名的 "event registry")

event loop 用 OS 的 readiness 介面同時等上萬條連線;事件發生時 kernel
每次只還你一個 u64。你要一個 registry:連線建立時登記(fd 是 kernel 給的
小整數),事件回來時用那個 u64 以 O(1) 找回連線狀態,連線關閉時移除。
注意:fd 關閉後 kernel 會把**同一個號碼**發給新連線,而事件佇列裡可能
還躺著舊連線的事件。高 churn(連線頻繁來去)。

需求:
- `register(fd, state) -> Token`:登記;token 要能塞進 u64(`to_raw` /
  `from_raw` 往返)——kernel 只給你這麼大的座位。
- `get / get_mut(token)`:O(1) 找回 state;**過期 token(fd 已回收再登記)
  必須回 `None`**,且不得影響現任住戶。
- `unregister(token) -> Option<T>`:移除並取回;過期 token 是 no-op。
- 全部操作 O(1);幾萬個 fd。

【clarify points——動手前先自答】
- fd 密集還是稀疏?這決定 HashMap 還是直接 index,trade-off 是什麼。
- 「舊事件 dispatch 到新住戶」的具體時序是什麼?你的結構用哪個欄位擋住它?
- token 憑什麼塞得進一個 u64?

API 簽名在 `src/fd_registry.rs`。

## 題目 f:telemetry_aggregator(Q5)

整機櫃數十億筆訊號,全存是不可能的。聚合進**固定數量**的 time window。

需求:
- `new(window_ms, num_windows)`:記憶體固定 O(num_windows),與樣本數無關。
- `record(ts, value)`;`stats(ts)` 回該 window 目前的 count / sum / min / max。
- window 是半開區間 `[k*w, (k+1)*w)`。
- timestamp 可能亂序:落在已淘汰的過去 → 拒絕(false);仍在保留範圍 → 收。
- ts 跳到未來 → 成為最新 window,中間被跳過的 window 視同空。
- 沒有資料的 window,`stats` → `None`。

【clarify points——動手前先自答】
- 為什麼空 window 不能回 zero 填充的 stats?
- slot 被重用之前,必須發生什麼事?

API 簽名在 `src/telemetry_aggregator.rs`。

## 題目 g:bounded_channel(Q6)

從零打造 bounded channel:producer 滿了 block、consumer 空了 block。std-only。

需求:
- `channel(capacity)`(capacity ≥ 1)→ `(Sender, Receiver)`;`Sender: Clone`
  (多生產者)、單消費者。
- `send`:滿 → block 到有空位;receiver 已 drop → `Err(SendError(v))` 把值原封還你。
- `recv`:空 → block;**所有** sender drop 且 buffer 清空 → `None`。
- block 中的一方,在對向條件成立**或對向消失**時都必須醒得來。

【clarify points——動手前先自答】
- 被喚醒就代表條件成立嗎?wait 要怎麼寫才對?
- 一顆還是兩顆 condvar?差在哪?

API 簽名在 `src/bounded_channel.rs`。

## 題目 h:timer_queue(Q7)

N 個 node、各自不同 interval 的週期 health check。誰下一個跑?該睡到什麼時候?

需求:
- `schedule(id, first_at, interval)`(interval ≥ 1;id 唯一性 caller 負責)。
- `next_deadline()`:給 caller 當 park 目標——**park 到那個時間點,不是輪詢**。
- `pop_due(now)`:收割所有到期觸發,依 (deadline, id) 排序;觸發後以
  「舊 deadline + interval」自動重排;now 落後很多時要補發錯過的週期。
- 時間用邏輯毫秒(u64),測試可決定性控制。

【clarify points——動手前先自答】
- 重排為什麼從舊 deadline 起算,不是 now?差在哪個字?
- heap 空的時候,caller 該 park 多久?新 timer 進來誰叫醒它?(接 executor 的 park token)

API 簽名在 `src/timer_queue.rs`。
