# signal_pipeline 設計取捨

對應程式碼:`reference/src/concurrency/signal_pipeline.rs`。drill:`drills/src/concurrency/signal_pipeline.rs`
(兩洞:send、掛牌握手);challenge:`challenges/src/concurrency/signal_pipeline.rs`(★)。
相關:[spsc_ring](spsc_ring.md)(佇列本體)、[cost-model](../cost-model.md)
(容量算式與 queue 三型)、`hw_bridge` 的 `server_evented_spsc`
(同一套握手接進 event loop)。

## 這張圖是誰

**訊號源執行緒 → SPSC ring → 消費執行緒**。telemetry JD 的核心圖;
也是 HFT 的標準管線形狀:pinned thread 之間全用 SPSC,熱路徑
零鎖、零 syscall、零配置(LMAX Disruptor 一脈)。三個決策正交:
佇列(SPSC)、full policy(掉誰)、等待策略(誰睡誰醒)。

## 消費端等待策略的階梯

| 策略 | 喚醒延遲 | 代價 | 誰在用 |
|---|---|---|---|
| busy-spin | ~ns | 燒滿一顆核 | HFT(核 pinned,寧燒不睡) |
| **spin-then-park(本實作)** | 熱路徑 ~ns、冷路徑 ~µs | spin 額度內燒 CPU | 通用低延遲 |
| 純 park / condvar | ~µs(futex + context switch) | 每次喚醒 syscall | 吞吐型 |
| eventfd | ~µs | 同上 | 睡覺方是 epoll loop 時(`server_evented_spsc`) |
| waker | 進 run queue 的延遲 | 無 syscall(通常) | 睡覺方是 async task 時 |

同一條 ring,插槽裡換誰都行——SPSC ring 刻意不內建等待策略的原因。

## 掛牌握手:repo 第一個真需要 SeqCst 的地方

naive 的 park 版有經典 bug:producer 每筆 unpark = 每筆一次 syscall,
SPSC 的零 syscall 白省。所以 consumer **掛牌**(`parked=true`)才睡、
producer **看到牌子**才 unpark——快路徑零 syscall。但這個握手是教科書
store-buffering(SB)litmus:

```text
consumer:  parked = true   ;  再查一次佇列(讀)
producer:  push(寫)       ;  查 parked(讀)
```

兩邊都是「先寫後讀」。Release/Acquire 之下,**兩邊都允許讀到舊值**
(store buffer 還沒 flush):producer 沒看到牌子(不 unpark)、
consumer 沒看到貨(去睡)→ 帶著貨睡死。解法:兩邊寫讀之間各插一道
`fence(SeqCst)`,這個交錯被全域順序禁掉。park 的 token 語意兜底另一半
(unpark 先於 park 不丟)。

面試裡把這個 trace 講出來,就是 pillar 5 對 lock-free 的「trace 兩條
thread 交錯」滿分示範——而且它解釋了 SeqCst 存在的意義(不是「保險」,
是 SB litmus 這一類交錯真的需要它)。口述版:
*"I relax the SPSC itself to acquire/release, but the parking handshake
is a store-buffering pattern — that one needs SeqCst."*

三個 deep-dive 加分點:

1. **acquire/release 擋不住**:acq/rel 給的是單一變數上的 happens-before;
   這裡是兩個獨立變數(牌子與 tail)交叉先寫後讀——SB 正是 acq/rel 允許、
   只有 SeqCst 禁止的形狀。
2. **這個 bug 在 x86 上是真的**:StoreLoad 是 x86-TSO 唯一允許的重排。
   對照 `loom_vs_stress` 那個 Relaxed bug(x86 硬體不重排 store-store /
   load-load,無物理表現、要 ARM 才炸)——掛牌握手這個在你的筆電上
   就可能炸,只是時間窗小到 stress 測不出。
3. **park token 救不了這裡**:token 救的是「unpark 先於 park」;
   這個 race 是 producer **根本沒呼叫 unpark**(讀到 stale 的 false)。
   沒發出的 unpark,token 存不了——這是它與 executor 那課的精確分界。

## async 買什麼、sync 買什麼(control plane vs data plane)

async 買的是**便宜的閒置等待**:萬條大多安靜的 socket 疊在幾條 thread 上。
硬體訊號流不是閒置,是**連續**——這時 async 反而倒貼:executor 排程的
latency jitter + 每筆訊號疊 poll/wake 開銷,p99.9 就是這樣被吃掉的。
口述版:*"Async buys you cheap idle waits — ten thousand mostly-quiet
sockets on a few threads. But a hardware signal stream isn't idle, it's
continuous. There I want pinned threads and SPSC rings: no runtime
scheduling on the hot path, predictable tail latency. Async is for the
control plane; the data plane stays sync."*

## 扇入:多源就 per-source SPSC,不是 MPSC

每源一條自己的 ring、一條 consumer 掃全部(`start_fan_in`):

```text
source_0 thread ──SPSC ring_0──┐
source_1 thread ──SPSC ring_1──┼──▶ consumer(round-robin 掃)
source_2 thread ──SPSC ring_2──┘
```

三個可講的 trade-off:
1. **保住 SPSC**:每條 ring 仍單寫者 → 零 CAS。全部打同一條 MPSC →
   tail 上 CAS 競爭,SPSC 的優勢原地蒸發。
2. **per-source 隔離**:某源爆量只塞滿自己的 ring、觸發自己的 drop 計數
   ——「單一 source 爆量怎麼不拖垮全局」的答案,結構本身就是答案
   (測試 `fan_in_slow_source_isolated_from_burst` 是可執行版)。
3. **ordering 說清楚**:per-source FIFO 保住、跨源全域順序沒有——
   telemetry 通常無所謂,但要說出口(clarify 的料)。

三個 follow-up 必問區:
- **公平性**:別把一條 ring 抽乾才換下一條(熱源餓死其他源)——
  每輪每條最多 `FAN_IN_BATCH` 筆,round-robin。
- **park 條件 = 全部都空**:掛牌後的 recheck 要掃完所有 ring;
  每個 producer push 完都跑同一套 fence + 查牌 + unpark。
- **一條 consumer 吃不動**:source 靜態分片給多條 consumer——每條 ring
  仍 SPSC、per-source 順序仍保住;work-stealing 兩者都破壞,不選。

一句收斂:**拓撲的每一步都在守單寫者**——守住單寫者,就守住零 CAS 與
per-source FIFO;所有 scale 手段(分片)都繞著「不破壞單寫者」設計。

## full policy:SPSC 上只有 drop-newest

`head` 是 consumer 單寫的——producer 動不了它,所以 **drop-oldest
(覆蓋最舊)在 SPSC ring 上做不到**;SPSC-safe 的是 drop-newest
(push 回 `Err`、丟這一筆、計數)。要「新蓋舊」得換結構:
per-key conflation slot(market data 的答案,capacity=1 覆蓋寫)。
`dropped` 計數是 producer 單寫欄位——普通 `u64` 就夠,連 atomic 都不用。

## 守恆測試的思路

`accepted + dropped == sent` 且 `stats.count == accepted`:每一筆要嘛
進聚合、要嘛被數到,沒有黑洞。lost-wakeup 的測試設計成**卡死顯性失敗**
(join 不回來),而不是靜默漏資料——壞要壞得看得見。

## 誠實邊界

- 掛牌握手的交錯靠手 trace + stress 把關;loom 版要把 fence 與 park
  收進 shim,留作延伸。
- std 無法 portable 地 pin 核心(`sched_setaffinity` 是 raw syscall 領域);
  HFT 的 pinning + isolcpus 屬 deployment 層,聲明即可。
- 單 consumer:要多消費者,每 consumer 一條 ring(shard),不是 MPMC。

## Production 對照

crossbeam 的 `Parker`(掛牌握手的工業版,同款 SeqCst 處理)、
rtrb / ringbuf(SPSC ring crate)、LMAX Disruptor(Java,batch +
sequence barrier)、tokio 的 mpsc(async 世界的對應)。

## 互動教材

[artifacts/signal_pipeline.html](artifacts/signal_pipeline.html) ——
五種睡法一張表(含「沒有 epoll 是不是只能 sleep」的按事件醒/按時間醒分類);
掛牌握手 stepper:fence 與 re-check 兩個開關、四種組合逐步走交錯,
拔掉 fence 看 consumer 帶著貨睡死(store buffer 的內容畫給你看);
扇入隔離:按下爆源,看 dropped 只長在它自己的 ring 上。
