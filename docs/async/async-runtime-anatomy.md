# async runtime 解剖:Executor × Reactor × Proactor

面試概念題最常沿這條線往下鑽。本 repo 三個模組各佔一角——
`executor`(誰該跑)、`event_loop`(誰該醒,readiness 版)、
`file_io_offload`(readiness 的極限)——這份文件把三塊接起來,
並補上 repo 沒有實作的第四塊:proactor(completion 版的「誰該醒」)。
互動版:[`docs/artifacts/async_runtime.html`](artifacts/async_runtime.html)。

> 2026-07 起,「接起來」不只在紙上:`reference/src/mini_runtime.rs` 把
> executor × reactor 真的縫起來(多 task run queue + IO futures +
> 可抽換 `Poller`:V0 O(n) scan → V1 epoll),取捨見
> [`mini_runtime.md`](mini_runtime.md)。

## 1. 分工:誰該跑 vs 誰該醒

- **Executor**:維護 run queue、poll task、把 `Waker` 塞進 `Context`;
  沒事就睡(park),被 wake 就把 task 排回隊。**不認識 fd。**
- **Reactor**:集中登記「誰在等什麼事件」,一次 block 在全部等待上,
  誰的事件到了就呼叫誰的 waker。**不認識 task。**
- 兩者唯一的耦合是 `Waker`:executor 造它、葉子 future 搬運它、reactor 引爆它。

```text
executor: poll(task) ──► 葉子 future 試 read ──► WouldBlock
                              │
                              ▼
              reactor.register(fd, READABLE, waker.clone())
                              │
          Pending ◄───────────┘        executor 沒事做 → park
    ⋯
reactor: epoll_wait 回來,fd ready ──► 查 interest table ──► waker.wake()
                              │
                              ▼
              task 回到 run queue(unpark)──► 再 poll ──► read 成功 ──► Ready
```

本 repo 對映:`executor` 的 `block_on` 是最小 executor
(park/unpark 的 token 語意讓「wake 先於 park」不丟);
`event_loop` 是 callback 形式的 reactor——把 callback 換成 waker 表就是
runtime 裡的那顆 reactor。

## 2. 成本模型:不是每個 future 一條 thread

`async fn` 疊一百層還是**一個 task、一次 poll**;需要 wake 來源的只有最底下
真正卡住的**葉子等待點**。所以真正的問題是:「每一類葉子,有沒有辦法
用一個東西等全部?」

| 葉子 | std 有沒有「一次等全部」的原語 | 結果 |
|---|---|---|
| timer | 有:`BinaryHeap<(deadline, Waker)>` + `Condvar::wait_timeout` | **一條 thread 服務所有 Delay**(tokio timer wheel 的概念雛形) |
| socket | 沒有:select / poll / epoll,std 一個都沒包 | 純 std 只能 thread-per-connection;epoll 讓一條 thread 等 N 個 fd |
| regular file | 這個問題無意義:檔案在 epoll 眼中永遠 ready | 只能 offload 到 thread pool(`file_io_offload`),或 completion model |

所以 thread-as-reactor 的真實成本落在「每**類**等待一條 thread」(timer)到
「每**個**等待一條 thread」(socket)之間,取決於該類有沒有多路等待原語。
epoll 買到的是把 socket 那一格從 O(connections) 條 thread 壓成 O(1)——
它是**效率條件,不是 async 的存在條件**。

## 3. Reactor vs Proactor

| | Reactor(readiness) | Proactor(completion) |
|---|---|---|
| 通知語意 | 「fd **可以**讀了」 | 「read **做完了**,資料已在你 buffer 裡」 |
| IO 誰做 | 你的 code(nonblocking syscall) | kernel / library |
| buffer 何時交出 | 通知之後才拿出來用,一直在你手上 | **提交時就交出去**,完成前不能碰 |
| 代表 | epoll、kqueue | io_uring、Windows IOCP(Boost.Asio 是模擬層) |
| regular file | 無效(永遠 ready) | 有效——真 async file IO |

## 4. Proactor 世界裡,executor 與 event loop 長什麼樣?

**Executor 完全不變。** 還是 run queue + poll + `Waker`——它從頭到尾只認識
「task 能不能往前走」,不在乎葉子等的是 readiness 還是 completion。
變的是兩塊:

**(a) 葉子 future 的合約反過來。**

```text
readiness 葉子(epoll):
  poll → 試 read → WouldBlock → 登記 interest → Pending
  (wake 後)再 poll → 這次自己做 read → Ready(n)

completion 葉子(io_uring):
  poll(第一次)→ 把 (op, buffer 所有權) 提交進 submission queue → Pending
  (wake 後)再 poll → 從 completion 拿回 (result, buffer) → Ready(n)
  —— 不再「試」任何東西;IO 在你睡覺時已經被 kernel 做完了
```

**(b) reactor 換成 completion driver,event loop 骨架跟著變。**

```text
reactor event loop(epoll):        completion event loop(io_uring):
loop {                             loop {
  n = epoll_wait(&mut events)        submit(&mut sq)              // 批次送出
  for ev in &events[..n] {           n = wait_cqe(&mut cq)
    let w = interest[ev.fd]          for cqe in &cq[..n] {
    w.wake()                           let (w, buf) = inflight.remove(cqe.id)
  }                                    deliver(cqe.result, buf)
}                                      w.wake()
                                     }
                                   }
```

差異三條:

1. **表的內容不同**:interest table 存「fd → 誰想被叫醒」;
   in-flight table 存「op id → 誰在等 + 它的 buffer」。
2. **syscall 節奏不同**:epoll 是每個 ready fd 醒來後各自再補一次 read
   syscall;io_uring 是 submit 一批、收割一批,syscall 數與操作數脫鉤
   ——高負載下這是它快的主因之一。
3. **取消語意不同——這條對 Rust 最痛。** readiness 模型下 cancel = drop
   future:buffer 從來沒離開過你手上,反登記即可,頂多一次假醒。
   completion 模型下 op 還在 kernel 手上,**buffer 不能還你**——drop 掉
   future 卻回收 buffer 就是 use-after-free。所以 io_uring × Rust 要嘛
   buffer 用 owned 語意進出(tokio-uring 的解法),要嘛 cancel 本身也是
   一個要等完成的 op。Rust 的 async(poll-based、cancel-by-drop)是圍繞
   readiness 設計的,proactor 塞進來天生彆扭——面試講得出這一層,
   比背 io_uring API 值錢。

順帶對齊:tokio 今天 = **executor(work-stealing)+ readiness reactor
(mio → epoll)**;io_uring 生態(tokio-uring、glommio、monoio)是
completion driver,且多半改走 thread-per-core 而非 work-stealing——
buffer 綁在 op 上之後,task 亂跑核心的代價變高。這是「executor 合約不變,
但最優策略被 IO 模型牽動」的好例子。

## 5. CoderPad 實戰對應(2026-07-15 實測)

- 手搓 mini runtime **可考**:`block_on` + `Wake` + park/unpark + 一個 Delay
  ——全 std,`reference/src/executor.rs` 就是答案卷。
- timer reactor 純 std 寫得出來(§2 的 heap + condvar),加分題等級的展示。
- socket 的 std 答案是 thread-per-connection;要 async IO 就上 tokio(pad 有)。
- raw syscall 綁定實測**可用**(`unsafe extern "C"` 宣告 `epoll_create1` /
  `eventfd` 直接拿到 fd)——epoll demo 技術上可行,但 45 分鐘手搓完整
  event loop 是壞賭注;它的價值在概念層 + 鎮場 demo。
- io_uring 在 pad 上沒有(沒 crate,手綁 SQ/CQ ring 更不現實)——
  它是 proactor 概念題的錨點,不是 coding 題。
