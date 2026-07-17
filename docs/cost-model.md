# cost model —— 面試前最後一眼的數字底稿

面試公式:**先給 Big-O,再給常數,再指出規模轉折點**。
這頁全是數量級(order of magnitude),不是精確值——面試講數量級就夠,
講得比這精確反而可疑。

## 一、基本操作的數量級

| 操作 | 成本 | 備註 |
|---|---|---|
| L1 cache hit | ~1ns | |
| LLC hit / 跨核 cache line 轉移 | ~40–100ns | false sharing 的單價;`#[repr(align(64))]` 就是在省這個 |
| RAM 存取 | ~100ns | pointer-chasing 每跳一次付一次 |
| atomic load/store(x86,無競爭) | ~1ns | 編出來就是 `mov`;acquire/release 在 x86 免費 |
| CAS(有競爭) | ~20–100ns | 含 cache line 搶奪;失敗還要重試 |
| Mutex lock/unlock(無競爭) | ~20ns | 純 userspace CAS,沒有 syscall |
| **Mutex(有競爭)** | **~1–10µs** | futex syscall + context switch——貴 100× 的來源 |
| syscall 來回 | ~數百 ns | Spectre/Meltdown 緩解後更貴 |
| context switch | ~1–5µs | 外加 cache 汙染的隱性成本 |
| thread spawn | ~數十 µs | stack 預設 2 MiB(Rust)——一萬條 = 20 GB 虛擬位址 |

## 二、queue 三型(對映 repo 模組)

| 型 | push/pop | 競爭行為 | 什麼時候用 |
|---|---|---|---|
| `Mutex<VecDeque>` + Condvar(`bounded_queue`) | O(1),無競爭 ~20ns | **全序列化**:核愈多搶愈兇,吞吐反而降(convoy + 鎖那條 cache line 互踢) | 需要 block/close 語意、吞吐中低——90% 的家常菜場景 |
| SPSC ring(`spsc_ring`) | O(1),~10–50ns,**永無 syscall** | 每個 index 單寫者,零 CAS 重試;pad 掉 false sharing | 恰好一產一消;幾千訊號就 shard 成 per-producer ring |
| MPMC lock-free(production: crossbeam) | O(1) 攤銷,但 CAS 重試迴圈 | 高競爭下失敗率上升 | 真的多對多才用;面試先講「我會先 shard,逼不得已才 MPMC」 |

**標準答案的形狀**:「Mutex 版和 lockless 版都是 O(1)——差別在常數與競爭行為:
contended lock 是 µs 級且隨核心數劣化,SPSC 是 ns 級且 per-signal 隔離。」

**lockless 買什麼**:不是吞吐——uncontended push ~20ns,對比一次 syscall
數百 ns、context switch µs 級,queue 只是總成本的零頭。它買的是 **tail**:
mutex holder 被 scheduler preempt 的那一刻,所有 waiter 陪卡一整個 timeslice
(ms 級)→ p99.9 爆掉。把這講成 trade-off(而非 gotcha)是滿分答案;
JD 寫「lockless 更快」時,你補上「更快的是 p99.9,不是平均」。

## 三、event registry:poll vs epoll

| | 每次呼叫 | 登記 | N=10,000、ready=10 時 |
|---|---|---|---|
| `select` / `poll` | **O(N_watched)**(整份 fd 表拷進 kernel 掃一遍) | 免登記(每次全帶) | 每次 wakeup 掃 10,000 |
| `epoll` | **O(N_ready)** | `epoll_ctl` O(log N)(RB-tree 常駐 interest) | 每次 wakeup 只碰 10——**差 1000×** |

這是「特定 event registry 更快」的真 Big-O 故事;lockless queue 那半反而是常數的故事(見二)。

## 四、space:telemetry 的三種形狀

| 策略 | 記憶體 | 語意 |
|---|---|---|
| unbounded 原始 queue | **O(rate × 落後時間)**——無上界 | consumer 卡住 = OOM/fd 爆;1M samples/s × 16B × 落後 60s ≈ **1 GB** |
| bounded ring + drop-oldest(彩排題 a) | **O(capacity)** 硬上界 | 新資料比舊值錢;`dropped` 計數 = 可觀測的洩壓閥 |
| per-window 聚合(min/max/sum/count) | **O(#windows)**,與樣本數脫鉤 | 1440 個 minute-bucket × 32B/天,對比上面的 GB |

加碼:chunk 化(連續記憶體批次處理)同時買到 time——順序掃 = prefetch 友善,
對比逐筆 pointer-chasing 每筆付 ~100ns cache miss。

## 五、並發模型的轉折點

| 模型 | 甜蜜區 | 轉折點 |
|---|---|---|
| thread-per-connection | 短請求、~百級連線 | 千級起 stack(2 MiB/條)+ context switch 吃掉你 → event loop |
| acceptor + 固定 pool | 短請求、需要並發上限 | 長連線多 → 第 N+1 條餓死在 queue → event loop / tokio |
| epoll event loop / tokio | 萬級連線、IO-bound | CPU-bound 任務會凍住 loop → offload(`file_io_offload`)|
| readiness(epoll) | socket | regular file 永遠 ready → completion(io_uring) |

## 六、被追問「再快呢?」的三句(彈藥,不主動開火)

主動講會顯得往 HFT 對齊而不是往題目對齊(與「開口就 lock-free」同款失分);
只在面試官自己把預算壓到 µs 以下時開火,一題一句:

| 追問 | 台詞 | 一句原理 |
|---|---|---|
| dashboard 讀路徑去鎖化? | *"Single writer, many readers, occasional re-read is fine — that's a seqlock: readers validate a version counter and retry, the writer never blocks."* | 比 RCU 便宜的第一站;寫者無等待,讀者撞到寫入中就重讀 |
| syscall 太貴怎麼辦? | *"Below a microsecond you evict the kernel: the NIC DMAs into a userspace ring and you busy-poll it — same SPSC shape, minus the kernel."* | kernel bypass 跟 SPSC ring 是同一張圖,只是把 kernel 請下車 |
| 為什麼交易系統要 cache warming、telemetry 不用? | *"Trading hot paths rarely fire, so they go cold — they dry-run to stay warm. A telemetry stream is continuous; the path keeps itself warm."* | 冷 i-cache/d-cache 吃 tail;連續流天然保溫——反手接回 data plane 用 sync + pinned thread 的理由(見 signal_pipeline) |
