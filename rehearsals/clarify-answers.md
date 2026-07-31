# clarify 情境卡答案 —— 寫完才開

每卡:該問的五問(具體化)→ 關鍵分支 → canonical 設計 → 一句 killer trade-off
→ 常見錯誤。五問的順序固定:**掉不掉 / 速率 / 規模 / SLA / 偵測**
(縮寫圖例與英文問法:`docs/clarify-playbook.md` 開頭的速記表)——
漏掉哪一問,回去 playbook 補那一節。

## 卡 → 對應 code(對完答案後想看實作,從這進)

| 卡 | canonical 的可執行版 |
|---|---|
| 1 telemetry hub | `reference/src/concurrency/signal_pipeline.rs`(扇入 `start_fan_in`)+ 彩排 f telemetry_aggregator;fan-in 的 spec-heavy 教學版 `concurrency/percpu_fanin`(sim k,⚠ 8/1 場後才開) |
| 2 RPC gateway | `hw_bridge::server_evented`(bounded + eventfd 回程;EPOLLIN 關閉的 backpressure 用講的)+ 彩排 h(deadline/timeout) |
| 3 market data | conflation slot——repo 無專模組;思路見 `docs/concurrency/signal_pipeline.md` 的 drop-newest vs conflation 節 |
| 4 log shipper | 彩排 a ring_drop_oldest(bounded + drop-oldest + dropped 計數) |
| 5 sensor bridge | `reference/src/concurrency/signal_pipeline.rs` 單源版(教科書本尊);ISR 紀律的 spec-heavy 教學版 `concurrency/isr_pipeline`(sim j,⚠ 7/31 場後才開) |
| 6 health prober | 彩排 h timer_queue + `thread_pool`(bounded submit 天然限流);timeout/重試政策的可執行版 `io/engine_watchdog`(sim m,⚠ 8/2 場後才開) |

---

## 卡 1:telemetry hub

**五問**:掉不掉/可聚合嗎?每 node 速率?幾台 node?儀表板要多新(秒級可)?
node 斷線怎麼判死?

**關鍵分支**:儀表板只要統計 → **就地聚合**(不存原始值);要原始值 → 退到
drop-oldest ring + dropped 計數。

**canonical**:per-node 或 per-metric 的 window 聚合(count/sum/min/max),
記憶體 O(#windows × #nodes) 與樣本數脫鉤;數千 node → 連線用 event loop /
tokio,聚合寫入 sharded 結構;heartbeat timeout 判死(timer queue)。

**台詞**:「unbounded queue 在這裡是 O(rate × 落後),1M/s × 16B × 60s ≈ 1GB
——所以我直接聚合,記憶體變 O(#windows)。」

**常見錯誤**:上來就設計 lock-free MPMC queue 存原始值——Q1 沒問,
方向就錯了;聚合根本不需要 queue 深度。

## 卡 2:RPC gateway

**五問**:請求可以丟嗎(必答 → 不可)?請求速率與大小?幾個 client / 幾個
backend?SLA p50 還是 p99(gateway 通常看 p99)?backend 慢/死怎麼偵測?

**關鍵分支**:必答 → **backpressure 是唯一合法答案**;要問「壓回去的邊界」
——排隊上限 + 超時回錯(fail fast),不是無限排。

**canonical**:bounded queue per backend + 滿了怎麼辦:對 client 停止讀
(`EPOLL_CTL_MOD` 關 EPOLLIN / tokio 停 poll)讓 TCP 把壓力傳回去;
請求掛 deadline(timer queue),backend 超時 → 回 504,不佔位。

**台詞**:「RPC 不能 drop,但可以**拒絕**——bounded + timeout 把失敗變
可預期的,unbounded 把失敗變 OOM。」

**常見錯誤**:queue 無上限(慢 backend = 記憶體照速率成長);
忘了 timeout——排在隊裡的請求 client 早就放棄了,你還在替死人排隊。

## 卡 3:market data feed

**五問**:舊 tick 有價值嗎(只要最新 → 沒有)?每 symbol tick 率?幾個
symbol?策略端要 p99.9 嗎(交易通常要)?feed 斷線怎麼知道?

**關鍵分支**:只要最新 → **conflation**:per-symbol 一格「最新值」覆蓋寫,
不排隊。這比 drop-oldest ring 更極端——capacity = 1。

**canonical**:`Vec<Slot>` per symbol(symbol id 密集 → 直接 index,
fd_registry 同思路),寫端覆蓋 + 版本號;讀端讀最新;p99.9 → 每 slot
單寫者,seqlock 或 atomic 對(進階);斷線 = feed 層 heartbeat。

**台詞**:「策略端慢不是問題——它醒來永遠拿到**最新**價,中間的 tick
本來就不該補。conflation 把 backpressure 問題直接消滅。」

**常見錯誤**:用 queue 排 tick——讀端慢時你餵給它的是**舊價**,
在交易語意裡比掉資料更糟。

## 卡 4:log shipper

**五問**:log 可以掉嗎(app 不能卡 → 極端下必須能掉)?寫入速率?
單 agent 收幾個程序?SLA(送達延遲幾秒可)?收集器斷線多久算常態?

**關鍵分支**:「app 不能被卡」+「網路會斷幾分鐘」兩個約束相乘 →
**bounded buffer + drop-oldest(帶計數)**;「不能掉」只在 buffer 沒滿時成立。

**canonical**:app → agent 用 bounded queue,滿了 drop-oldest + `dropped`
計數(寧掉 log 不卡 app);agent → 收集器批次送 + 重試 backoff;
斷線期靠 buffer 深度撐:capacity = 寫入率 × 最大容忍斷線時長(算給面試官看)。

**台詞**:「這題的 capacity 不是拍腦袋:10k lines/s × 200B × 斷 60s ≈ 120MB
——要嘛給我這麼多記憶體,要嘛接受掉 log,兩者選一個,我把 dropped 做成
可觀測的。」

**常見錯誤**:選 blocking submit(app 卡死,違反題目約束);
或 unbounded(斷線 5 分鐘 = OOM)。

## 卡 5:sensor bridge

**五問**:訊號可聚合/可掉嗎?爆發速率與平均速率?單一裝置(→ 單 producer)?
消費端要多即時?裝置死了怎麼知道(訊號斷流算嗎)?

**關鍵分支**:單一裝置 + 單一消費者 → **SPSC ring** 的教科書入場;
裝置推不回去 → 滿了只能 drop(overwriting)或聚合。

**canonical**:SPSC overwriting ring(power-of-2、cache-line padding),
爆發吸收靠 capacity = 爆發率 × 爆發時長;消費端批次 drain;
斷流偵測:heartbeat deadline(上次訊號時間 + 容忍值)進 timer queue。

**台詞**:「硬體沒有 flow control——backpressure 物理上不存在,所以 full
policy 只剩 drop-oldest 或聚合。我用 overwriting ring,dropped 計數是
儀表板上的一級公民。」

**常見錯誤**:對裝置談 backpressure(推不回去);
用 Mutex queue 扛 MHz 爆發(p99.9 會被 preemption 打爆——這卡就是
lockless 故事成立的地方)。

## 卡 6:health prober

**五問**:probe 結果可以丟嗎(自己排的工作 → 不丟,但可跳過過期的)?
幾百台 × 頻率 = 每秒幾個 probe?併發上限(別打掛目標)?判死延遲 SLA?
「死」的定義(連不上?慢?連續幾次?)

**關鍵分支**:工作是**自己排程的** → rate 你完全可控,這題不需要任何
lock-free——bounded queue + blocking submit 就是正解。

**canonical**:min-heap timer queue 排「下一個該 probe 誰」;固定 worker
pool(併發上限 = pool 大小,天然限流);probe 掛 timeout;連續 N 次失敗
才標紅(抖動抑制);錯過排程的 probe 跳過補跑(別追積欠)。

**台詞**:「這題的量級是每秒幾百個 probe,uncontended mutex ~20ns,
瓶頸在網路 RTT(ms 級)——結構上任何優化都不值得,我把力氣花在
timeout 與抖動抑制。」

**常見錯誤**:對這題掏 SPSC ring / lock-free(cost model 沒算,
炫技扣分);沒有併發上限(重試風暴把目標打掛——你變成攻擊者)。
