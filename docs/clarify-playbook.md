# clarify playbook —— 開場五問,答案決定設計

定位:pillar 1(Clarify)的操作手冊。JD 已經把線索寫給你(node 限制、
network flakiness、跨 rack 規模)——面試官等你把線索變成問題,把答案變成
設計決策。**紀律:0–5 分鐘問完、鎖 contract、之後不中途改設計。**

互動版:[`artifacts/clarify_playbook.html`](artifacts/clarify_playbook.html)
(選答案,看設計跟著變)。
卡片練習:`rehearsals/clarify-cards.md`(6 張情境卡;答案在
`rehearsals/clarify-answers.md`,**寫完才開**)。

## JD 線索 → 該問的問題

| JD 裡的字眼 | 觸發哪一問 | 為什麼 |
|---|---|---|
| "thousands of nodes" / "per rack" | Q3(規模)、Q2(速率) | thread 數、shard 策略 |
| "network flakiness" | Q5(偵測)、Q1(掉不掉) | 資料會遲到/亂序/斷流 |
| "can't store them all" / "memory constrained" | Q1(full policy) | 這是 space complexity 題 |
| "real-time" / "dashboards" | Q4(SLA)、Q1 | 新鮮度 vs 完整性 |
| "hardware signals" / "interrupts" | Q1、Q3 | 上游推不回去 |

## 五問決策表

### Q1:資料掉不掉?(最重要——決定整個 full policy)

問法:*"Under pressure, can we drop or aggregate, or is every sample required?"*

判準一句話:**上游推不推得回去?**

| 答案 | full policy | 形狀 |
|---|---|---|
| 一筆不能掉(命令、計費) | **backpressure** | bounded queue;上游是自己人 → blocking submit;是 TCP 對端 → `EPOLL_CTL_MOD` 關掉 EPOLLIN,消化完再開 |
| 新資料比舊值錢(行情、遙測原始值) | **drop-oldest** | overwriting ring + `dropped` 計數器——掉多少要看得見 |
| 只要統計不要原始值 | **就地聚合** | per-window count/sum/min/max,記憶體 O(#windows) 與樣本數脫鉤 |

數字:unbounded 的下場 = O(rate × 落後時間)——1M samples/s × 16B × 落後 60s ≈ **1 GB**
(見 [cost-model](cost-model.md) 第四節)。
硬體 / 行情推不回去 → 只剩 drop / aggregate 兩條路;TCP 對端推得回去 → backpressure 合法。

結構陷阱:**SPSC ring 上做不到 drop-oldest**(`head` 是 consumer 單寫的,
producer 動不了)——SPSC-safe 的是 drop-newest(push `Err` + 計數);
要「新蓋舊」得換 per-key conflation slot(見 [signal_pipeline](signal_pipeline.md))。

### Q2:速率多少?(決定 queue size 與結構檔次)

問法:*"What's the signal rate per node — hundreds per second, or millions?"*

**capacity = rate × 容忍落後時間**。例:100k/s × 容忍 100ms = 10k slots × 16B = 160 KB——
講得出這條算式,capacity 就不是拍腦袋。

| 速率 | 結構 |
|---|---|
| < 10k/s | `Mutex<VecDeque>` + Condvar,**不要炫技** |
| 100k–1M/s | bounded ring;開始考慮批次(chunk 化 = prefetch 友善,見 cost-model) |
| > 1M/s per producer | SPSC ring per producer + 批次消費 |

### Q3:幾個 producer / node / rack?(決定 thread 與 shard)

問法:*"How many producers hit this path — one device, or the whole rack?"*

| 答案 | 設計 |
|---|---|
| 1 個 | 恰好一產一消 → SPSC ring 的入場券 |
| 數十~數百 | 固定 pool + 共享 Mutex queue;先量再優化 |
| 數千+ | per-producer SPSC shard(每個 index 單寫者)/ sharded map;連線數千級 → thread-per-connection 到頭(2 MiB stack/條),換 event loop |

### Q4:SLA 是 p50 還是 p99.9?(決定 lock-free 值不值得)

問法:*"Are we optimizing average throughput, or tail latency?"*

| 答案 | 鎖策略 |
|---|---|
| p50 / 吞吐 | mutex 版就贏了:uncontended ~20ns,syscall(數百 ns)才是大頭。先寫它。 |
| p99.9 | lock-free 的理由**才**成立:mutex holder 被 scheduler preempt → 全體 waiter 陪卡一整個 timeslice(ms 級) |

台詞:**「lockless 買的是 p99.9,不是吞吐。」** 開場永遠先寫
`Arc<(Mutex<VecDeque>, Condvar)>`(十分鐘、能跑、正確),然後自己點出
tail 失敗模式作為升級理由——這是 pillar 3(Start Simple)+ pillar 4(Trade-offs)
的連續技。

### Q5:節點死掉,loop 怎麼知道?(決定偵測機制)

問法:*"How do we learn a node died — does TCP tell us, or do we own heartbeats?"*

| 答案 | 機制 |
|---|---|
| TCP 對端 | `read() == 0` = 正常關;但**半開連線**(對端斷電)TCP 不會告訴你 → heartbeat + idle timeout |
| 硬體 / UDP | 沒有連線概念:heartbeat deadline 進 min-heap timer queue;`epoll_wait` 的 timeout = 下一個 deadline |
| 自己排程(prober) | timer queue 決定「下一個該跑誰」;bounded queue + blocking submit 控制併發上限 |

這一問直接接彩排題 h(timer_queue)與題 d(heartbeat 保活)。

## 問完之後:鎖 contract

三十秒宣言,然後不再改:

> 「我假設:單機、每 node 10k/s、telemetry 可以聚合、SLA 是儀表板級(秒)。
> 所以:per-window 聚合、固定 windows、單 consumer。滿了不會發生(記憶體固定)。
> shutdown 時 drain 完再退。我開始寫。」

面試官糾正你 → 當場改一次 → 再鎖。**寫到一半改設計是最大的時間殺手**
(rehearsals README 的 45 分鐘 protocol,0–5 分鐘那格)。
