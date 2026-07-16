# signal_pipeline 設計取捨

對應程式碼:`reference/src/signal_pipeline.rs`。drill:`drills/src/signal_pipeline.rs`
(兩洞:send、掛牌握手);challenge:`challenges/src/signal_pipeline.rs`(★)。
相關:[spsc_ring](spsc_ring.md)(佇列本體)、[cost-model](cost-model.md)
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
是 SB litmus 這一類交錯真的需要它)。

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
