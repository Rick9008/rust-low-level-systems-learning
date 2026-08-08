# Mock 8/10(一)20:00 — 考官包:Thread Pool + JobHandle

Withers 當面試官,考 William(他也面 Etched Supercomputing SWE)。用法一句話:**Phase 1 題面開場直接貼;做到驗收線才放 Phase 2;Phase 3 用嘴考不寫 code。** 全程你是唯一 oracle——他問 clarify 你才答,他不問你不講。

## 時間預算(45m + 15m debrief)

| 時段 | 做什麼 |
|---|---|
| 0–5m | 題面閱讀 + clarify(好的候選人這裡至少問 3 個洞) |
| 5–30m | Phase 1 實作 |
| 30–40m | Phase 2(JobHandle) |
| 40–45m | Phase 3 口頭設計(scheduler) |
| +15m | debrief:先讓他自評 → 三個好的 → 兩個洞 → 一個「如果是 Etched 真場」建議 |

## Phase 1 題面(開場貼這段)

```text
You're building a fixed-size worker thread pool for a service that must
not spawn a new thread per request.

Requirements:
- `ThreadPool::new(n: usize) -> ThreadPool` spawns `n` worker threads.
- `pool.execute(job)` accepts a closure and runs it on one of the workers.
- Jobs may be submitted from multiple threads.
- When the pool is dropped, it must shut down cleanly.

Use only the standard library. Tell me your plan before you type.
```

**埋好的洞(他該 clarify 的;問到才答,沒問到記下來 debrief 講)**:

| 洞 | 被問到時你的答案 |
|---|---|
| shutdown 時 queue 裡還沒跑的 job 怎麼辦? | drain:跑完才退(submit-after-shutdown 三分法你熟,他若答 drop-remaining 也接受,但要他講理由) |
| execute after shutdown? | 這版 panic 或忽略都可,要他自己選並說出來 |
| queue 有界嗎?滿了 block 還是丟? | 無界即可;他主動談有界=加分,追一句「有界要幾個 condvar 方向?」 |
| job panic 了 worker 要死嗎? | Phase 1 可以死;預告 Phase 2 會回來 |
| `new(0)`? | documented panic 或 clamp 到 1,他選,講理由就過 |

**驗收線(全中才放 Phase 2)**:
1. worker loop:拿鎖 → predicate loop 裡 wait 直到「有 job 或 shutdown」→ **放掉鎖再跑 job**(鎖圈住 job 執行=你 7/17 親手抓過的洞,最常見)。
2. Drop:flag → `notify_all` → join 全部 worker。
3. condvar wait 有 predicate loop——直接問他:「spurious wakeup 會發生什麼事?」

## Phase 2 題面(口頭給即可)

```text
Now fire-and-forget isn't enough: callers need the result back.
Add `submit<T>(job) -> JobHandle<T>` where `handle.join()` blocks until
the job finishes and returns the result. Decide what `join()` returns
if the job panicked — it must not hang forever.
```

考點:oneshot(mpsc 或自製 Mutex+Condvar)、`Box<dyn FnOnce + Send>` type erasure(**泛型放方法、不放 struct**——pool 對 job 型別是瞎的)、panic 路徑:job panic → sender 被 drop → `recv()` 回 `Err` = 天然的 panic 信號(或 `catch_unwind(AssertUnwindSafe)` 包)。題面那句 "must not hang forever" 是提示燈,看他接不接。

## Phase 3 口頭設計(不寫 code,每題 1–2 分鐘)

1. 加 priority 會改哪些型別?(BinaryHeap 進 Mutex;FIFO 平手怎麼破 → seq 次鍵)
2. job 之間有依賴(DAG)呢?(indegree 入場閘:完成時遞減 dependents、歸零才入 heap)
3. 高優先 job 等一個低優先 job 完成——這現象叫什麼、怎麼辦?(priority inversion;priority inheritance / 提升)

## Bug watchlist(你自己踩過的,他大概率也踩)

1. Drop 忘 join / 忘 `notify_all`(worker 永遠睡著,join 卡死)。
2. 鎖圈住 job 執行(並行度=1,程式「對」但整個 pool 白做)。
3. submit push 完沒 notify(lost wakeup)。
4. 退出條件 ∧/∨ 寫反(你的 De Morgan 傷疤;處方=loop + 正面條件 break)。
5. channel 版的隱形同款:`while let Ok(job) = rx.lock().unwrap().recv()`——guard 活過整個 job 執行。

## 考官守則(R1/R2 換來的)

- 你是唯一 oracle:他問了才答,只答被問的那件事。
- 卡住超過 3 分鐘給一個**方向性**提示(「你的 worker 怎麼知道該醒?」),記一筆。
- 好訊號隨手記,debrief 用:clarify 數量與品質、先講 plan 才動手、自己寫測試、自己 dry-run。
- 對你自己:這場是 8/11 Jan 場前的主動回憶——他每踩一個 watchlist 的洞,等於幫你把傷疤複習一遍。mock 完仍跑你自己的輕 taper(§E + 8/6 洞清單),00:00 前熄燈。
