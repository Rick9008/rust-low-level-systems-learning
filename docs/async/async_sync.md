# async_sync 設計取捨

對應程式碼:`reference/src/runtime/async_sync.rs`。相關:
[thread_pool](../concurrency/thread_pool.md)(JobHandle:condvar 睡的 one-shot)、
[file_io_offload](../io/file_io_offload.md)(JoinFuture:waker 睡的 one-shot)、
[bounded_queue](../concurrency/bounded_queue.md)(predicate-wait 的 blocking 原版)、
[executor](executor.md)(park/unpark 的 token 語意——Notify 的前身)。

## 一個變換打通全部

| | blocking 原語 | async 原語 |
|---|---|---|
| 睡的單位 | 執行緒 | task |
| 等待佇列存什麼 | 睡著的執行緒(futex/park) | `Waker` |
| 睡 | `park` / `cv.wait` | 回 `Pending` |
| 醒 | `unpark` / `notify` | `waker.wake()` |
| 醒來的紀律 | `while` 重查 predicate | re-poll 重試 |

**rendezvous 三部曲**:`JobHandle`(one-shot、condvar 睡)→
`JoinFuture`(one-shot、waker 睡)→ 本模組(可重複使用、waker 睡)。
三章的等待端程式碼幾乎同形——變的只有「睡」那一行。

## Condvar 為什麼不直譯

Condvar 綁著一把鎖(wait 要原子地「放鎖 + 睡」);async 世界把它拆成
兩個更小的原語:**Notify**(等通知,= park/unpark 的 permit 語意)與
**AsyncMutex**(等鎖)。predicate-wait 變成
`loop { if pred { break } notify.notified().await }`——`while` 換裝,契約不變。
`bounded_queue` 的 async 對應則是 Semaphore / async channel
(`Semaphore(capacity)` + `Semaphore(0)` 兩顆就是 bounded queue;
`tokio::sync::mpsc` 是工業版),`AsyncMutex` = `Semaphore(1)`。

## Lost wakeup 在哪裡被殺掉

「查狀態」與「登記 waker」在**同一把 std Mutex** 下原子完成——
「查完沒鎖、登記之前對方 unlock 了」這條 race 在結構上不存在。
若把狀態拆成 `AtomicBool` + 另一把 waiters 鎖(更快的版本),
就必須「登記後再重試一次 CAS」補洞——與 bounded_queue「醒來重查」
是同一顆肌肉。教學版選一把鎖:正確性看得見。

## 內部用 std Mutex 不是作弊

「async 裡不准用 std Mutex」是訛傳。真正的規則:**guard 不跨 `.await`**。
本模組的臨界區是幾個欄位的讀寫(~幾十 ns)、絕不跨讓位點——合法且比
無鎖版好講。反例(經典 bug):std guard 跨 `.await`,task 掛起不放鎖,
同 worker 的其他 task 要鎖 → 整條 thread 卡死;tokio 的 guard 是 `Send`、
std 的不是,編譯器擋你一半——這句是「被咬過」的訊號。

## 什麼時候才需要 AsyncMutex(cost model)

幾乎都不需要:std Mutex 無競爭 ~20ns,臨界區短就用它。
AsyncMutex 的唯一必要場景:**guard 要活過 `.await`**(序列化一個 IO 資源)。
代價:contended path 每次多付 waker clone + 佇列操作 + 二次 poll。

## 誠實邊界

- **取消**:`.await` 到一半被 drop 的 LockFuture 不撤銷已登記的 waker,
  unlock 可能把喚醒交給死人,下一位要再等一輪。production(tokio)在
  future 的 Drop 裡轉交喚醒;教學版宣告不做。
- **公平性**:新 task 可 barging,被喚醒者撲空重排——吞吐好、極端下
  餓死隊首;tokio Mutex 做 FIFO 交棒。
- 重複登記:spurious 後 re-poll 會多 push 一張 waker——多餘那張只造成
  一次無害的 spurious wake。

## Production 對照 / pad 上怎麼用

pad 有 tokio:`tokio::sync::{Mutex, Notify, Semaphore, mpsc}` 直接用,
手搓版是概念題彈藥(「Notify 跟 park/unpark 什麼關係?」——permit 語意,
一張、飽和、不累積)。面試裡若被追問實作,waiters 佇列 + 一把鎖的版本
十分鐘寫得完——drill 就是這個形狀(`drills/src/runtime/async_sync.rs`,四洞)。
