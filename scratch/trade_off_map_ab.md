# a / b 題 trade-off 地圖(7/21 凌晨對話沉澱;7/25 口述錄音底稿;**已上板 2026-07-24:「Low level learning」`b7fa1ee5`**)

公式(每軸三句,JD pillar 4):**I chose X → it costs Y → I'd switch at Z**。

## a · ring_drop_oldest(四軸)

1. **Full policy**:drop-oldest vs backpressure vs drop-newest——telemetry 選 drop-oldest(新鮮度 > 完整性;記憶體 O(cap) 固定、全 op O(1))
2. **head+len vs head+tail**:滿/空二義(a#1 判空 bug 的根);len 表示法不浪費一格、無二義
3. **dropped 計數**:近零成本觀測性;沒有它 = silent data loss
4. **規模轉折**:跨 thread 時 drop-oldest 撞牆(consumer/producer 都動 head = 兩個 writer)→ 談判降級

> "I chose drop-oldest because for telemetry the newest reading matters most. The cost is silent
> data loss, so I expose a dropped counter. If every sample must be seen, I'd switch to
> backpressure — block or reject on push."

## b · pool_graceful_shutdown(五軸)

1. **graceful drain vs immediate abort**:drain 保證已接受必跑,代價 = shutdown 延遲無上界 → production 第三選項:兩段式(drain + timeout 後 abort)
2. **unbounded vs bounded queue**:submit 永不塞 vs backpressure(b#1 clarify 漏問的那條)
3. **Mutex+Condvar vs lock-free**:無競爭 Mutex ~20ns、臨界區兩下指標;lock-free 買尾延遲、付複雜度稅;tokio injection queue 也是 Mutex(qa_eventfd_doorbell §5)
4. **AtomicBool flag vs 狀態塞進 Mutex**:flag 讓 submit fast-path 免鎖,代價 = notify 同步自己顧(b#1 的 lost-wakeup 傷疤——講自己的 bug + 修法)
5. **Big-O 收尾**:submit O(1);shutdown = O(剩餘 jobs) drain + O(workers) join

> "Graceful here means: stop accepting, but every accepted job still runs. The cost is unbounded
> shutdown latency, so in production I'd add a timeout escalation — drain first, abort what's left."

## 被逼 lock-free 時的應對(兩題通用)

**核心:那是 pillar-4 測驗,不是實作要求。階梯 + 挑會的寫。**

- **a 被逼 MPMC**:①點名衝突(drop-oldest 並發 = 兩 writer 動 head;MPMC 再搶 tail)②階梯:
  `Mutex<ring>`(45 分正解)→ per-producer SPSC + 單 consumer 掃描聚合(= signal_pipeline /
  perf per-CPU ring,MPMC 直接拆掉)→ Vyukov 每槽序號 / seqlock 覆寫 + 點名 ABA ③寫會的那格:SPSC(loom 驗過)
  > "Lock-free MPMC with drop-oldest is a research-grade structure — production systems sidestep
  > it with per-producer SPSC rings and one draining consumer. Let me show that decomposition."
- **b 被逼 lock-free pool**:①拆熱/冷路徑——lock-free 只對 submit/pop 有意義;shutdown/join 是
  rendezvous,冷路徑鎖便宜又對 ②idle worker 要睡,睡沒有 lock-free 這回事(parker/futex);
  lost-wakeup 不消失只搬家 ③tokio 實況:per-worker local queue + stealing,injection queue = Mutex
  > "Even tokio keeps a mutex on its cold path. I'd make submit and pop cheap, keep shutdown
  > boringly simple, and reach for per-worker queues only after measuring contention."

## 單 consumer 掃描聚合(fan-in,7/23 signal_pipeline 預習)

N producer 各寫各的 SPSC ring → 一條 consumer `for ring in rings { drain }` 彙整。
「多」從寫端(爭 tail)搬到讀端(多走路)——走路無爭用、零 CAS。
代價三條:O(P) 掃描(稀疏時要髒名單/門鈴)、無全域順序(per-ring FIFO)、單 consumer 是吞吐上限(不夠再 shard)。
留給 7/23 對答案:consumer 沒貨要不要睡?誰叫醒?「剛放貨你剛睡著」怎麼不丟?(門鈴頁 + b#1 bug 的形狀)

## 英文地雷(b#1 現場)

- **park / unpark**(不是 pack)
- "check **all jobs are done**"(不是 all down)
