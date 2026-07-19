# English prompts — 彩排時讀這份,不讀中文版

面試是英文的:**認題、clarify、定界宣言、trade-off 收尾全程英文**。
彩排規則改為:題幹只讀本檔(中文版留在 README 當對照與出處);
clarify 卡的五問也用英文寫。開場句不變:
_"Before I start, let me make sure I understand the constraints."_

---

## a · ring_drop_oldest

> **用途**:Q1 預測題(認題:"continuous stream" / "most recent N")。🔴 主菜,全程 45+30 ×2(7/19、7/23)。
> = ring_buffer→spsc 的 challenge(Part 2 就是 spsc)。第一個 clarify:**滿了怎麼辦**。寫完才開 `examples/sol_ring_drop_oldest.rs`。

A sensor produces readings (`u32`) at a fixed rate. The downstream consumer
is unreliable — sometimes it stalls. Build a fixed-capacity buffer that sits
between them.

Requirements:

- Fixed capacity, holds exactly `capacity` items (`capacity >= 1`).
- When full, you may **not block and may not reject** the new reading —
  evict the oldest one to make room.
- Keep a running count of evicted readings; a monitoring system reads it
  periodically.
- Consumption is FIFO.

Part 1: single-threaded (`SensorRing`).
Part 2: producer and consumer each on their own thread (exactly one of
each) — make it thread-safe (`channel(capacity) -> (Producer, Consumer)`).

## b · pool_graceful_shutdown

> **用途**:Q2 預測題(認題:"concurrently" / "health checks" / "no external libraries")。🔴 主菜 ×2(7/20、7/26)。
> = thread_pool 的 challenge。考點在 **shutdown 語意**(accepted 必跑完、重複呼叫安全),不在 pool 本體。

On startup the service runs health checks against a few hundred devices
concurrently. Each check is a blocking call, handled by a fixed set of
worker threads.

Requirements:

- `new(workers)` starts the pool; `submit(job)` queues work.
- On termination the service calls `shutdown()`, which must be **graceful**:
  - every job that was **accepted** (submit returned `Ok`) must run to completion;
  - when `shutdown()` returns, those jobs are guaranteed done;
  - any `submit` after `shutdown()` is rejected (`Err(Rejected)`);
  - `shutdown()` may be called more than once (signal handlers do that) —
    it must be safe.

std only (`std::thread` / `std::sync`).

## c · frame_parser_heartbeat

> **用途**:Q3 預測題(認題:"byte stream" / "protocol" / "frames")。🔴 主菜 ×2(7/22、7/25),**傷疤區,永不砍**。
> = hw_bridge framer 的 challenge。第一個 clarify:**len 含不含 header?max frame size?**

Devices send frames over TCP. Wire format:

```text
[u32 len (big-endian)][payload: len bytes]
```

`len` is the payload byte count; a frame with `len == 0` is a **heartbeat**
(no payload). TCP is a byte stream — a single `read` may hand you half a
frame, or several frames at once.

Write an incremental parser: `feed(&[u8])` consumes the newly arrived bytes
and returns **all frames completed by this call**, in stream order.
Heartbeats must be reported too. Assume the stream is well-formed (trusted
peer — no malformed handling needed).

## d · tokio_frame_server(唯一可用 crate:tokio)

> **用途**:「面試官說可用 crate」那條分支的**保險**,只跑一遍(7/24);預設路線仍 std-only + 陳述假設。
> c 題 framer 的延伸(黏包邏輯直接重用)+ idle timeout / heartbeat 保活。

A device gateway: many devices connect over TCP, speaking the protocol from
problem c — `[u32 len (BE)][payload]`, `len == 0` is a heartbeat.

Requirements:

- Write the server with tokio: `serve(listener, idle_timeout)`, runs until
  the listener errors out.
- Connections are served concurrently and independently.
- Data frame → echo it back unchanged (same wire format).
- Heartbeat → no response.
- If a connection goes more than `idle_timeout` without **any** bytes
  arriving, close it. Heartbeats count as traffic — that's what keeps an
  idle device's connection alive; that's why they exist.
- TCP still has no message boundaries — reuse your problem-c homework.

## e · event_registry(Q4)

> **用途**:Q4 預測題(認題:"event id" / "handlers" / "thousands of signals")。**recognition 級**:讀題 → 30 秒定界 → 口述 arc,不計全程(7/26)。第一個 clarify:**id 密集還是稀疏?**

Hardware signals come in tagged with an event id. Build a registry: hang
handlers on ids, dispatch events as they arrive. Thousands of distinct ids,
high event rate.

Requirements:

- `register(id, handler)` — multiple handlers per id; `dispatch(id, payload)`
  runs all handlers for that id **in registration order**, returns how many ran.
- After running, a handler reports its fate (`After::Keep` / `After::Remove`)
  — `Remove` means it's never called again.
- Unknown id is a no-op.
- Nobody registers while a dispatch is in progress (caller guarantees it).

## e2 · fd_registry(Q4 進階,JD sleeper)

> **用途**:JD 的 event registry 沉睡題(fd + generation),**你的弱點 → 例外升級為全程跑** 🔴 ×2(7/21、7/24),永不砍。
> 第一個 clarify:**fd 會回收嗎?unregister 後佇列裡的舊 event 怎麼辦?**(= stale token 驗票)

An event loop waits on tens of thousands of connections through the OS
readiness API; when an event fires, the kernel hands you back a single u64.
Build the registry: when a connection is established you register it (the
fd is a small integer the kernel gave you); when an event comes back you
use that u64 to find the connection state in O(1); on close you remove it.

Careful: after an fd is closed, the kernel **reuses the same number** for a
new connection — and stale events for the old connection may still be
sitting in the event queue. Churn is high.

Requirements:

- `register(fd, state) -> Token`; the token must fit in a u64
  (`to_raw` / `from_raw` round-trip) — that's all the room the kernel gives you.
- `get / get_mut(token)`: O(1) lookup; **a stale token (fd recycled and
  re-registered) must return `None`** and must not disturb the current tenant.
- `unregister(token) -> Option<T>`: remove and return; stale token is a no-op.
- Everything O(1); tens of thousands of fds.

## f · telemetry_aggregator(Q5)

> **用途**:Q5 預測題(認題:"can't store them all" / "aggregate" / "windows")。recognition + **7/24 配套動手**(延伸寫在 `drills/src/ring_buffer.rs` 同檔)。
> = 卡#1 的實作版:playbook Q1「就地聚合」留白的三個邊界(slot 重用 / 遲到樣本 / 未來 ts 清格)在這題落地。第一個 clarify:**window 多大?timestamp 會亂序嗎?**

A whole rack produces billions of signals — storing them all is off the
table. Aggregate into a **fixed number** of time windows.

Requirements:

- `new(window_ms, num_windows)`: memory is O(num_windows), independent of
  sample count.
- `record(ts, value)`; `stats(ts)` returns that window's current
  count / sum / min / max.
- Windows are half-open intervals `[k*w, (k+1)*w)`.
- Timestamps may arrive out of order: older than the retained range →
  reject (false); still within range → accept.
- A timestamp jumping into the future becomes the newest window; the
  windows skipped over are treated as empty.
- `stats` on a window with no data → `None`.

## g · bounded_channel(Q6)

> **用途**:Q6 預測題(認題:"producers block when full")。recognition 級(7/26)。
> bounded_queue 的 MPSC 變體(`Sender: Clone`+兩邊 drop 語意)。第一個 clarify:**capacity?close 語意?**

Build a bounded channel from scratch: producers block when it's full, the
consumer blocks when it's empty. std only.

Requirements:

- `channel(capacity)` (capacity ≥ 1) → `(Sender, Receiver)`;
  `Sender: Clone` (multiple producers), single consumer.
- `send`: full → block until there's room; receiver dropped →
  `Err(SendError(v))`, handing the value back intact.
- `recv`: empty → block; **all** senders dropped and buffer drained → `None`.
- A blocked party must wake up when the opposite condition becomes true
  **or the other side disappears**.

## h · timer_queue(Q7)

> **用途**:Q7 預測題(認題:"periodic" / "interval" / "what runs next")。recognition 級(7/18 接尾、7/26)。
> 接 clarify Q5 偵測那條線(heartbeat deadline 進 min-heap;**park, don't poll**)。第一個 clarify:**幾個 timer?精度?**

N nodes, each health-checked on its own periodic interval. Who runs next,
and how long should we sleep?

Requirements:

- `schedule(id, first_at, interval)` (interval ≥ 1; id uniqueness is the
  caller's problem).
- `next_deadline()`: the caller parks until that instant — **park, don't poll**.
- `pop_due(now)`: harvest everything due, ordered by (deadline, id); after
  firing, reschedule at **old deadline + interval**; if `now` has fallen far
  behind, missed periods must be made up.
- Time is logical milliseconds (u64) so tests can control it deterministically.

---

# Clarify cards(卡片題幹英文版——五問用英文寫)

**Card 1 · telemetry hub** — Several thousand nodes continuously report
telemetry (temperatures, voltages, error counts) to one aggregation service
that dashboards read from. The volume is far more than you can store.
Design the ingestion side.

Clarify question:

(Scale and Data rate)
1. How many nodes are we covering? And what's the data input frequencies? How the data comes? How big the data is?
    I assume that the data arrives over tcp, because it might monitoring a rack 
    If nodes less than 20, maybe we can use thread-per-connections and simple mutex on data structure are enough.
    But when  0 <= nodes <= 3000, we need to use event loop to handle the connections
    
    And if the data frequencies is 1 Hz then we can handle the connections very simple.
    But if the freq >= 1k Hz, then it's hard to handle all the data input, we might need to use a data structure to store the inputs and batch process or other tech skill
    And the description says the volume is far more than you can store, we need to know it's the per second volume very large or we are saying one day's data size is very large.
    I will take freq as 1k Hz now.

    I assume: 
    1. 0 <= the temperatures <= 200, so use u8
    2. 0 <= voltages <= 1200, so use u16
    3. 0 <= error counts <= 255, so use u8
    4. 0 <= ts(data time) <= 2^31 - 1, so use u32
    5. 0 <= nodeid <= 3000, so use u16
    1 + 2 + 1 + 4 + 2 =  10 bytes
    10 * 3000 nodes * 1000 Hz = 30000000 bytes ~= 30000 KB = 30 MB
    30000 * 60 * 60 * 24 = 1.8 GB * 60 * 24 ~= 2.6 TB 1 day
    so it's very hard to store all the data in on day even 1 sec we received.

(Usage / Spec)
2.  What do the dashboard need to show in telemetry? How do we aggregate them?
    If we need to show every node in separate, we need to store the data from every node in shard
    If all data need to aggregates together, we need a lock on a single data structure, or we just use a single thread to aggregate every new data into the only info we store. 

    Maybe we can aggregate the data into minimum and maximum and count and error counts and sum by statistics? However when sum going overflow, we need to know how long the data we need to keep
    With statistics we can store the data within a window we care.
    -> per-window aggregation
    single consumer read from the windows 

(Data Loss)
3. Under pressure, can we drop or aggregate, or is every sample required?
    1. we must need to track every data, we need to use back pressure to keep the data
    2. we can kick off the old data, then we can just pop oldest and push. memorize how many data we kickoff
    3. we can just discard the data we cannot serve.
    4. but if we can use statistics to aggregate the data, the data can just aggregates in-place(but the real data in that time might lose) 
    Back-pressure -> memory is fixed so no need to care this 

(SLA)
4. What's the scenario of this solution? Are we optimizing average throughput, or tail latency?
    1. It's for human read, then it's average throughput
    2. It's for machine / automata read, then we need to care about tail latency
    Because we are targeting dashboard, it's human read 
    
(Failure Detection)
5. How do we learn a node died?
    If it's TCP, we can check the pipe healthy, if node breaks itself, however we still need to own heartbeats for those disconnect not healthy
    We need to own heartbeats? -> it's might need to use ping and deadline timer 
    Tcp -> on shutdown, drain and exit

Let me look into our assumption:
3000 nodes at about 1k Hz
statistics are enough
dashboard-level SLA
the data flow is 30 MB/s
So single-thread event loop with batched reads, per-window aggregation, single consumer.
Memory is fixed, 'full' can't happen. Back-pressure isn't needed.
On shutdown, drain and exit.
I'll start coding.


> **重寫清單(7/17 批改;寫完就刪掉這塊)** — 方法論已進
> [`clarify-playbook.md`](../docs/clarify-playbook.md):Q1 的「就地聚合到底在做什麼」
> 與 Q2 的「反推題目有沒有被消滅」。自我批改五條在
> [`clarify-cards.md`](clarify-cards.md) 規則 5。
>
> 1. **漏了「偵測」**——七題裡沒有一題問 node 怎麼判死。半開連線 TCP 不會告訴你。
> 2. **`nodeid: i8` 裝不下 3000 台**(上限 127)→ `u16`。
> 3. **`voltage: u8` 裝不下你自己給的 1000**(上限 255)→ `u16`。實際 record ≈ 10–12 B。
> 4. **8 bytes 只改了一半**:Q2 說 8 B,observation 還寫 `24 bytes`;而 `almost 10^8`
>    又是 8 B 的答案。同一段裡混了兩代算式。
> 5. **`72000 × 3600 = 259 MB`,不是 100 MB**;而且 `× 3000` 已經在 72000 裡了,
>    不能再乘一次得出 300 GB。
> 6. **結構跟你自己的 Q5 答案打架**:只要 min/max/avg,卻留 60 筆原始樣本。
> 7. **event loop 的理由要對**:不是速率(3000 events/s 一條 thread 就夠),
>    是 fd 數 × 2 MiB stack ≈ 6 GB。
> 8. **`O(3000*60)` 沒有主詞**:誰做這件事、多久做一次?
> 9. **每問只寫了一個答案**,規則要 2 個 + 各自後果。
> 10. **沒有 30 秒定界宣言**(假設 + 結構 + full policy + shutdown)。

**Card 2 · RPC gateway** — A gateway accepts client requests and forwards
them to a backend; every request must get a response. The backend gets slow
sometimes, and requests keep arriving while it's slow. Design the gateway's
queuing and flow control.

7/19
(Scale and Data rate)
1. What's the request rate and how big is a request data size? And how the backend architecture we need to handle? How long does backend handle the request?
    For request rate:
        1. Under 10^4, then thread-per-request might be enough, 1MB * 10000 = 10GB, very close
        2. Larger than 10^5 and lower than 10^7, use epoll is good.
        3. Larger than 10^7, we need cluster and L3 router for different RPC gateway instance, because the memory will too large
    For request data size:
        I guess there's a backend name and payload and session token for auth state: 128 u8 + 1024 u8 + 256u8 ~= 1400 bytes ~= 1KB
    For backend:
        Let try this in only simple backend name with only one instance now.
        And we need different backend queue for the rpc request we need to route to the backend.
    For backend handle rate:
        1 request 50ms now.
(Data loss)
2. Under pressure, can we return a error code for the requests we can't handle? Or we need a back-pressure strategy. If do, how long does a client give up request.
    Let guess that the client will wait for 1s and if exceed return Error code
    And if the requests is too large, we need to back-pressure. We can close the epollin for client requests to the full queue backend, and gateway we need to handle the connection and check if the queue full.
    If queue full we return Error code instantly.

(Failure Detection)
3. If a backend is dead how do we learn? And if the client request drop how do we know?
    If backend can know from TCP, good. Otherwise we need a heartbeat detect and request timeout.
    If client request drop, I think we can know from tcp connection dead.

(SLA)
4. Are we optimizing average throughput or tail latency?
    If tail latency, the queue need lock-free queue, because mutex holder will be preempt by scheduler, and lock-free is targeting for p99.9 but not the throughput
    If average throughput, we can only use the lock with simple data structure, 


Let's look into the assumptions:
10^5 request per sec, with 1KB
10^5 * 1KB = 100000 KB = 100MB per seconds coming

And we need to record the client connection socket and fd data and request id, maybe 16B for id and fd.
10^5 * 16B = 1600000 B = 1600 KB = 1.6 MB per second in normal time
1s drop the client request.
1000 / 50ms = 20 requests for sequential requests
10^5 * 0.05 = 5000 requests concurrency
5000 * 16 B = 80 KB, cheap
queue capacity = 10^5 * 1 sec = 100k entries
And body will store in the kernel buffer, epoll will care this for us when tcp flow control
Under pressure:
the handle time become 500 ms
10^5 * 0.5 = 50000 requests concurrency

I will use per queue for different backend service
And use a map for name to queue.
Per queue has a event loops to process the connections.
And we need event loop + epoll to handle the connections from client and backends.
** Each queued request carries a deadline; on expiry I evict it and return 504 — it must not occupy a slot or waste backend capacity. **
Single thread event loop is enough because it's only to route the rpc.
I'll start with a plain bounded queue behind the event loop - single threaded, no locking needed.
If we later shard across threads and tail latency matters, that's where lock-free in.
Ok I'll start to code, begin with the backend queue and event loop


Bad version:
(Scale and Data rate and Scenario SLA)
1. how many client requests and backends we need to handle? And what's the backend structure? Backend is a cluster or one service one node.
    1. client requests = cr
        if 0 <= cr <= 10^7 per seconds then we need to use a cluster to handle this, distribute by a machine and multiple machine with shard request
        if 0 <= cr <= 10^5, we can use event loop to handle in a single machine, we take it in this time.
    2. backends are one service one node, and we need to authorize, and redirect to the node
        if backend is cluster then it's more complicated because we need to prepare the strategy for those nodes with same service.

    we take cr <= 10^5, backend is a service a node
    then for the client requests we need to use event loop to handle the connections and requests.
    If the client requests are <= 10^5 , the cpu-bound task isn't on the rpc gateway, so we can just use single-thread event loop without thread pool.
    and we need a map for backend name and the node address
    Furthermore, we can use some threads as thread per service, and create a queue for them.
    However, there's lots of strategy we can use for different scenario so it's complex, we use the most simple way here.
    If you think there is some constraint or condition we need to care about, we can discuss more.

(Data loss)
2. Under pressure, the requests are too much, how do we handle this?
    If we can just discard, then we can return with a error code ask user to request later
    If we can't, we need to back-pressure and timeout for exhaust case.
    We need to protect the DDOS scenario too?

(Fail Detection)
3. If there's some service nodes going down, should we do anything to backup? Or there's some request died?
    1. If the service going down and no replica nodes, we can just return Error code and ask admin to repair
    2. If request died, when we try to return the result, we will know that

Let we look into the assumptions:
10^5 requests, one event loop for every request
map for backend target and node address
we need back-pressure for huge request in a same time
return error when service is down
I will start to code for the map backend and request wire format first, focus on main logic

Furthermore:
redis cache for some cheap and simple request like CDN?

> **批改(7/17;修完這輪可刪或保留當對照)** — canonical 見
> [`clarify-answers.md`](clarify-answers.md) 卡 2;台詞:
> *"RPC can't drop, but it can **reject** — bounded + timeout makes failure
> predictable; unbounded makes it an OOM."*
>
> **失分點:**
> 1. **題幹 contract 沒讀進去**:"every request must get a response" 已經回答了
>    「可不可以丟」——不可 drop,只能「拒絕(拒絕也是回應)」或 backpressure。
>    該問的是**壓回去的邊界**(排隊上限、超時多久),不是能不能丟。
> 2. **SLA 掛名沒問(4/5 類)**:這題最值錢的數字是 client timeout budget
>    (*"How long will a client wait before giving up?"*);gateway 看 **p99**。
> 3. **零算術**——卡#1 的單位鏈沒帶過來(該算什麼見下表)。
> 4. **timeout 的第二層**:不只回錯——**過期請求要踢出隊**,不替死人排隊
>    (每請求 deadline 進 timer queue = 彩排 h;超時回 504、不佔位)。
> 5. **backpressure 機制沒具體化**:對 client **停止讀**(關 EPOLLIN),
>    TCP 收窗自動把壓力傳回 client。
> 6. **宣言缺 shutdown、full policy 沒數字**,沒 *"I'll start coding."* 收尾;
>    "Let **me** look"。
> 7. **Scope creep**:authorize / DDoS / redis cache 都不是 queuing and flow
>    control;Furthermore 段面試時整段不要講。
> 8. **偵測機制反了**:是靠 backend call 掛 deadline **超時**得知慢/死,
>    不是等回程才發現;補一句 health check / circuit breaker。
>
> **缺的姿勢:**
> - **EPOLLIN 是什麼**:epoll 訂閱遮罩裡「這個 fd 可讀」的 bit。「關掉」=
>   `EPOLL_CTL_MOD` 改成不含 EPOLLIN 的遮罩(訂閱還在,只是 kernel 不再叫你讀)
>   → 你不讀 → rcv buffer 積滿 → **TCP 收窗縮到 0 → client 的 `send()` 塞住**
>   ——backpressure 沿 TCP flow control 免費傳回去;消化完再 MOD 回來。
> - **per-backend queue,為什麼敢關「整條連線」的 EPOLLIN?** 因為 TCP flow
>   control 的單位是**連線**,不是請求——EPOLLIN 只能整條關。一條 client 連線
>   只打一個 backend(常見拓撲)→ 粒度剛好。一條連線**混打多個 backend** →
>   關整條會把去健康 backend 的請求也擋住(head-of-line blocking)——這時改用:
>   照讀照解析、對滿隊的 backend **立即回 503/504**(在解析層 shed),或
>   application-level credits。把這個粒度 trade-off 講出來就是加分題。
>
> **該算什麼 → 導向什麼答案:**
>
> | 算式 | 導向 |
> |---|---|
> | in-flight ≈ rate × client timeout(10⁵/s × 1s = 100k)| queue 深度上限(Little's law)|
> | in-flight × per-request size(100k × ~1KB ≈ 100MB)| 記憶體存不存得下 → bounded 的具體數字 |
> | 到達率 λ vs backend 服務率 µ,λ > µ 的期間 | 隊伍以 (λ−µ)/s 成長 → bounded 只需撐過 timeout 窗,再多是替死人排隊 |
> | 隊中等待 > client timeout? | 過期踢出隊——每省一筆就是還 backend 一份容量 |


**Card 3 · market data feed** — A market data feed pushes high-frequency
price ticks per symbol. The strategy side only cares about the **latest**
price for each symbol, and it reads at an uneven pace. Design the layer
between the feed and the strategies.

**Card 4 · log shipper** — Every host runs an agent that collects logs from
all local processes and ships them to a remote collector. The network
flakes a few times a day, from seconds to minutes; the application's
logging call must never block. Design the agent's buffering and shipping.

**Card 5 · sensor bridge** — A single hardware device pushes signals in via
interrupts/DMA — millions per second in bursts. Your bridge hands them to
an upstream consumer. The device has no pause mechanism whatsoever. Design
the bridge.

**Card 6 · health prober** — Periodically health-check a few hundred
machines (TCP connect + application-level ping). A dead node must be
flagged within a predictable window; the prober must not hammer its targets
or blow itself up. Design the scheduling and concurrency.
