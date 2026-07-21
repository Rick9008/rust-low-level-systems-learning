# mini_runtime 設計取捨

對應程式碼:`reference/src/runtime/mini_runtime.rs`。相關:
[executor](executor.md)(Waker 協議與 park token)、
[event_loop](../io/event_loop.md)(V1 poller 的底層)、
[fd_registry](../io/fd_registry.md)(interest table)、
[async-runtime-anatomy](async-runtime-anatomy.md)(概念總圖——本模組是它的可執行版)。

## 缺的那條線

`executor` 會 poll 但不懂 IO;`event_loop` 懂 readiness 但不懂 future。
中間缺的是兩個轉換:**readiness → wake**(reactor 查 interest table 叫醒 task)
與 **WouldBlock → 登記**(future 把 waker 留給 reactor 再 `Pending`)。
mini_runtime 就這兩件事,加一個 run queue。

## Poller trait:面試 stub 的兩個真實作

面試裡 epoll 以三行 stub 帶過(coderpad-constraints 的 Abstract the Noise);
這裡證明那個 stub 是真的可抽換邊界——**兩個實作,runtime 與 future 一行不改**:

| | V0 `ScanPoller` | V1 `EpollPoller` |
|---|---|---|
| 機制 | 睡一個 tick,把所有 armed token 全數回報 | `epoll_wait`,只回報 ready |
| 每輪成本 | O(n_armed) 次 re-try syscall | O(n_ready) |
| 延遲 | ≤ tick | ~0 |
| 依賴 | 純 std(**pad 上寫得出來**) | epoll(Linux) |
| 正確性 | 相同 | 相同 |

V0 合法的原因是契約:**wait 允許 spurious,醒了不代表好了,re-try 才算數**
——與 condvar 的 predicate-wait 同構。N=10,000、ready=10 時兩者差 1000×
(cost-model 第三節),但測試分不出它們——效率與正確性是兩個座標軸。

## run loop 的一個真 bug:reactor 餓死

第一版 run loop 只在「無事可做」時才 poll reactor。結果:一個自旋的 task
(不斷 yield 自我喚醒)讓 loop 永遠覺得「有事」,**IO 事件永遠不被收割**
——live-lock。修法:**reactor 每圈都 poll**,忙的時候 timeout 0(只收割不睡)、
閒的時候才睡。tokio 的 worker 同款結構(每輪 maintenance 收割 driver)。
這個 bug 值得記住:它是「executor 與 reactor 共用一條 thread」這個設計
天生的公平性問題。

## interest table = FdRegistry<Waker>

token 帶 generation:連線關閉(`Drop` → unregister,gen +1)後,
poller 裡可能還躺著舊 token 的 readiness——查表 `None`,事件自然丟棄。
[fd_registry](../io/fd_registry.md) 的 stale-dispatch 防禦在 runtime 裡的實戰位。

## 誠實邊界

- **一個 fd 一個 waker 槽**:同連線同時讀+寫要 reader/writer 兩個槽
  (tokio 的做法),本模組宣告不做。
- **idle 睡覺帶 20ms 上限**:跨執行緒 wake(timer thread)沒有 eventfd
  通知路徑,靠上限保底;production 用 eventfd 自我喚醒把它變成無限等
  (`event_loop::WakeHandle` 就是那個機制,V1 可以接上,留作延伸)。
- **connect 是同步的**:真 async connect = `EINPROGRESS` + 等 WRITABLE
  + `SO_ERROR` 檢查,聲明後略過。
- 單執行緒:CPU-bound task 凍住一切(與 `server_evented_inline` 同病;
  解法同樣是 offload,見 [file_io_offload](../io/file_io_offload.md))。

## Production 對照

tokio = 本模組的每個簡化各自展開:mio(跨平台 poller)、每 fd reader/writer
雙 waker、多 worker + work-stealing、timer wheel 進 driver、eventfd unpark。
結構同構,規模不同——面試講這句就夠。
