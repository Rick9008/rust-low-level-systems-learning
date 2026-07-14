# executor(mini block_on)設計取捨

對應程式碼:`reference/src/executor.rs`。相關:[bounded_queue](bounded_queue.md)(predicate-wait 的同構)、[event_loop](event_loop.md)(reactor 端)。

## Future 協定,一句話版

`poll` 回 `Pending` 是一個**承諾**:「我已把你給我的 waker 交給某個未來會
叫它的人」。executor 的全部正確性建立在這個契約上——敢睡,是因為約好了有人叫。

## Waker 為什麼長這樣

- `std::task::Wake` trait 要求 `Arc<Self>`:waker 會被 clone 進任意執行緒
  (timer、IO reactor),在未來任意時刻呼叫——需要共享所有權 + Send/Sync。
- 對 block_on 而言喚醒 = `Thread::unpark`。`Thread` handle 本身是內部 Arc 的
  廉價 clone,跨執行緒安全。

## park/unpark 的 token(permit)語意——本模組的考點核心

`unpark` 存入一個**飽和的 permit**(多次 unpark 只存一個);
`park` 有 permit 就消耗並立即返回,沒有才睡。

這解掉致命的時序窗口:

```text
executor:  poll → Pending ──(窗口!)──→ park
timer:                 └ wake (unpark) ┘
```

wake 落在窗口裡:permit 已掛上,隨後的 park 直接穿過,不丟。
若用「裸 condvar notify」實作喚醒(沒有 predicate/permit),notify 在
wait 之前到達就人間蒸發——lost wakeup,executor 永眠。
這與 bounded_queue 的 predicate-wait 是同一課的兩種面貌:
**喚醒訊號要嘛帶狀態(permit),要嘛醒來重查條件(predicate)——裸訊號不可靠。**

另一半協定:`park` 允許**虛假返回**,所以醒來一律 re-poll,
由 future 自己判斷完成與否。loop { poll; park } 的形狀因此是唯一正確形狀。

## Pin,面試夠用的深度

async fn 編譯成的狀態機可能**自引用**(跨 await 的借用指向自身欄位),
搬家(memmove)會讓內部指標懸空。`Pin` 的承諾:「這個值直到 drop 不再移動」。
`std::pin::pin!` 把 future 釘在 stack frame(零配置);`Box::pin` 釘在 heap
(要跨函式傳遞時用)。block_on 裡 `as_mut()` 每輪 re-borrow 同一個 pinned future。

## Delay:leaf future 的標準形狀

- **lazy**:建構不做事,第一次 poll 才 spawn timer——future 的通則
  (不 await 不執行)。
- **waker 更新**:每次 poll 都把最新 waker 放進 slot。契約只保證
  「最後一次 poll 給的 waker」被叫;future 可能被搬到別的 task,舊 waker 叫錯人。
- **恰好喚醒一次**:timer `take()` waker。
- thread-per-delay 是刻意的 stub;production 是 timer wheel / 時間堆
  (tokio:分層 wheel,一條 driver thread 管百萬計時器)。

## 這個 executor 缺什麼(通往 production 的路)

單 future、單執行緒、無任務佇列:`spawn` 需要 task 佇列 + 每個 task 自己的
waker(喚醒 = 把 task 重新排回佇列,而非 unpark);多執行緒需要 work-stealing;
IO 需要 reactor(見 event_loop)把「fd ready」翻譯成 wake。
骨架不變:loop { poll; 沒事做就睡 }。

## Production 對照

futures::executor::block_on(同構)、tokio(多執行緒 runtime + driver)、
smol / async-executor(精簡可讀,推薦源碼閱讀)。
