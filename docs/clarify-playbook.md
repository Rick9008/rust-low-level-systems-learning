# clarify playbook —— 開場五問,答案決定設計

定位:pillar 1(Clarify)的操作手冊。JD 已經把線索寫給你(node 限制、
network flakiness、跨 rack 規模)——面試官等你把線索變成問題,把答案變成
設計決策。**紀律:0–5 分鐘問完、鎖 contract、之後不中途改設計。**

互動版:[`artifacts/clarify_playbook.html`](artifacts/clarify_playbook.html)
(選答案,看設計跟著變)。
卡片練習:`rehearsals/clarify-cards.md`(6 張情境卡;答案在
`rehearsals/clarify-answers.md`,**寫完才開**)。

## 五問速記表(縮寫圖例 + 面試英文問法)

全 repo 的「掉不掉/速率/規模/SLA/偵測」五類縮寫,對照如下——
面試時**用英文問**,右欄背到能脫口:

| 縮寫 | 完整問題 | 英文問法 |
|---|---|---|
| **掉不掉** | 資料掉不掉得起?(決定 full policy) | *"Under pressure, can we drop or aggregate, or is every sample required?"* |
| **速率** | 每個源頭每秒幾筆?(決定容量算式) | *"What's the signal rate per node — hundreds per second, or millions?"* |
| **規模** | 幾個 producer / node / rack?(決定 thread/shard) | *"How many producers hit this path — one device, or the whole rack?"* |
| **SLA** | 追平均還是尾延遲?(決定 lock-free 值不值) | *"Are we optimizing average throughput, or tail latency?"* |
| **偵測** | 對端死了怎麼知道?(決定 heartbeat/timer) | *"How do we learn a node died — does TCP tell us, or do we own heartbeats?"* |

Note: 
1. rack:一整櫃機器(幾十台 node,每台又可能有多個訊號源)。
2. SLA is Service Level Agreement
   Decide how the data should be handled, like if it read by human, latency can target on average, if it need to be machine(automata) it need to target to tail

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
要「新蓋舊」得換 per-key conflation slot(見 [signal_pipeline](concurrency/signal_pipeline.md))。

#### 「就地聚合」到底在做什麼

上表第三列是個**答案**,不是個方法。答不出「怎麼做」就選不下去,所以把它攤開——
以下都是 clarify 階段講得出口的層次,實作的邊界在彩排 f。

**關鍵是換索引。** ring buffer 的索引是「第幾個到的」,所以亂序是災難。
window 的索引是**樣本自己的時間屬於哪一格**:

```
window_index = ts / W        ← 整數除法,W = window 長度
```

這是個**只看樣本自己的純函數**。同一筆 ts,不管現在到、晚三秒到、還是跟五百筆
擠在一起到,算出來永遠是同一格。業界術語:你用 **event time** 分格,不用
**processing time**。「亂序怎麼辦」的答案是**不辦**——你根本沒在看到達順序。

**格子裡不存樣本,只折進四個數字**:

```
count += 1;  sum += v;  min = min(min, v);  max = max(max, v);
```

每筆 O(1)。記憶體跟**進來幾筆無關**,只跟你留幾格有關——這就是「與樣本數脫鉤」。
亂序無害的真正理由是這四個運算**可交換**:先折 A 再折 B 等於先折 B 再折 A。

順帶兩個一定被追問的點:

- **average 要存 `sum` + `count`,不能存 running average**——兩個平均沒辦法合併。
- **min/max 只在 tumbling(固定)window 是 O(1)**。sum 有反元素(樣本離開時
  `sum -= v` 就還原),min/max 沒有——你不能「減掉」一個 min。固定 window 從不移除
  樣本(時間到就關格、開新的),所以不需要反元素;**sliding window 的 min/max**
  只能重掃或上 monotonic deque。這不是實作技巧問題,是代數結構問題。

**什麼時候聚合才划算:每格樣本數 ≫ 1。** 這條沒人講但很致命——

| 每 node 速率 | 存原始值(覆蓋 60s) | 聚合(1 秒格 × 60) |
|---|---|---|
| 1 Hz | **4.3 MB** ← 原始值贏 | 8.6 MB |
| 1 kHz | 4.3 GB | **8.6 MB** |
| 1 MHz | 4.3 TB | **8.6 MB** |

1 Hz 下每格只有 1 筆,你花 4 個數字去記 1 筆——聚合是把資料**變大**的有損壓縮。
crossover 在**每格約 2 筆**。所以正確的說法不是「聚合比較省」,是
**「聚合的成本與速率無關」**;哪條划算由 Q2 的數字決定,這也是為什麼
**Q1 要等 Q2 的答案才能定案**。

兩條式子背起來:

```
存原始值:memory = rate × T × size      ← rate 在式子裡
聚合:    memory = (T / W) × 固定        ← rate 不在式子裡
```

代價講清楚才叫 trade-off:**聚合之後你永遠答不了「14:03:22 那筆讀數是多少」**。
所以切點是產品需求不是效能——儀表板只要統計 → 聚合;要看得到原始波形 → 退回
drop-oldest ring,並把 `rate × 保留秒數 × size` 算給面試官看,讓他決定付不付這個記憶體。

實作的三個邊界(只保留 N 格時:格子住哪個 slot、落在已淘汰的過去怎麼判、ts 跳到
未來時中間那些格裡的舊資料誰清)——那是**彩排 f `telemetry_aggregator`**,
排在 `SCHEDULE.md` 的 7/24,別在這裡先看答案。

### Q2:速率多少?(決定 queue size 與結構檔次)

問法:*"What's the signal rate per node — hundreds per second, or millions?"*

**capacity = rate × 容忍落後時間**。例:100k/s × 容忍 100ms = 10k slots × 16B = 160 KB——
講得出這條算式,capacity 就不是拍腦袋。

| 速率 | 結構 |
|---|---|
| < 10k/s | `Mutex<VecDeque>` + Condvar,**不要炫技** |
| 100k–1M/s | bounded ring;開始考慮批次(chunk 化 = prefetch 友善,見 cost-model) |
| > 1M/s per producer | SPSC ring per producer + 批次消費 |

**算完立刻反推:這個數字有沒有把題目消滅掉?** 卡 1 實測會踩到——
1 Hz × 3000 nodes × 8 B = **24 KB/s**,但題幹說「遠超過你能存的記憶體」。
24 KB/s 你當然存得下,所以要嘛速率不是 1 Hz,要嘛他要的是**長期保留**;
讓 unbounded 成立的乘數是**時間軸**(`O(rate × 保留時長)`),不是瞬時速率。

發現這件事的正確反應是**當場講給面試官聽**,不是偷偷換一個假設繞過去:

> *"At 1 Hz across 3000 nodes that's only 24 KB/s — that fits in memory easily.
> So either the real rate is much higher, or you want long retention.
> Which is it?"*

這一句直接證明你在算,不是在猜——而且它把題目的真正約束逼出來了。

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
