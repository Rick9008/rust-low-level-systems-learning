# Technical deep dive 英文稿(履歷專案)

場次:**8/10(一)10:00–10:45(Ulysses Kao),緊接 recruiter debrief 10:45–11:00(Molly Huang)**(8/1 官方行程信確認;7/31 曾記 8/11,已修正)。形式:挖履歷上的專案,一路追問到底。
練法:每個專案照下面模板寫英文骨架(不背逐字稿)→ 晚上出聲講 + Claude 當面試官追問(口述 #1–#3 與雙全串整段排 8/7–8/10,細排 8/6 收帳時定;8/6 前不碰)。

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
> **✅ 確認項(7/29 晚 Withers 回填,稿已按此修正)**:
> 1. PostgreSQL = **14** → 「雙向邏輯複製當時不可用」一句結案;根本理由 = 複製決定不了先後順序、也不知道誰的操作不可挽回(刪除/寄出)。
> 2. 「客戶端自運轉」真義 = **收信必須自動、不能暫停**——凌晨 AA 斷線也不能停收信,不能等人。
> 3. 觀測手段 = **內建不一致偵測:偵測到就自動重新同步 + 記次數**;半年以上全客戶機器計數 0,且零 support 回報。這比 hedge 句強,直接當證據講。
> 4. CPU-spin 是**前人程式碼的陳年 bug** → 只當 debugging 故事,不當 failure 認領;個人 failure 改用 **authz/authn 缺失**(CVSS 10.0 的起源,culture fit ★7)。
> 5. logging daemon 實際架構 = **dovecot → syslog → syslog-ng → 自寫 C++ daemon 解析後寫 DB**;慢 sink 策略/吞吐數字不確定 → **不講數字、不掰**,被問吞吐就誠實說沒有量測記憶、講架構瓶頸在 DB 寫入端。
>
> **8/7 進度**:✅ ★6 conflict 人物與場景補真(tech lead 提「mail flag + dsync」捷徑,場合=你召集的架構 review)、
> ✅ ★3 Moderation 三細節補真(兩週/上線一年多/點名 William;訊息同步改寫,通宵與「睡眠紅線」原稿對撞已解)。
> ✅ 專案三第 6/7 題落地(以 repo `libsynomailserver-moderation` 實證改寫,見該節)。**三個洞全清。**
> **殘項只剩兩條**:① 8/9 taper 重讀 Gjengset 40-hours 原文確認轉述正確;② 3,500 msg/s 要講就自帶 scope(專案三 §5)。

### 專案一:Two-node Active-Active HA(Go + Rust,審核決策狀態同步層)

**1. 90-second elevator**

- "I built the **application-state replication layer** for a two-node Active-Active mail-security platform — the layer that keeps **human approval decisions** consistent across two nodes, when the network between them can **drop, stall, or duplicate messages**."
- "The platform already replicated *data* three ways; **nobody replicated what the application had *decided***. That was the gap I filled."
- "In production **6+ months, zero reported data-inconsistency incidents**, survived **one real node failover** with no mail delay."

**2. 問題與限制(卡住別人的是什麼)**

- Exactly **two nodes** — product form factor; a third machine is not an option.
- Runs **on customer premises**, and **mail flow must never stop**: if the inter-node link drops at 3 a.m., both nodes keep accepting mail independently and reconcile later — pausing to wait for a human is not an option.
- Network between nodes: drops, stalls, duplicates. **No trusted global clock.**
- Either node can die; service must continue, and states must converge after recovery.

**3. 設計(一張圖畫得出來)**

- Operation log + **Redis Stream** delivery + task dispatcher; HAProxy in front for traffic.
- **At-least-once + idempotent apply**: the sender can never distinguish "peer never got it" from "the ACK was lost" (two generals) — so make **redelivery harmless** instead. Receiver: message-ID already in the processed table → ACK without re-executing; otherwise **execute the business logic and record the message ID in the same transaction**.
- 關鍵句逐字:"**The check and the write have to be one atomic operation — otherwise two concurrent retries both see 'not processed' and double-execute. Idempotency that isn't atomic only *looks* like idempotency.**"
- **Split-brain resolution derived from operation semantics**, not timestamps. 關鍵句逐字:"**An approval that already sent the mail beats a later reject — you can't recall a sent email by overwriting a row. The invariant lives in the real world, not in the database. Irreversible actions win.**"

**4. Trade-off ≥2(每個都有被否決的對手)**

- **vs PostgreSQL replication**(口述版):
  > "We were on Postgres fourteen. Streaming replication is single-writer, so no active-active; and bidirectional logical replication wasn't available yet — that needs Postgres sixteen. But the deeper reason holds on any version: replication has no notion of which write should win. It can't order two partitioned writes, and it certainly doesn't know that one of them already sent an email."
  關鍵句逐字:"**Replication is a transport-layer concern; conflict resolution is a policy-layer concern. Off-the-shelf tools move the data, but they can't decide which write should win.**"
- **vs Raft/Paxos**(口述版):
  > "With Raft or Paxos at n equals two, the quorum is two — lose either node and the whole system stops writing. Availability would be worse than a single machine, and a third node wasn't an option in this product. So I changed the question: instead of asking who is allowed to write, both sides write, replay-safe, and converge afterwards."
- **vs Last-Write-Wins**(口述版):
  > "Last-write-wins assumes a trusted clock we don't have — and 'later' doesn't mean 'should win'. It silently overwrites operations that already had real-world consequences."
- **主動講代價**(口述版):
  > "And I'll volunteer the cost: there is an inconsistency window before convergence, and the conflict rules are a policy I defined and had to validate myself — there's no textbook to point at."

**5. 數字(含觀測手段——這題必被問,答案是武器不是弱點)**

- 2 nodes · 6+ months production · 1 real failover(郵件收發無延遲)· v3 代價 = **每操作多一次本地寫入**。
- 「你怎麼知道零不一致?」逐字:"**We don't just hope it's consistent — the system detects divergence, re-syncs automatically, and counts every occurrence. Across every customer machine, for six-plus months, that counter stayed at zero — and support never received a single inconsistency report.**"

**6. 最難的 bug(v2 遺失視窗——演進本身就是證據)**

- 口述版(直接唸,不要只講終版):
  > "Version one used shared NFS — after a link flap the file locks were unreliable, and the shared storage itself was a single point of failure. Version two moved to command queues on each node's local database — but if the queue drained or the consumer restarted, operations that hadn't landed on the peer were simply gone. That loss window forced version three: back up locally before dispatch, and only clear the backup after the peer confirms it landed. Each version was forced by a concrete failure, not by a whiteboard."

**7. 如果重來**

- Add a **witness / fencing token** to break the two-node symmetry(future work #1;當時部署形態不允許額外元件——誠實講這是設計邊界)。

**預期追問(簡報 notes 原有,保留)**:Redis 在關鍵路徑是不是單點(誠實講 persistence 設定 + AOF fsync 視窗)| consumer group / PEL / XAUTOCLAIM,訊息卡在 PEL 怎麼辦 | message-ID 表成長 → 時間窗回收,**窗口必須大於最大重試視窗** | 兩邊都做了不可逆操作?→ 設計上不允許同時發生;若真可能就得退回 fencing / witness——誠實說這是設計邊界。

### 專案二:Log-ingestion pipeline + C++ daemon(Modern C++ / Asio)——Etched 主打

> 誠實版架構(7/29 確認):**dovecot → syslog → syslog-ng(路由/過濾)→ 自寫 C++ daemon(解析/正規化)→ DB**。
> 別再講「多後端 sink 扇出」那個膨脹版——sink 就是 DB;多後端 logging library 是**另一件事**(各服務發送端的統一介面,−30% 重複碼),兩者分開講。

**1. 90-second elevator**

- "I own the **tail of the mail platform's log pipeline**: services log through syslog, **syslog-ng** does the routing and filtering, and a **C++ daemon I wrote consumes that stream, parses and normalizes it, and lands it in a database** so diagnostics are queryable instead of grep-able."
- "Alongside it, a **shared logging library** standardized how services emit diagnostics — that cut roughly **30% of duplicated logging code**."
- Etched 定位句逐字:"**It's a telemetry pipeline: many producers, a routing layer, one structured sink — the same shape as an event-loop-over-hardware-signals problem, just with logs instead of interrupts.**"

**2. 問題與限制(口述版,直接唸)**

> "Before this, diagnostics lived in plain-text logs scattered across services, each with its own format — debugging one cross-service issue meant grepping three formats and joining them in your head. We needed a structured, queryable landing zone — and the collection path must never interfere with the mail-delivery hot path."

**3. 設計(口述版,直接唸)**

> "I stood on the syslog ecosystem instead of fighting it: transport and routing go to syslog-ng — battle-tested, config-driven. My daemon owns exactly two things: the parsing-and-normalization logic, and the database schema. Asio drives the daemon's IO."

**4. Trade-off ≥2(口述版;中文只是標籤)**

- **重用 syslog-ng vs 自建 collector**:
  > "The off-the-shelf transport is battle-tested and cheap to walk away from; the cost is being bound to syslog's format and delivery semantics. I didn't build a transport — the interesting problem was parsing and schema, so that's where my code lives."
- **獨立 daemon vs syslog-ng 直寫 DB**:
  > "Parsing rules and schema are code — they evolve and they need tests. I didn't want that living in a config layer. And the failure modes decouple: if my daemon dies, syslog-ng still buffers."
- **結構化落 DB vs 留純文字**:
  > "Query power versus write cost. And I won't invent a throughput number — I don't have it from memory; the bottleneck by construction is the DB write path, not the parse."

**5. 數字**:−30% duplicated logging code(library 那條);其餘不編數字。

**6. 最難的 bug——兩個 war story(Etched 加分區:strace/perf 紀律)**

- **FD exhaustion 全服務中斷,10 分鐘內復原**(口述版):
  > "The process was alive, but every new connection failed. First move: split 'the program is wrong' from 'a resource is exhausted' — the two paths need completely different evidence. It turned out a post-deploy load spike stacked on a DDoS, and we ran out of file descriptors. We restored service in about ten minutes."
  追問備案(口述):"We stopped the bleeding and chased the root cause in parallel — and we captured the scene before any rollback, because a rollback destroys your evidence."
- **CPU 空轉追到確切 syscall**(✅ 確認:**前人程式碼的陳年 bug,我是追查者**——當 debugging 故事講,不認領也不指責,說 "a long-standing bug in inherited code"):`perf` 熱點在 `read` → `strace` 看到同一個 fd 反覆回傳 0 → read loop 沒把 0-byte 當 EOF。關鍵句逐字:"**A 0-byte read is EOF, not 'no data this time'. Loop exit conditions have to match the syscall contract.**"

**7. 如果重來(口述版,直接唸)**

> "I'd add an explicit backpressure-and-drop ledger on the daemon side. Today I inherit syslog-ng's buffering behavior, so the drop policy isn't in my hands — I want 'how much did we lose' to be a number I own. It's the same dropped-counter shape as any telemetry pipeline."

### 專案三:Mail moderation / content-inspection daemon(Rust / Tokio / Rayon / gRPC)

> **8/7 全節改寫(必讀)**。原稿(7/29)是只憑履歷 + Holdwin 簡報起草的,**§4 那句關鍵句是 Claude 編的、不是你講過的話**(commit `1c13825`),已換成你自己守得住的版本;§2–3 的「Tokio runs a task per matcher」也不準。本次事實全部出自 repo
> `/synosrc/git_source/libsynomailserver-moderation`(Cerberus,daemon `syno_mail_moderation`)的實際碼與 commit,每條註出處。
> **這個 repo 就是 culture fit ★3 的 content-moderation 專案**——William Hsieh 是 repo 第二大貢獻者(106+45 commits),
> 兩個故事同一份 codebase,面試官串起來是加分。
> ⚠ **保密紅線(講法,不是道德課)**:講結構與教訓,不講可被利用的細節。**絕對不碰**:gRPC 曾 fail-open 成明文、
> attachment 路徑曾可任意讀檔(以 root)、token 與 Redis 密碼共用同一份 secret。那是雇主出貨產品的攻擊面,
> 講給第三方公司零加分、風險全歸你。也不報內部 ticket 編號、主機名、檔案路徑。★7 的 authz 故事同一把尺:
> 停在「我漏了授權檢查、後果量級、我怎麼改流程」,不描述怎麼利用。

**1. 90-second elevator**

- "I develop and operate the **mail moderation daemon** on our mail platform: every inbound message gets matched against admin-defined rules across sender, recipient, subject, body, attachments and IP, and I return a block or quarantine decision. Regex, exact-word, IP, and **ML-based PII detection**. The MTA's milter calls me **synchronously, in the delivery path** — so **my latency is the platform's latency, and a crash tempfails real mail**. I also **introduced Rust into the team's stack**."
- ⚠ 履歷若寫 "microservices"(複數):這個 repo 是**一支 daemon + 兩個 transport**(milter 用的 localhost 二進位協定 + 節點間 gRPC)。被追問「哪些 service」要答得出來,別讓複數變成沒下文的字。

**2–3. 問題與設計(口述版,直接唸)**

> "Two things make it hard. First, it's synchronous and in the delivery path — the milter blocks on my answer, so my p99 is the platform's p99. Second, matching is a fan-out: one message runs against tens of matchers across every field, plus every attachment, and the PII matcher is an ONNX model. That's real CPU, not waiting.
>
> So the shape is an async front end with the matching hopped off the async runtime entirely. Tokio owns the two listeners — a length-prefixed binary protocol on localhost for the milter, and gRPC with mTLS to the peer node in an H-A pair. Once a mail is parsed, rule matching goes through `spawn_blocking`, and inside that Rayon fans the matchers out with `par_iter`. The rule set itself is an `ArcSwap` snapshot, so a config reload is an atomic pointer swap — the matcher never waits for the admin."

出處:`src/server/mod.rs:54-66`(pool 切分)、`src/database/rule/rules.rs:161-175`(`spawn_blocking` 跳離 runtime,
含 6 行理由註解)、`src/database/pattern/mod.rs:408,489,545,592`(三層 `par_iter`:~40 matcher / 每附件 14 個
content matcher / per-recipient)、`src/database/cache.rs:15` + `src/matcher/pii.rs:17`(`ArcSwap` / `OnceLock<ArcSwap>`,
commit `d9a53de`——**它取代的是 Rayon worker 裡的全域 `blocking_lock()`,那把鎖把 PII 平行化的效果整個抵消掉**,
這是 trade-off 的好料)。

**4. Trade-off ≥2**

- **Trade-off 1 = pool 切分本身,連代價一起講**(你自己的 commit `66ab3e8`,2025-04-03):`total_cores` 對半,
  一半給 Tokio、一半給 Rayon,兩邊各再 ×2。它解掉的真問題 = CPU 比對卡住 async runtime。**代價你要自己先說出來**:
  8 核變成 16 條執行緒 = 2 倍超訂,而且沒量測過(`docs/TECHNICAL_REVIEW.md` §2.4,LOW-MEDIUM,至今未修)。
- 關鍵句逐字(**8/7 換句,原句是 Claude 編的**):"**Tokio owns the waiting, Rayon owns the computing — but the part people skip is that you now have two thread budgets against one core count. Size them independently and you oversubscribe; and with no admission limit on top, one compute-heavy message holds threads while new connections just queue.**"
  這句是你 8/7 自己講的機制,precise 化而已——**而且它是你 repo 的實況**:掃 `Semaphore|max_concurrent|concurrency_limit|backpressure` 全 `src/` **零命中**,`src/server/command_server.rs:38` 是無上限 accept→spawn。
- 關鍵句二(reload):"**A config reload should be a pointer swap, not a lock.**"
- **Trade-off 2 = 隔壁團隊三個 merged MR**(不同 codebase,邊界講清楚):thread-per-request → **Tokio async**
  (max concurrent connections **~300×**)+ **Rayon** 平行掃描(**12.5× average QPS**,`perf` + load benchmark 驗證)
  + macro-based trait interface;成為他們的 **go-to Rust reviewer**。

**5. 數字(⚠ 出處分級——這節照「不講數字、不掰」的老規矩)**

| 數字 | 能不能講 | 依據 |
|---|---|---|
| ~300× max concurrent connections、12.5× average QPS | ✅ 講 | 隔壁團隊那三個 MR,`perf` + load benchmark 驗過(**說清楚是另一個 service**) |
| **~3,500 msg/s @ 8 cores** | 🟡 **要講就自帶 scope,不能裸講**(8/7 查證後定案) | 出處你自己指認 = 分支 `feat-adaptive-dispatch` 的 harness `src/database/pattern/bench_dispatch.rs`。**它確實印絕對微秒**(`seq (us)` / `par (us)`,`median_ns` 取 31 次中位數),所以當時螢幕上有絕對值——只是 committed 的 results doc **只留了比值,絕對值沒入檔**。反推 3,500/s ⇔ 每封 **286µs**,量級與這個 harness 對得上。🔴 **但它量的是 `dispatch_text_matchers` 單一階段的隔離量測**:檔案自己寫明 **PII 排除在外**(「it is the heaviest matcher」)、**4 核開發機**、**不含** `from_socket` 解析與 `new_moderation` 的 DB 寫入(你自己的文件估 1–10ms 與 5–50ms)。所以 **"the service sustains 3,500 msg/s" 站不住**(被問「這數字包含什麼」會塌);**"the matching stage clocked about 3,500 messages a second in isolation" 站得住**。差別只是一個修飾語,句子自己帶著就安全 |
| 想把那個數字升級成硬證據 | 選配,一行命令 | `BENCH_DISPATCH=1 cargo test --lib database::pattern::bench_dispatch -- --nocapture --test-threads=1` → 把絕對 µs 那兩欄補進 `docs/adaptive-dispatch-crossover-results.md`。做了就能講「量過、記下來了」;沒做就照上面那行自帶 scope 的講法 |
| 「sequential vs `par_iter` 交叉點 ~2000 bytes;≥8 個 active matcher 時 1KB 就有 1.6× 平行勝出;heavy profile 最高 3.58×」 | ✅ 講,但要說「沒 merge」 | `docs/adaptive-dispatch-crossover-results.md`(branch `feat-adaptive-dispatch`),**31 次取中位數、4 核開發機、2026-07-13、PII 排除**。**這是 repo 裡唯一的真實量測,而且比吞吐數字更值錢**:結論是**決定要不要平行的是 active matcher 的數量,不是郵件大小**,因此推翻了團隊沿用的「50KB 交叉點」——那個數字是在只有 ExactMatch(≈sparse profile)的負載上校準的,而 sparse 在整張表上根本永遠不該平行(0.99×~1.00×,差異是雜訊)。落地成 `CostClass` 四檔:PII active → Heavy 一律平行、≤1 active → Light 一律序列、≥8 active → Heavy、2–7 active → Adaptive(subject+body ≥ 2000 bytes 才平行)。**分支未 merge,講成「已上線」會被抓** |
| PII ~100ms/KB、Rayon 排程 ~100μs、P50<10ms/P99<50ms | 🔴 別當實測講 | 前兩者檔案自己標「估計」,第三個是 **target**(`docs/TECHNICAL_REVIEW.md:420`) |
| 0 `unsafe`、~16k LOC、67 個 Rust 檔 | ✅ 可講 | `withers_doc/07_code_review.md:305-317` |

**6. 最難的 bug(口述版,直接唸;~330 words ≈ 2.5m——deep dive 的「walk me through」題,這個長度是對的)**

> "The one I'd pick: real mail started getting tempfailed with a 451, and the only clue was on the *Perl* side — the milter logged a read failure, 'no data received'. From its point of view we accepted the connection and then vanished without answering.
>
> That framing was most of the debugging. Nothing in my request path returns 'no answer' — every branch either decides or errors. So we weren't returning a wrong result, we were dying mid-handler. And a panic in a Tokio worker closes the connection exactly like that.
>
> The root cause was a `dbg!` left in the code. `dbg!` expands to `eprintln!`, and `eprintln!` **panics** if the write to stderr fails. Our stderr is a pipe into the system logger — so once that logger had restarted, the pipe was gone. And the two `dbg!` calls sat in the else-branches of the From and To header extraction, which is the *normal* path for envelope-only mail. So it needed a conjunction: a Cc-only message arriving while the log pipe was broken. Neither one alone does anything, which is why it looked random.
>
> What I shipped was the removal plus a written invariant at that call site — never add a fallback write to stderr on the logging path, we have been bitten here before. Then a week later the same hazard came back from the other direction: an `unwrap` inside a `LazyLock` initializer, so one unavailable log socket poisoned the lock and *every* later log call panicked. That one I fixed properly — in the type. The logger became a `LazyLock<Option<...>>`, so a logger that fails to build makes logging a no-op instead of a landmine.
>
> And the honest ending: the real prevention — a lint that rejects `dbg!` and `eprintln!` in this crate — still isn't there. Our CI runs clippy with `-D warnings`, but those two live in clippy's restriction group and are allow-by-default, so they slip through. It's a two-line attribute I should have added then."

**收尾兩版(8/7 你說「先當作我已經修好了」——只講真的做了的那個版本)**

- **已加 lint 版**(`#![deny(clippy::dbg_macro, clippy::print_stderr)]` 真的進了 crate):把最後一段換成
  > "And the prevention I care about is the one that doesn't rely on memory: the crate now denies `dbg!` and `print_stderr` outright. `-D warnings` alone didn't catch them — both live in clippy's restriction group, allow-by-default — so it had to be an explicit deny. A written invariant tells the next reader; a lint tells the compiler."

  這版更強:它把「comment 級的防止」升級成「機制級的防止」,正好是面試官問「how did you prevent it」想聽的層級。
- **未加 lint 版**(照現況):留原稿最後一段的誠實收尾(「still isn't there… a two-line attribute I should have added then」)。
  它也不弱——**承認自己的修法只到註解層、並當場說出正確做法**,是資深訊號;但**只有在你真的還沒加時才能這樣講**。
- 🔴 鐵律:**面試不能講沒做的事**。上場前 30 秒自問「這句現在是真的嗎」,兩版選一版,不要混。

⚠ **不准編的部分**:commit 記了症狀鏈(mimedefang 的錯誤字串、451、兩個前提條件),**但沒記你當時用什麼工具、
什麼順序查到的**。上面的稿子只從「已記錄的症狀鏈」往下推理(「沒有分支會回『不回答』→ 所以是 handler 中途死」),
這是誠實的。**被追問「你怎麼縮小範圍的」不要生出 profiler / bisect / flamegraph**——repo 全域沒有任何這類工具的痕跡。
就說:從 milter 那句 read failure 反推、對照 handler 的所有回傳路徑。
出處:`e71077d`(2026-05-06,Ref 6362,author witherslin,reviewer joerao)、不變量註解 `src/server/service.rs:293-300`
(「we have been bitten by this before」)、續集 `1ae6363`(2026-05-14,Ref 6384,**由你自己新寫的 `from_socket` 附件測試打出來的**——
第一批走到 `log_*` 的單元測試)、型別修法 `src/server/service.rs:301-321`、CI `.gitlab-ci.yml:42`。

**備用 bug 故事(依用途選,不要一場講三個)**

- **「我錯了」題的最佳解 = DB 連線池**:你把 `max_connections(5)` 改成 `(cores*3).max(10)` + `min_connections=10`
  想解「連線池餓死」(`8e3fe40`),結果被同事在 bug ticket 下 revert 回 5(`07142ee`),你隔天自己寫出根因
  (`892a534`):Cerberus 不直連 PostgreSQL,中間有 pgbouncer `pool_mode=session`、`default_pool_size=5`;
  超出的連線在 pgbouncer 無限排隊,SQLx 的健康檢查 ping 卡在殭屍連線上、**佔住 semaphore permit**,最後
  **全部**查詢 15 秒後 `pool timed out`——比它要解的餓死更糟,而 `min_connections=10` 讓它在啟動就發生。
  在 NAS 上用 **8 個情境的 POC、100% 可重現**驗掉。教訓可直接送給 Etched:**你設的資源上限只是一疊上限的最上層;
  下游一跳有個 session-mode proxy 卡你 5 條時,`num_cpus` 算出來的數字沒有意義**。附帶第二個訊號:
  這個 POC 是去**推翻一條被標成 MEDIUM-HIGH 的 review finding**,不是去證實它。
- **「最好的機制性防止」題 = RegexSet 單位錯置**:1MB `size_limit` 你以為是 per-pattern,實際是 per-`RegexSet`
  (runtime 的編譯單位);超限時 fallback 成 `RegexSet::empty()`,整個欄位的 regex 規則**靜默失效**——
  moderation 產品靜默不比對是 fail-open,不是降級。而且寫入端驗證用的是「單條 pattern」這個不同的單位,
  所以存得進去、只在 runtime 爛掉。三層修法:①把驗證單位對齊編譯單位(`validate_regex_set`)②`Compiled::PerPattern`
  fallback = **降級不消失**,只丟單獨超限的那條、還帶著 index 讓 pattern id 不錯位 ③4 個以不變量命名的回歸測試。
  一句話教訓:**在你實際執行的那個單位上驗證,而且 fallback 要降級不要消失**。⚠ 這題**沒記「怎麼發現的」**,
  別編發現過程。

**7. 如果重來(口述版,直接唸;~200 words ≈ 95s)**

> "One decision: I'd make concurrency an **explicit** limit instead of an emergent one.
>
> What I did was size the two visible thread pools — half the cores to Tokio, half to Rayon. That fixed the real problem it was aimed at: CPU-bound matching stalling the async runtime. What I didn't do was bound how many messages are in flight. So the actual limit in the system turned out to be the database pool — five connections — and we found that out by accident. And it's the wrong failure mode: a saturated pool doesn't shed load, it waits, and times out fifteen seconds later. Because the milter is blocking on me, that surfaces as a tempfail on legitimate mail.
>
> Rebuilt, I'd put one semaphore at accept, sized from the actual bottleneck instead of from core count; give the pool an acquire timeout so saturation fails fast instead of hanging; and export a saturation metric so it's visible before a customer finds it.
>
> **If you don't choose the limit, the system picks one for you — and it won't be the one with a good failure mode.**"

**若你已經回去修了(8/7「先當作我已經修好了」)**:這題**不需要改寫**——它問的是「當初的決定」,不是現況,所以主體照唸。
只在最後一段前加一句,答案會更強:
> "That's not hypothetical, by the way — I went back and put the limit in."

反之若還沒修,**主體一字不動**即可(「rebuilt, I'd…」本來就是條件式,沒有謊)。🔴 同一把尺:那句加了就必須是真的。

**追問深度(留著,別主動全倒)**:①**第三個池沒人管**——`spawn_blocking` 的 blocking pool 從沒設過
`max_blocking_threads`(掃 `src/` 零命中),吃 Tokio 預設上限 512;真實路徑是 tokio worker → blocking pool(無界)
→ Rayon global pool,**被仔細算過的 50/50 核心切分只管到頭尾兩個,中間那個是無界的**。這條是你自己的分析、
review 文件沒寫,拿來收尾很強。②Rayon 迴圈裡還有同步、無大小上限的檔案讀取
(`src/database/pattern/mod.rs:494`,`std::fs::read_to_string(...).unwrap_or_default()`)——大附件可能 OOM,
且讀取錯誤被靜默吞掉。
**備用第 7 答(被追問流程/架構題時用)= refactor 的順序**:你把 5 個 domain service trait 照 strangler-fig 寫在舊
`ServiceExt` god-trait 旁邊,理由寫成「zero-risk deployment」。結果 8 個月後 `src/server/domain/` 的 1,611 行
**連 `mod domain` 都沒宣告**(`src/server/mod.rs:1-7`),等於沒編譯;`TransactionManager` 掛著 `#[allow(unused)]`
和 `// Todo(withers)`、production 零呼叫者;而 `grpc_service.rs` 與 `sync_service.rs` 是同一套冪等+交易邏輯的
兩份逐行複製(review 評 [嚴重],並指出真正代價 = 「未來修 bug 只改一份」+「給 reviewer『已重構』的錯覺」)。
教訓是**順序**:「零風險」是假的——把目標架構寫在舊架構旁邊、卻一條呼叫路徑都不接,等於把零風險換成永久重複
加上假的完工訊號。**該做的是一次搬一條垂直切片、並在同一個 MR 裡刪掉舊路徑,讓 refactor 不可能半途而廢。**
⚠ 這個答案要以「我沒收尾的工作」自己認,不能怪 reviewer;風險是聽起來「不收尾」,所以它是備用不是首選。

### 備用口袋:kv-storage-rs(個人專案)

- 一句話備著:"A Redis-command-compatible in-memory KV engine in Rust — 60 commands, TTL, RDB snapshots, **nanosecond-level benchmarks**; currently adding a RESP-over-TCP server." 被問 side project 或 Rust 深度時再展開。
