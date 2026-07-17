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
