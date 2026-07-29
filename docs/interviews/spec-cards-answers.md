# Spec-cards 答案鍵(埋的洞 + 期望 clarify + state 表 + 英文稿)

⚠ 同 `sol_*` 保護:每張卡**先做完(clarify 寫下 + 30 秒出聲 + state 表)才准開**。

英文稿用法:對照你講的 30 秒,缺什麼補什麼;不背逐字,背骨架(what → states → policy question → plan)。

---

## SP — Sensor interrupt pipeline

**埋的洞**:FIFO 深度/watermark 值?ring 滿了 drop 誰(oldest/newest/計數就好)?ISR 能不能 alloc/block/log?spurious interrupt 有沒有?wake 會不會合併(edge vs level)?

**期望 clarify ≥3**:overflow policy(drop 還是必須不掉)/ ISR 內允許的操作邊界 / 喚醒語意(醒來保證有資料嗎)。

**State 表**:ISR 側=無 state(只搬);ring=SPSC(生產者 ISR、消費者 worker);worker=drain-then-sleep。**經典雷:check-empty → sleep 之間來了 IRQ = lost wakeup**,順序必須 push→wake、drain 完再檢查一次才睡(signal_pipeline 的老朋友)。

**30-sec framing**:"This is a classic top-half/bottom-half split. The ISR must be minimal — no allocation, no blocking, no logging — it just drains the hardware FIFO into a lock-free SPSC ring and wakes the worker. The worker drains in batches and does the slow logging. The two policy questions I need answered: what do we drop when the ring is full — for telemetry I'd drop oldest and count drops — and whether wakeups can coalesce, which decides my re-check-before-sleep loop."

**Trade-off 收尾**:"I chose SPSC over a mutex queue because the producer is an ISR — it can never block. The cost is a fixed capacity and a drop policy we must own explicitly."

## FP — Frame parser with heartbeat

**埋的洞**:length 欄位大小/endianness?max frame length(防 OOM)?CRC/corrupt frame 怎麼辦?heartbeat 間隔與幾次 miss 才算死?`read_bytes` 會給半個 frame(一定會)。

**期望 clarify ≥3**:length prefix 格式與上限 / 壞 frame 的恢復策略(resync 還是斷線)/ heartbeat timeout = 多少個 interval。

**State 表**:parser state = 累積 buffer + 「等 header / 等 body(還缺 n bytes)」兩態;liveness state = `last_seen_ms`。兩個 state 互相獨立,poll_loop 每輪:讀 → 餵 parser(可能吐 0..n 個 frame,每個 frame 更新 last_seen)→ 查 `now - last_seen > timeout` → on_link_down。**雷:checked_add 防 length 惡意值;partial read 跨輪保留 buffer。**

**30-sec framing**:"Two independent state machines: a byte-stream reassembler — buffer plus a header/body state with a strict max-length check — and a liveness timer keyed off the last valid frame. The poll loop feeds bytes to the parser and checks the deadline every iteration. I need to clarify the resync story: after a corrupt length, do I scan for a magic byte or declare the link dead?"

**Trade-off 收尾**:"I keep one growable buffer per link instead of a fixed ring: simpler partial-frame handling, at the cost of needing the max-frame-length cap to bound memory."

## FR — Device event registry

**埋的洞**:unregister 後、queue 裡還躺著的舊 event 送給誰?(**stale delivery——本卡核心**)device id 會不會回收重用?dispatch 中 callback 能不能 register/unregister(re-entrancy)?同 id 重複 register?

**期望 clarify ≥3**:id 是否重用(是 → 需要 generation)/ 在途 event 對 unregister 的語意(必須靜默丟棄)/ callback 內再操作 registry 允不允許。

**State 表**:fd-dense `Vec<Option<Handler>>` slots + 每 slot `generation: u32`;token = `(gen << 32) | id`,event 帶 token,dispatch 時 gen 不合 = stale,丟。**這就是 e2,也是 arena generation 防 ABA 的同一招。**

**30-sec framing**:"The trap here is stale delivery: an event queued for the old owner of a reused device id must not reach the new owner. So the registry is a dense slot table with a generation counter per slot, and the dispatch token packs generation plus id. On dispatch I compare generations and silently drop mismatches. My clarify questions: are ids reused, and can callbacks mutate the registry re-entrantly — that decides whether I dispatch outside the borrow."

**Trade-off 收尾**:"A HashMap works, but device ids are small and dense — a Vec of slots is O(1) with no hashing, and the generation trick needs a stable slot anyway."

## TA — Telemetry aggregator

**埋的洞**:遲到樣本(ts 落在已關窗)收不收?未來 ts(clock skew)?窗什麼時候關(watermark?看到下一窗的樣本?)sensor 基數(記憶體上限)?沒樣本的窗要不要 emit?

**期望 clarify ≥3**:late/future 樣本策略 / 關窗條件 / sensor 數量級(決定 map 還是陣列)。

**State 表**:`HashMap<SensorId, (window_start, Stats)>`;每個樣本:算它的 window_start → 同窗就累積、新窗就先 emit 舊的再開新;future ts 直接視為新窗(**7/25 aggregator drill 的「未來 ts 清 window」case**);late 樣本按 policy 丟+計數。

**30-sec framing**:"Per-sensor windowed aggregation: a map from sensor to its open window and running stats. A sample either lands in the open window, or closes it — emit, then start the new window, including the clock-jump-forward case. The two policy decisions to clarify: what to do with late samples — drop and count is my default for telemetry — and whether empty windows must still emit, which changes this from event-driven to timer-driven."

**Trade-off 收尾**:"Event-driven close (a new sample closes the old window) needs no timer but delays emission on idle sensors; if the consumer needs bounded latency, I'd add a periodic flush tick."

## TQ — Timer service

**埋的洞**:同 deadline 的順序?cancel 已到期未執行的算成功嗎?callback 在誰的 thread 跑(能不能 block)?時鐘是 monotonic 嗎?大量 timer 的量級(BinaryHeap vs wheel)?

**期望 clarify ≥3**:cancel 語意(fire 和 cancel 的 race 誰贏)/ callback 執行環境 / timer 數量級與精度要求。

**State 表**:`BinaryHeap<(deadline, seq, id)>`(seq 破同刻平手)+ `HashSet<CancelledId>`(**lazy deletion**:pop 到才檢查,不從 heap 中間挖);loop:算最近 deadline → `park_until`(可能早醒 → 重算,不能假設醒了就到期)→ pop 所有 `<= now` 的、跳過 cancelled。

**30-sec framing**:"A min-heap of deadlines with a sequence number for same-tick ordering, plus lazy cancellation — cancelled ids go into a set and get skipped at pop time, so cancel is O(1). The run loop parks until the earliest deadline, and because park can wake early, firing is always guarded by re-reading now. I'd clarify the cancel-vs-fire race semantics and how many timers we expect — past tens of thousands I'd switch to a hierarchical wheel."

**Trade-off 收尾**:"Heap is O(log n) per op and precise; a wheel is O(1) but quantizes deadlines to ticks — for this scale the heap is simpler and fast enough."

## HW-L — MMIO command queue

**埋的洞**:**寫 descriptor 和寫 doorbell 之間需要 ordering(寫穿到 device 前 descriptor 必須先可見)——本卡核心**;ring 滿(head==tail 語意/保留一格?)怎麼辦;completion 會亂序嗎(→ descriptor 要帶 tag);doorbell 寫的是絕對 tail 還是增量;completion 用 IRQ 還是 poll。

**期望 clarify ≥3**:device 看記憶體的順序保證(要不要 write barrier)/ 完成是否亂序、descriptor 有沒有 tag 欄位 / ring 滿的回壓策略。

**State 表**:submit 側=cached tail + 讀 head 判滿;completion 側=自己的 head;in-flight 表 `tag → cmd`。**submit 順序鐵律:填 descriptor → write barrier(Release)→ 寫 doorbell**;讀 completion 反向:讀到新 head(Acquire)→ 才讀 descriptor 內容。跟 SPSC ring 的 head/tail 座標系同構,只是對手從另一條 thread 換成 device。

**30-sec framing**:"This is an SPSC ring where the consumer is hardware. The invariant that matters: the descriptor contents must be globally visible before the doorbell write — so fill, then a release barrier, then ring the doorbell; and symmetrically, acquire on the completion head before reading completion data. I track in-flight commands by tag since completions may come back out of order. My clarify questions: what ordering the bus guarantees, whether completions reorder, and what to do when the submission ring is full."

**Trade-off 收尾**:"Polling completions burns a core but gives microsecond latency; IRQ-driven frees the core at the cost of interrupt overhead — for a high-rate accelerator I'd poll with a budget, then fall back to IRQ."

## HW-M — Engine watchdog

**埋的洞**:timeout 設多少、誰提供(spec 沒給就要問)?slow vs dead 分不清 → 誤判重送的後果(**重複執行——block 寫兩次可以嗎?idempotent?**)?hang 的 engine 之後還能用嗎(隔離/復活)?重試幾次後放棄、誰收 error?

**期望 clarify ≥3**:操作是否 idempotent(決定敢不敢 retry)/ timeout 值與誤判容忍 / engine 永久移出還是探活復用 / request 失敗要不要往上報。

**State 表**:R1 的兩張表(per-request 剩餘塊、engine→(request, block) 佔用)再加第三個 state:**時間**——`engine → deadline`。event loop 每輪:`wait_event_timeout(最近 deadline - now)` → 有 done 處理 done;超時 → 該 engine 標記 suspect、block 重派給別台、engine 移出 pool(policy 再議)。完成判定不變(per-request counter)。

**30-sec framing**:"The dispatcher already tracks two kinds of state — per-request progress and engine occupancy; the watchdog adds a third: a deadline per dispatched block. The event loop waits with a timeout equal to the nearest deadline. On expiry I re-dispatch the block to a healthy engine and quarantine the suspect. The question that decides everything: is a DMA block idempotent? If re-executing can corrupt, I need completion-side dedup by request-id before I'm allowed to retry at all."

**Trade-off 收尾**:"A short timeout recovers fast but misfires on slow engines, causing duplicate work; I'd start conservative — several times the p99 block latency — and count timeouts per engine to separate slow from dead."
