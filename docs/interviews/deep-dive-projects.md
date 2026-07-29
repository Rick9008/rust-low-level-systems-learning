# Technical deep dive 英文稿(履歷專案)

場次:8/12(三)09:15–09:45 開場,45m。形式:挖履歷上的專案,一路追問到底。
練法:每個專案照下面模板寫英文骨架(不背逐字稿)→ 晚上出聲講 + Claude 當面試官追問(口述 #1–#3 排 7/31、8/2、8/7,全串 8/9)。

## 每個專案的模板(5-pillar 變體)

1. **90 秒 elevator**:這是什麼、為誰、規模數字一句話。
2. **問題與限制**:為什麼難(不是「做了什麼」,是「什麼卡住別人」)。
3. **設計**:架構一張圖能畫出來;資料流從進到出講一遍。
4. **Trade-off ≥2**:沒選的方案 + 為什麼(要有數字或失敗模式,不是「比較簡單」)。
5. **數字**:規模、延遲/吞吐、前後對比。講不出數字的段落會被追問到講出來為止。
6. **最難的 bug**:怎麼發現、怎麼縮小範圍、根因、之後的機制性防止。
7. **如果重來**:一個具體會改的決定(顯示反思,不是「都很好」)。

## 面試官必問清單(每個專案都拿這張表自檢)

- Why X over Y?(每個技術選型都要有一個被否決的對手)
- What breaks at 10× scale?
- What was YOUR contribution vs the team's?
- Hardest bug — walk me through the debugging.
- What would you change if you rebuilt it?

## 專案清單(2026-07-29 起草;素材 = 履歷 + Holdwin 簡報轉軸)

> 挑選邏輯:Etched 軸是 event loop / telemetry / 低階診斷 / Rust,不是分散式共識——所以
> **專案二(logging daemon)對 Etched 的命中率其實最高**(它就是一條 telemetry pipeline:
> 多 producer → event loop → 多 sink),開場順序建議 二 → 一 → 三。Holdwin 簡報裡的
> 交易對照句(fill/cancel、開盤時間)全部拿掉,不要帶進 Etched 場。
>
> **⚠ 唸稿前要你確認的事實(簡報 notes 裡你自己標的,別帶著洞上場)**:
> 1. 當時 PostgreSQL 版本是幾?(<16 → 「雙向邏輯複製當時不可用」一句結案;≥16 → 重量壓在「無自動衝突解決」)
> 2. 「客戶端自運轉、不能人工介入」的措辭是否精確?有遠端維運管道就改掉這條前提。
> 3. 「6+ 個月零不一致」的觀測手段是什麼(對帳 job / 告警 / 日誌比對)?觀測不到的部分準備誠實句。
> 4. CPU-spin 那個 read loop 是誰寫的?是自己的 code 才能當 failure 故事;只是追查者就只放 debugging 故事。
> 5. logging daemon 的慢 sink 策略實際是什麼(backpressure / drop / buffer)?有沒有任何吞吐數字?

### 專案一:Two-node Active-Active HA(Go + Rust,審核決策狀態同步層)

**1. 90-second elevator**

- "I built the **application-state replication layer** for a two-node Active-Active mail-security platform — the layer that keeps **human approval decisions** consistent across two nodes, when the network between them can **drop, stall, or duplicate messages**."
- "The platform already replicated *data* three ways; **nobody replicated what the application had *decided***. That was the gap I filled."
- "In production **6+ months, zero reported data-inconsistency incidents**, survived **one real node failover** with no mail delay."

**2. 問題與限制(卡住別人的是什麼)**

- Exactly **two nodes** — product form factor; a third machine is not an option.
- Runs **on customer premises**: no DBA, must **self-converge** after faults, no manual repair.(⚠ 確認項 2)
- Network between nodes: drops, stalls, duplicates. **No trusted global clock.**
- Either node can die; service must continue, and states must converge after recovery.

**3. 設計(一張圖畫得出來)**

- Operation log + **Redis Stream** delivery + task dispatcher; HAProxy in front for traffic.
- **At-least-once + idempotent apply**: the sender can never distinguish "peer never got it" from "the ACK was lost" (two generals) — so make **redelivery harmless** instead. Receiver: message-ID already in the processed table → ACK without re-executing; otherwise **execute the business logic and record the message ID in the same transaction**.
- 關鍵句逐字:"**The check and the write have to be one atomic operation — otherwise two concurrent retries both see 'not processed' and double-execute. Idempotency that isn't atomic only *looks* like idempotency.**"
- **Split-brain resolution derived from operation semantics**, not timestamps. 關鍵句逐字:"**An approval that already sent the mail beats a later reject — you can't recall a sent email by overwriting a row. The invariant lives in the real world, not in the database. Irreversible actions win.**"

**4. Trade-off ≥2(每個都有被否決的對手)**

- **vs PostgreSQL replication**:streaming = single writer,拿不到 Active-Active;雙向邏輯複製要 PG 16 `origin=none`(⚠ 確認項 1);決定性的一點——**no automatic conflict resolution**:衝突會讓複製停下來等人。關鍵句逐字:"**Replication is a transport-layer concern; conflict resolution is a policy-layer concern. Off-the-shelf tools move the data, but they can't decide which write should win.**"
- **vs Raft/Paxos**:n=2 → quorum 門檻 2 → **任一節點掛掉全組停寫——可用性反而比單機差**;n=3 產品形態不允許。所以換問題:不問 "who is allowed to write",改成 "both sides write, replay-safe, converge afterwards."
- **vs Last-Write-Wins**:時鐘不可信,而且 "later" ≠ "should win"——LWW 會靜默覆蓋已對現實世界產生後果的操作。
- **主動講代價**:收斂前存在 **inconsistency window**;衝突規則是**自己定義、自己驗證的 policy**。

**5. 數字**

- 2 nodes · 6+ months production · 1 real failover(郵件收發無延遲)· v3 代價 = **每操作多一次本地寫入**。

**6. 最難的 bug(v2 遺失視窗——演進本身就是證據)**

- v1 shared NFS:斷線重連後檔案鎖不可靠;共享儲存本身是 SPOF。→ v2 command queue(各自本地 DB):**佇列清空或 consumer 重啟時,尚未在對端落地的操作直接遺失**。→ v3:**派送前先寫本地備份;對端確認落地後才清除**。
- 講法:"each version was forced by a concrete failure, not by a whiteboard." 不要只講終版。

**7. 如果重來**

- Add a **witness / fencing token** to break the two-node symmetry(future work #1;當時部署形態不允許額外元件)。
- **Active inconsistency detection(reconciliation job)from day one**。「你怎麼知道沒事故?」的誠實句逐字:"**Strictly speaking, what I can guarantee is that no inconsistency was ever reported or caught by our checks; proactive reconciliation is the first thing I'd invest in if I rebuilt it.**"(⚠ 確認項 3:把實際觀測手段填進來)

**預期追問(簡報 notes 原有,保留)**:Redis 在關鍵路徑是不是單點(誠實講 persistence 設定 + AOF fsync 視窗)| consumer group / PEL / XAUTOCLAIM,訊息卡在 PEL 怎麼辦 | message-ID 表成長 → 時間窗回收,**窗口必須大於最大重試視窗** | 兩邊都做了不可逆操作?→ 設計上不允許同時發生;若真可能就得退回 fencing / witness——誠實說這是設計邊界。

### 專案二:High-concurrency system-logging daemon(Modern C++ / Asio)——Etched 主打

**1. 90-second elevator**

- "I built the **system-logging daemon** for the mail platform: every service on the box sends diagnostics to it; an **Asio event loop** receives, parses, routes and filters; a **multi-backend logging library** fans out to file, remote and system sinks."
- Etched 定位句逐字:"**It's a telemetry pipeline: many producers, one collector, multiple sinks — the same shape as an event-loop-over-hardware-signals problem, just with logs instead of interrupts.**"
- "Standardizing the diagnostics format cut roughly **30% of duplicated logging code** across services."

**2. 問題與限制**:各服務自刻 logging(格式發散、程式碼重複);**慢 sink 不能拖住 producer**(⚠ 確認項 5:實際策略);daemon 要能活過個別服務重啟。

**3. 設計**:single `io_context` event loop;receive/parse → route/filter → 統一 sink 介面,後端可插拔(file / remote / system)。

**4. Trade-off ≥2**:統一 library vs 各服務自由(代價 = 遷移工 + 一個共同依賴;沒走的路 = 放任,代價是跨服務查問題要先學每一種格式)| 慢 sink:block vs drop vs buffer(⚠ 填實際選擇與理由)| 單 event loop vs thread-per-source。

**5. 數字**:−30% duplicated code;全平台服務接入;(⚠ 確認項 5:有吞吐數字就補)。

**6. 最難的 bug——兩個 war story(Etched 加分區:strace/perf 紀律)**

- **FD exhaustion 全服務中斷,10 分鐘內復原**:現象 = 程序活著但所有連線失敗 → 關鍵句逐字:"**First split 'the program is wrong' from 'a resource is exhausted' — the two paths need completely different evidence.**" → 部署後負載尖峰疊 DDoS,fd 耗盡。追問備案:止血與根因並行,rollback 前保留現場。
- **CPU 空轉追到確切 syscall**:`perf` 熱點在 `read` → `strace` 看到同一個 fd 反覆回傳 0 → read loop 沒把 0-byte 當 EOF。關鍵句逐字:"**A 0-byte read is EOF, not 'no data this time'. Loop exit conditions have to match the syscall contract.**"(⚠ 確認項 4)

**7. 如果重來**:(挑一個講)end-to-end structured backpressure(per-source 計帳——跟 telemetry 題同構)/ 評估 completion-model IO(io_uring),並講清楚為什麼當時 readiness 就夠。

### 專案三:Real-time content-inspection microservices(Rust / Tokio / gRPC)

**1. 90-second elevator**

- "I develop and operate **production Rust microservices** doing real-time email content inspection — regex, IP filtering, PII detection, exact-word matching — with **Tokio running matchers concurrently**, sustaining **~3,500 messages/second on a single 8-core node**. I also **introduced Rust into the team's stack**."

**2–3. 問題/設計**:inspection 是 fan-out 形狀(一封信 → 多個 matcher)且延遲敏感;Tokio task per matcher,gRPC 進出;CPU-bound 與 IO-bound 分清楚。

**4. Trade-off ≥2(現成好戲在隔壁團隊的三個 MR)**

- Contributed **three merged MRs** to a partner team's Rust service:thread-per-request → **Tokio async**(max concurrent connections **~300×**)+ **Rayon** parallel scanning(**12.5× average QPS**,`perf` + load benchmark 驗證)+ macro-based trait interface;became their **go-to Rust reviewer**。
- 關鍵句逐字:"**Two runtimes for two kinds of work: Tokio owns the waiting, Rayon owns the number-crunching — mix them in one pool and you starve your IO.**"

**5. 數字**:3,500 msg/s @ 8 cores | ~300× max concurrent connections | 12.5× average QPS。

**6. 最難的 bug**:(⚠ 待你挑:async 化過程最痛的一課?)

**7. 如果重來**:(⚠ 待填一個具體會改的決定)

### 備用口袋:kv-storage-rs(個人專案)

- 一句話備著:"A Redis-command-compatible in-memory KV engine in Rust — 60 commands, TTL, RDB snapshots, **nanosecond-level benchmarks**; currently adding a RESP-over-TCP server." 被問 side project 或 Rust 深度時再展開。
