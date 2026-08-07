# Deep dive 練習本(問題 + 要唸的英文 + 中文意思)

**這個檔只放三樣東西:面試官會問的問題、你要講的英文、那段英文的中文意思。**
證據、出處、追問彈藥、保密紅線 → 全部搬到 [deep-dive-notes.md](deep-dive-notes.md)。
怎麼練 → [practice-method.md](practice-method.md)。

場次:**8/10(一)10:00–10:45 technical deep dive(Ulysses Kao)——這天只有這一場,沒有 debrief。**
(recruiter debrief 改到 **8/11 10:00–10:15**,接在 Jan 的 coding 場之後。)
講的順序:**專案二 → 專案一 → 專案三**(專案二對 Etched 命中率最高,它就是一條 telemetry pipeline)。

每個專案七個問題,順序固定,面試官幾乎一定照這個走:

| # | 問題 | 中文 |
|---|---|---|
| 1 | What is it? | 90 秒:這是什麼、為誰做、多大 |
| 2 | What made it hard? | 難在哪(不是「我做了什麼」,是「什麼卡住別人」) |
| 3 | Walk me through the design. | 架構,資料流從進到出 |
| 4 | Why X over Y? | 至少兩個 trade-off,每個都要有被否決的對手 |
| 5 | What are the numbers? | 規模、延遲、前後對比 |
| 6 | Hardest bug — walk me through it. | 怎麼發現、怎麼縮小、根因、之後怎麼從機制上防止 |
| 7 | What would you change? | 一個具體會改的決定 |

---

# 專案二:Log-ingestion pipeline + C++ daemon(先講這個)

## Q1. What is it?(90 秒)

> "I own the **tail of the mail platform's log pipeline**: services log through syslog, **syslog-ng** does the routing and filtering, and a **C++ daemon I wrote consumes that stream, parses and normalizes it, and lands it in a database** so diagnostics are queryable instead of grep-able.
>
> Alongside it, a **shared logging library** standardized how services emit diagnostics — that cut roughly **30% of duplicated logging code**.
>
> It's a telemetry pipeline: many producers, a routing layer, one structured sink — the same shape as an event-loop-over-hardware-signals problem, just with logs instead of interrupts."

**中文意思**

1. 我負責這個郵件平台 log 管線的**尾端**:各服務透過 syslog 寫 log,syslog-ng 做路由和過濾,而**我寫的一支 C++ daemon 消費那條串流、解析並正規化、最後寫進資料庫**——讓診斷從「用 grep 撈」變成「可以查詢」。
2. 另外還有一個**共用的 logging library**,統一了各服務發 log 的方式,砍掉大約 **30% 的重複程式碼**。
3. 它本質就是一條 telemetry pipeline:很多 producer、一層路由、一個結構化的 sink——**跟「用 event loop 處理硬體訊號」是同一個形狀**,只是流過來的是 log 不是中斷。

> 第 3 句是**給 Etched 聽的定位句**,一定要講。

## Q2. What made it hard?

> "Before this, diagnostics lived in plain-text logs scattered across services, each with its own format — debugging one cross-service issue meant grepping three formats and joining them in your head. We needed a structured, queryable landing zone — and the collection path must never interfere with the mail-delivery hot path."

**中文意思**:在這之前,診斷資訊散在各服務的純文字 log 裡,每個格式還都不一樣——要查一個跨服務的問題,你得 grep 三種格式、然後在腦袋裡把它們接起來。我們需要一個結構化、可查詢的落地區,而且**蒐集這條路絕對不能干擾郵件投遞的熱路徑**。

## Q3. Walk me through the design.

> "I stood on the syslog ecosystem instead of fighting it: transport and routing go to syslog-ng — battle-tested, config-driven. My daemon owns exactly two things: the parsing-and-normalization logic, and the database schema. Asio drives the daemon's IO."

**中文意思**:我選擇站在 syslog 生態上,而不是跟它對打:傳輸和路由交給 syslog-ng——它久經考驗、用設定檔驅動。**我的 daemon 只擁有兩件事:解析與正規化的邏輯,以及資料庫 schema。** daemon 的 IO 由 Asio 驅動。

## Q4. Why X over Y?(三個 trade-off)

**① 重用 syslog-ng,而不是自己寫 collector**

> "The off-the-shelf transport is battle-tested and cheap to walk away from; the cost is being bound to syslog's format and delivery semantics. I didn't build a transport — the interesting problem was parsing and schema, so that's where my code lives."

**中文意思**:現成的傳輸層久經考驗、而且要抽身也很便宜;代價是被綁在 syslog 的格式和投遞語意上。我沒有去做傳輸層——**有意思的問題在解析和 schema,所以我的程式碼長在那裡。**

**② 獨立 daemon,而不是讓 syslog-ng 直接寫 DB**

> "Parsing rules and schema are code — they evolve and they need tests. I didn't want that living in a config layer. And the failure modes decouple: if my daemon dies, syslog-ng still buffers."

**中文意思**:解析規則和 schema 是程式碼——它們會演進、需要測試,我不想讓那種東西住在設定檔那一層。而且**失效模式被解耦了:我的 daemon 掛掉,syslog-ng 還在幫我緩衝。**

**③ 結構化落 DB,而不是留純文字**

> "Query power versus write cost. And I won't invent a throughput number — I don't have it from memory; the bottleneck by construction is the DB write path, not the parse."

**中文意思**:這是「查詢能力」換「寫入成本」。而且吞吐數字我不會編——我記不得確切數字;但**從結構上看,瓶頸一定在 DB 寫入端,不在解析。**

## Q5. What are the numbers?

> "The number I'll stand behind is the library one: roughly **thirty percent less duplicated logging code** across services. For pipeline throughput I don't have a measurement I trust from memory, so I won't invent one — by construction the bottleneck is the database write path, not the parsing."

**中文意思**:我敢背書的數字是 library 那個:各服務的重複 logging 程式碼少了大約**三成**。至於管線吞吐,我沒有一個記得住又敢信的量測,所以我不編——從結構上看瓶頸在 DB 寫入端,不在解析。

> 「我不編數字」這句本身是加分,不是扣分。

## Q6. Hardest bug — walk me through it.(兩個故事,現場挑一個講)

**① FD 耗盡,全服務中斷,10 分鐘內復原**

> "The process was alive, but every new connection failed. First move: split 'the program is wrong' from 'a resource is exhausted' — the two paths need completely different evidence. It turned out a post-deploy load spike stacked on a DDoS, and we ran out of file descriptors. We restored service in about ten minutes.
>
> We stopped the bleeding and chased the root cause in parallel — and we captured the scene before any rollback, because a rollback destroys your evidence."

**中文意思**

1. 程式還活著,但每一條新連線都失敗。**第一個動作:把「程式錯了」和「資源耗盡了」分開**——這兩條路要找的證據完全不同。結果是一次部署後的流量尖峰疊上一場 DDoS,file descriptor 用光了。我們大約十分鐘內恢復服務。
2. 我們一邊止血、一邊並行追根因;而且**在任何 rollback 之前先把現場保存下來,因為 rollback 會把你的證據銷毀掉。**

**② CPU 空轉,追到確切的 syscall**

> "The other one: a service was burning CPU while doing nothing useful. `perf` put the hotspot in `read`. Then `strace` showed the same file descriptor returning zero bytes over and over — and the read loop treated zero as 'nothing to read right now' and went around again. A zero-byte read is EOF, not 'no data this time'.
>
> This was a long-standing bug in code I inherited, and the fix was one condition. The lesson I kept: **a loop's exit condition has to match the syscall's contract, not your intuition about it.**"

**中文意思**

1. 另一個:一個服務在燒 CPU 但什麼有用的事都沒做。`perf` 指出熱點在 `read`。接著 `strace` 顯示同一個 file descriptor 一直回傳 0 bytes——而那個 read loop 把 0 當成「這次剛好沒資料」,就繞回去再讀一次。**0-byte read 是 EOF,不是「這次沒資料」。**
2. 這是我接手的舊程式碼裡一個陳年 bug,修法只是一個判斷條件。我留下的教訓是:**迴圈的離開條件必須對上 syscall 的合約,不是你對它的直覺。**

> ⚠ 講「a long-standing bug in code I inherited」——不認領也不指責前人。

## Q7. What would you change?

> "I'd add an explicit backpressure-and-drop ledger on the daemon side. Today I inherit syslog-ng's buffering behavior, so the drop policy isn't in my hands — I want 'how much did we lose' to be a number I own. It's the same dropped-counter shape as any telemetry pipeline."

**中文意思**:我會在 daemon 這側加一本明確的**背壓與丟棄帳**。現在我是繼承 syslog-ng 的緩衝行為,所以丟棄政策不在我手上——我要讓「我們丟了多少」變成**一個我自己擁有的數字**。這跟任何 telemetry pipeline 裡的 dropped counter 是同一個形狀。

---

# 專案一:Two-node Active-Active HA(第二個講)

## Q1. What is it?(90 秒)

> "I built the **application-state replication layer** for a two-node Active-Active mail-security platform — the layer that keeps **human approval decisions** consistent across two nodes, when the network between them can **drop, stall, or duplicate messages**.
>
> The platform already replicated *data* three ways; **nobody replicated what the application had *decided***. That was the gap I filled.
>
> In production **six-plus months, zero reported data-inconsistency incidents**, and it survived **one real node failover** with no mail delay."

**中文意思**

1. 我做的是一個雙節點 Active-Active 郵件安全平台的**應用狀態複製層**——負責讓**人工審核的決定**在兩個節點之間保持一致,而它們之間的網路**會掉包、會卡住、會重複投遞**。
2. 這個平台原本已經有三套**資料**複製了,但**沒有人複製「應用程式決定了什麼」**。那就是我填上的缺口。
3. 上線**六個多月、零件資料不一致事故回報**,而且**撐過一次真實的節點 failover**,郵件收發沒有延遲。

## Q2. What made it hard?

> "Four constraints, and they fight each other.
>
> Exactly **two nodes** — that's the product form factor, a third machine is not an option. It runs **on customer premises**, and **mail flow must never stop**: if the link between the nodes drops at three in the morning, both nodes keep accepting mail independently and reconcile later — pausing to wait for a human is not an option. The network between them drops, stalls, and duplicates, and there is **no trusted global clock**. And either node can die, so service has to continue and the states have to converge after recovery."

**中文意思**

1. 四個限制,而且它們互相打架。
2. **就只有兩台**——那是產品形態,第三台機器不在選項裡。它**跑在客戶自己的機房**,而且**郵件流絕對不能停**:如果凌晨三點兩台之間的連線斷了,兩邊都要繼續獨立收信、事後再對帳——「停下來等人處理」不是選項。它們之間的網路會掉包、會卡住、會重複,而且**沒有可信的全域時鐘**。任一台都可能死掉,所以服務必須繼續,而且**恢復之後狀態必須收斂**。

## Q3. Walk me through the design.

> "An operation log, delivered over Redis Streams, with a task dispatcher on each side; HAProxy in front for traffic.
>
> The core is **at-least-once delivery plus idempotent apply**. The sender can never distinguish 'the peer never got it' from 'the ACK was lost' — that's the two-generals problem — so instead of trying to make delivery exact, I made **redelivery harmless**. On the receiving side: if the message ID is already in the processed table, ACK without re-executing; otherwise **execute the business logic and record the message ID in the same transaction**.
>
> The check and the write have to be one atomic operation — otherwise two concurrent retries both see 'not processed' and double-execute. **Idempotency that isn't atomic only *looks* like idempotency.**
>
> And split-brain resolution is derived from **operation semantics**, not timestamps. An approval that already sent the mail beats a later reject — **you can't recall a sent email by overwriting a row. The invariant lives in the real world, not in the database. Irreversible actions win.**"

**中文意思**

1. 一份操作日誌,透過 Redis Streams 投遞,兩邊各有一個 task dispatcher;前面用 HAProxy 導流量。
2. 核心是**至少一次投遞 + 冪等套用**。發送端永遠分不出「對方根本沒收到」和「ACK 掉了」——那就是兩軍問題——所以我不去追求「精確投遞」,而是讓**重送本身無害**。收端:如果 message ID 已經在已處理表裡,直接 ACK、不重跑;否則**在同一個交易裡執行業務邏輯並記下 message ID**。
3. **檢查和寫入必須是同一個原子操作**——否則兩個並行的重試都會看到「還沒處理」然後雙重執行。**不是原子的冪等,只是「看起來像」冪等。**
4. 而 split-brain 的解決規則是從**操作語意**推出來的,不是從時間戳。**已經把信寄出去的核准,贏過比較晚的退回——你不可能靠覆寫一列資料把寄出去的信收回來。不變量活在現實世界,不在資料庫裡。不可逆的操作贏。**

> 第 3 句和第 4 句是這個專案的兩顆金句,要能單獨脫口而出。

## Q4. Why X over Y?(四段,前三段是被否決的對手,第四段主動講代價)

**① vs PostgreSQL 內建複製**

> "We were on Postgres fourteen. Streaming replication is single-writer, so no active-active; and bidirectional logical replication wasn't available yet — that needs Postgres sixteen. But the deeper reason holds on any version: **replication has no notion of which write should win.** It can't order two partitioned writes, and it certainly doesn't know that one of them already sent an email.
>
> **Replication is a transport-layer concern; conflict resolution is a policy-layer concern.** Off-the-shelf tools move the data, but they can't decide which write should win."

**中文意思**

1. 我們用的是 Postgres 14。Streaming replication 是單寫者,所以做不了 active-active;而雙向邏輯複製當時還沒有——那要 Postgres 16。但更根本的理由在任何版本都成立:**複製機制沒有「哪一個寫入該勝出」這個概念。** 它沒辦法為分區期間的兩個寫入排序,更不會知道其中一個已經把信寄出去了。
2. **複製是傳輸層的事,衝突解決是政策層的事。** 現成工具能搬資料,但它決定不了誰該贏。

**② vs Raft / Paxos**

> "With Raft or Paxos at n equals two, the quorum is two — lose either node and the whole system stops writing. Availability would be worse than a single machine, and a third node wasn't an option in this product. So I changed the question: instead of asking *who is allowed to write*, both sides write, replay-safe, and converge afterwards."

**中文意思**:在 n 等於 2 的情況下用 Raft 或 Paxos,quorum 就是 2——任一台掉了,整個系統就不能寫了,**可用性會比單機還差**,而這個產品不能加第三台。所以我把問題換掉:不去問「誰有資格寫」,而是**兩邊都寫、可安全重播、事後收斂**。

**③ vs Last-Write-Wins**

> "Last-write-wins assumes a trusted clock we don't have — and 'later' doesn't mean 'should win'. It silently overwrites operations that already had real-world consequences."

**中文意思**:Last-write-wins 假設有一個我們沒有的可信時鐘——而且**「比較晚」不等於「該贏」**。它會默默覆寫掉那些已經在現實世界產生後果的操作。

**④ 主動講自己的代價**

> "And I'll volunteer the cost: there is an inconsistency window before convergence, and the conflict rules are a policy I defined and had to validate myself — there's no textbook to point at."

**中文意思**:代價我自己講:**收斂之前有一段不一致的視窗**,而那些衝突規則是我自己定、自己驗證的政策——沒有教科書可以指。

## Q5. What are the numbers?

> "Two nodes, six-plus months in production, one real failover with no mail delay. The cost of version three is one extra local write per operation.
>
> And on how I know it's consistent — **we don't just hope it's consistent. The system detects divergence, re-syncs automatically, and counts every occurrence. Across every customer machine, for six-plus months, that counter stayed at zero — and support never received a single inconsistency report.**"

**中文意思**

1. 兩個節點、上線六個多月、一次真實 failover 且郵件無延遲。v3 的代價是**每個操作多一次本地寫入**。
2. 至於我怎麼知道它一致——**我們不是「希望」它一致。系統會偵測分歧、自動重新同步、並且把每一次都記下來。全客戶機器、六個多月,那個計數器一直是零,而且 support 從來沒收到一件不一致的回報。**

> 「你怎麼知道零不一致?」這題一定會被問。第 2 段就是答案,而且它是武器不是弱點。

## Q6. Hardest bug — walk me through it.(演進本身就是證據)

> "Version one used shared NFS — after a link flap the file locks were unreliable, and the shared storage itself was a single point of failure.
>
> Version two moved to command queues on each node's local database — but if the queue drained or the consumer restarted, operations that hadn't landed on the peer were simply gone.
>
> That loss window forced version three: back up locally before dispatch, and only clear the backup after the peer confirms it landed.
>
> **Each version was forced by a concrete failure, not by a whiteboard.**"

**中文意思**

1. v1 用共享 NFS——連線抖動之後檔案鎖變得不可靠,而且共享儲存本身就是單點故障。
2. v2 改成每個節點本地資料庫上的命令佇列——但如果佇列被清空或 consumer 重啟,**那些還沒落到對方的操作就直接消失了**。
3. 就是這個遺失視窗逼出了 v3:**派送前先在本地備份,而且只有在對方確認落地之後才清掉備份。**
4. **每一個版本都是被一次具體的失敗逼出來的,不是在白板上想出來的。**

## Q7. What would you change?

> "I'd break the two-node symmetry with a **witness** — a lightweight third party that stores no data but can arbitrate — or a fencing token. With that, the genuinely hard cases stop being judgment calls and become decidable.
>
> I didn't have it because the product ships as exactly two boxes on a customer's premises; a third component wasn't on the table. So I'll be honest: **that's a design boundary I accepted, not a problem I solved.**"

**中文意思**

1. 我會用一個 **witness** 來打破雙節點的對稱性——一個不存資料、但可以當裁判的輕量第三方——或者用 fencing token。有了它,那些真正困難的情況就從「靠判斷」變成「可判定」。
2. 我當時沒有,是因為這個產品就是出兩台機器放在客戶機房,第三個元件不在選項裡。所以我誠實說:**這是我接受的設計邊界,不是我解掉的問題。**

---

# 專案三:Mail moderation / content-inspection daemon(第三個講)

> ⚠ 這個專案有**保密紅線**,講之前先看 [deep-dive-notes.md](deep-dive-notes.md) 的紅線那節。
> 一句話版:**講結構與教訓,不講可被利用的細節。**

## Q1. What is it?(90 秒)

> "I develop and operate the **mail moderation daemon** on our mail platform: every inbound message gets matched against admin-defined rules across sender, recipient, subject, body, attachments and IP, and I return a block or quarantine decision. Regex, exact-word, IP, and **ML-based PII detection**.
>
> The MTA's milter calls me **synchronously, in the delivery path** — so **my latency is the platform's latency, and a crash tempfails real mail**.
>
> I also **introduced Rust into the team's stack**."

**中文意思**

1. 我開發並維運這個平台的**郵件審核 daemon**:每一封進來的信,都要對管理員定義的規則做比對——寄件者、收件者、主旨、內文、附件、IP——然後我回傳「攔下」或「送審隔離」的決定。規則型態有 regex、完全字串、IP,還有**機器學習的個資偵測**。
2. MTA 的 milter 是**同步呼叫我的,而且就在投遞路徑上**——所以**我的延遲就是平台的延遲,而我如果崩潰,真實郵件會被暫時退回。**
3. 我也是**把 Rust 帶進團隊技術棧的人**。

## Q2 + Q3. What made it hard? / Walk me through the design.

> "Two things make it hard. First, it's synchronous and in the delivery path — the milter blocks on my answer, so my p99 is the platform's p99. Second, matching is a fan-out: one message runs against tens of matchers across every field, plus every attachment, and the PII matcher is an ONNX model. That's real CPU, not waiting.
>
> So the shape is an async front end with the matching hopped off the async runtime entirely. Tokio owns the two listeners — a length-prefixed binary protocol on localhost for the milter, and gRPC with mTLS to the peer node in an H-A pair. Once a mail is parsed, rule matching goes through `spawn_blocking`, and inside that Rayon fans the matchers out with `par_iter`. The rule set itself is an `ArcSwap` snapshot, so a config reload is an atomic pointer swap — the matcher never waits for the admin."

**中文意思**

1. 有兩件事讓它難。第一,它是**同步的、而且在投遞路徑上**——milter 阻塞著等我的答案,所以**我的 p99 就是平台的 p99**。第二,比對是一個 fan-out:一封信要對上幾十個 matcher、跨所有欄位、再加上每個附件,而個資 matcher 是一個 ONNX 模型。**那是真的在算,不是在等。**
2. 所以形狀是:**一個 async 的前端,而比對整個跳出 async runtime。** Tokio 擁有兩個 listener——給 milter 的、走 localhost 的長度前綴二進位協定,以及 HA 對節點之間、帶 mTLS 的 gRPC。信一解析完,規則比對就走 `spawn_blocking`,而在那裡面 Rayon 用 `par_iter` 把 matcher 攤開跑。規則集本身是一個 `ArcSwap` 快照,所以**設定重載是一次原子的指標交換——比對永遠不會卡在等管理員。**

## Q4. Why X over Y?(兩個 trade-off)

**① 執行緒池切分——連代價一起講**

> "The trade-off I'd put first is the thread-pool split, and I'll give you the cost with it. I split the cores in half — one half sizes the Tokio runtime, the other sizes the Rayon pool — and then doubled each. It solved the real problem it was aimed at: CPU-bound matching was stalling the async runtime.
>
> The cost is that on an eight-core box that's sixteen threads, so two-times oversubscription — and I never measured whether that multiplier was right. It's still an open item in our review doc.
>
> **Tokio owns the waiting, Rayon owns the computing — but the part people skip is that you now have two thread budgets against one core count. Size them independently and you oversubscribe; and with no admission limit on top, one compute-heavy message holds threads while new connections just queue.**"

**中文意思**

1. 我會先講執行緒池的切分,而且**代價我自己一起給**。我把核心數對半切——一半決定 Tokio runtime 的大小,另一半決定 Rayon 池的大小——然後兩邊各再乘二。它解掉了它要解的真問題:**CPU 密集的比對會讓 async runtime 停擺。**
2. 代價是:**八核機器上那就是十六條執行緒,兩倍超訂**——而我從來沒量過那個乘二是不是對的。它到今天還掛在我們的 review 文件上沒解。
3. **Tokio 管等待,Rayon 管運算——但大家會跳過的是:你現在有兩份執行緒預算,對著同一個核心數。各自獨立去 size 就會超訂;而上面又沒有入場上限的話,一封重運算的信佔住執行緒,新連線就只能排隊。**

**② 隔壁團隊的三個 merged MR(不同 codebase,邊界要講清楚)**

> "A different kind of trade-off, from a partner team's Rust service — I contributed three merged changes there. Thread-per-request to Tokio async, which took max concurrent connections up by about **three hundred times**; Rayon for the parallel scanning path, about **twelve and a half times** the average QPS, verified with `perf` and a load benchmark; and a macro-based trait interface. I ended up as their **go-to Rust reviewer**."

**中文意思**:另一種 trade-off,發生在隔壁團隊的 Rust 服務——我在那邊有三個 merged 的改動。thread-per-request 改成 Tokio async,最大並行連線數上去大約**三百倍**;平行掃描那條路用 Rayon,平均 QPS 大約**十二點五倍**,用 `perf` 加上壓力測試驗過;還有一個 macro 型的 trait 介面。最後我成了他們**遇到 Rust 就找的 reviewer**。

> ⚠ 一定要說清楚這是**另一個 service**,不要跟你自己的 daemon 混在一起。

## Q5. What are the numbers?

**被問到吞吐或「你怎麼量的」,講這一段就好,講完立刻把話題帶到天花板在哪:**

> "We measured it with the daemon's own counter — it exports a Prometheus counter, so we read the delta over a window instead of trusting the client — plus CPU accounting from `/proc` to see where the time actually went. **Order of a few thousand a second per node.**
>
> And the interesting part is that the ceiling turned out to be the **request path**, not the matching: it's one TCP connection per message, so it's syscall-bound. Throughput peaked around **thirty-two** concurrent connections and dropped off after that."

**中文意思**

1. 我們是用 **daemon 自己的計數器**量的——它會輸出一個 Prometheus counter,所以我們讀的是一段時間內的差值,而不是相信 client 自己數的;再加上從 `/proc` 算 CPU 帳,看時間到底花在哪。**量級是每個節點每秒幾千封。**
2. 而有意思的地方是,天花板結果不在比對,而在**請求路徑**:每封信一條 TCP 連線,所以它是被 syscall 綁住的。吞吐在大約**三十二個並行連線**時到頂,之後就往下掉。

> **為什麼這樣就夠**:①「用 daemon 自己的 counter 而不是 client 自己數」= 一句話證明你懂量測方法,這是面試官真正在聽的;②`/proc` 的 CPU 帳 = 你有看時間去哪;③**主動說出天花板在請求路徑,面試官就不會再追那個數字**,他會去追更有意思的那件事,而那件事你答得出來。
> **不要**主動報精確數字、不要主動解釋核心數。`order of a few thousand a second` 就是對的粒度。

## Q6. Hardest bug — walk me through it.(~2.5 分鐘,這題長是對的)

> "The one I'd pick: real mail started getting tempfailed with a 451, and the only clue was on the *Perl* side — the milter logged a read failure, 'no data received'. From its point of view we accepted the connection and then vanished without answering.
>
> That framing was most of the debugging. Nothing in my request path returns 'no answer' — every branch either decides or errors. So we weren't returning a wrong result, we were dying mid-handler. And a panic in a Tokio worker closes the connection exactly like that.
>
> The root cause was a `dbg!` left in the code. `dbg!` expands to `eprintln!`, and `eprintln!` **panics** if the write to stderr fails. Our stderr is a pipe into the system logger — so once that logger had restarted, the pipe was gone. And the two `dbg!` calls sat in the else-branches of the From and To header extraction, which is the *normal* path for envelope-only mail. So it needed a conjunction: a Cc-only message arriving while the log pipe was broken. Neither one alone does anything, which is why it looked random.
>
> What I shipped was the removal plus a written invariant at that call site — never add a fallback write to stderr on the logging path, we have been bitten here before. Then a week later the same hazard came back from the other direction: an `unwrap` inside a `LazyLock` initializer, so one unavailable log socket poisoned the lock and *every* later log call panicked. That one I fixed properly — in the type. The logger became a `LazyLock<Option<...>>`, so a logger that fails to build makes logging a no-op instead of a landmine."

**中文意思**

1. 我會挑這個:真實郵件開始被暫時退回、回 451,而**唯一的線索在 Perl 那一側**——milter 記了一筆讀取失敗,「沒收到資料」。從它的角度看,我們接了連線然後就人不見了、沒有回答。
2. **那個框架就是這次 debug 的大半。** 我的請求路徑裡沒有任何分支會回「不回答」——每個分支要嘛做出決定、要嘛回錯誤。所以我們不是回錯結果,**我們是在 handler 中途死掉**。而 Tokio worker 裡的一個 panic,關連線的樣子就正是這樣。
3. 根因是留在程式裡的一個 `dbg!`。`dbg!` 會展開成 `eprintln!`,而 `eprintln!` 在**寫 stderr 失敗時會 panic**。我們的 stderr 是一條通往系統 logger 的 pipe——所以那個 logger 一重啟過,pipe 就沒了。而那兩個 `dbg!` 剛好在 From 和 To 標頭抽取的 else 分支裡,**那是只有信封資訊的信會走的「正常」路徑**。所以它需要兩件事同時發生:一封只有 Cc 的信,碰上 log pipe 斷掉。**單獨任一個都不會出事,這就是它看起來很隨機的原因。**
4. 我當時交出去的是「移掉它」加上在那個位置寫下一條不變量——**這條 logging 路徑上永遠不要加 stderr 的備援寫入,我們在這裡被咬過**。然後一個星期後同一個危險從另一邊回來:一個 `LazyLock` 初始化裡的 `unwrap`,於是一個連不上的 log socket 就讓那把鎖中毒,**後面每一次 log 呼叫都 panic**。那次我修得對了——**修在型別上**。logger 變成 `LazyLock<Option<...>>`,所以一個建不起來的 logger 只會讓 logging 變成 no-op,而不是變成地雷。

> **收尾要不要加「lint 還沒加」那段**,看你到時候真的加了沒有 —— 兩種收尾在 [deep-dive-notes.md](deep-dive-notes.md)。鐵律:**只講真的做了的那個版本。**
> ⚠ 被追問「你怎麼縮小範圍的」**不要生出 profiler / bisect / flamegraph**。就說:從 milter 那句讀取失敗反推、對照 handler 的所有回傳路徑。

## Q7. What would you change?(~95 秒)

> "One decision: I'd make concurrency an **explicit** limit instead of an emergent one.
>
> What I did was size the two visible thread pools — half the cores to Tokio, half to Rayon. That fixed the real problem it was aimed at: CPU-bound matching stalling the async runtime. What I didn't do was bound how many messages are in flight. So the actual limit in the system turned out to be the database pool — five connections — and we found that out by accident. And it's the wrong failure mode: a saturated pool doesn't shed load, it waits, and times out fifteen seconds later. Because the milter is blocking on me, that surfaces as a tempfail on legitimate mail.
>
> Rebuilt, I'd put one semaphore at accept, sized from the actual bottleneck instead of from core count; give the pool an acquire timeout so saturation fails fast instead of hanging; and export a saturation metric so it's visible before a customer finds it.
>
> And I have the curve for it — throughput peaks at thirty-two concurrent and *drops by half* at sixty-four.
>
> **If you don't choose the limit, the system picks one for you — and it won't be the one with a good failure mode.**"

**中文意思**

1. 一個決定:我會把並發上限做成**顯式的**,而不是讓它自己長出來。
2. 我當時做的是把兩個看得見的執行緒池 size 好——一半核心給 Tokio、一半給 Rayon。它解掉了它要解的真問題:CPU 密集的比對讓 async runtime 停擺。**我沒做的是限制同時有幾封信在飛。** 結果系統裡真正的上限變成了資料庫連線池的**五條連線**,而且我們是**意外發現的**。而它的失效模式是錯的:飽和的池子不會卸載流量,它會等,然後十五秒後 timeout。**因為 milter 阻塞在我身上,那件事最後長出來的樣子就是正常郵件被暫時退回。**
3. 重做的話,我會在 accept 那裡放一個 semaphore,大小是從**真正的瓶頸**推出來的、不是從核心數;給連線池一個取得超時,讓飽和**快速失敗而不是吊著**;再輸出一個飽和度指標,讓它在客戶發現之前就看得到。
4. 而且**我有那條曲線**——吞吐在三十二個並行時到頂,到六十四**掉一半**。
5. **如果你不去選那個上限,系統會替你選一個——而它不會是失效模式好的那個。**

---

# 加分料:一個我這週才發現、而且推翻自己的量測(~70 秒)

> 這段是**主動端出來的**,不是被問到才講。放在專案三講完、或面試官問「還有什麼想聊的」時用。
> 完整證據與數字在 [deep-dive-notes.md](deep-dive-notes.md)。

> "One more, because I only found it this week and it changed my mind about my own work.
>
> I'd calibrated a parallel-versus-sequential threshold at two kilobytes, from a benchmark I ran in a **debug build**. When I re-ran the same harness in release, the matcher work got roughly **twenty-five times cheaper** — but the fan-out cost barely moved, because **that's synchronization, not computation**.
>
> So the crossover shifted by more than an order of magnitude, out to somewhere between ten and fifty kilobytes. Which means my shipped constant would have picked parallel for every realistic mail size — and at one kilobyte, parallel is about **thirty times slower** than just doing it inline.
>
> The number I'd confidently overturned — an old fifty-kilobyte heuristic — turned out closer to right than mine.
>
> And the lesson isn't the number: **a threshold that separates computation from coordination can't be calibrated in a build that only speeds up one of them.**"

**中文意思**

1. 再補一個,因為我這週才發現,而且它改變了我對自己工作的看法。
2. 我把一個「平行還是序列」的門檻校準在兩千位元組,依據是我在 **debug build** 上跑的一個 benchmark。當我用 release 重跑同一支 harness,matcher 的工作變便宜了大約**二十五倍**——但 fan-out 的成本幾乎沒動,**因為那是同步,不是運算。**
3. 所以交叉點位移了超過一個數量級,跑到十到五十 KB 之間。也就是說**我出的那個常數,會在所有現實郵件大小上都選擇平行**——而在一 KB 的時候,平行比直接同步做**慢大約三十倍**。
4. 那個被我很有信心推翻掉的數字——一個舊的五十 KB 啟發式——結果比我的更接近事實。
5. 而教訓不是那個數字:**一個用來分隔「運算」和「協調」的門檻,不可能在一個只加速其中一邊的 build 上校準出來。**

> ⚠ 誠實邊界:那條分支**還沒 merge**,所以**沒有 production bug**。要說 "before that branch merges, the threshold has to be derived, not baked."

---

# 備用口袋:kv-storage-rs(個人專案)

被問 side project 或想深挖 Rust 時才拿出來:

> "A Redis-command-compatible in-memory KV engine in Rust — sixty commands, TTL, RDB snapshots, nanosecond-level benchmarks; currently adding a RESP-over-TCP server."

**中文意思**:一個 Rust 寫的、指令相容 Redis 的記憶體 KV 引擎——六十個指令、TTL、RDB 快照、奈秒級的 benchmark;現在正在加 RESP over TCP 的 server。
