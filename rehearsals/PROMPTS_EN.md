# English prompts — 彩排時讀這份,不讀中文版

面試是英文的:**認題、clarify、定界宣言、trade-off 收尾全程英文**。
彩排規則改為:題幹只讀本檔(中文版留在 README 當對照與出處);
clarify 卡的五問也用英文寫。開場句不變:
_"Before I start, let me make sure I understand the constraints."_

---

## a · ring_drop_oldest

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

1. how many actual nodes we will have
   maybe 1 <= nodes <= 3000? then we should use event loop for this problem
2. how big the report telemetry data we will have?
   we just take (temperatures, voltages, error counts) and all as i8, then the data size of a report is 24 bytes
3. how long does a report comes back?
   maybe 1s?
4. if the total data we cannot store, how do you expect that we handle this case?
   just discard old data?
5. what do we need to express in dashboards?
   maximum, minimum, average?
6. how the data will get aggregates?
   list datas by node?

if all take my assumption we need to calculate several things:

1. 1 _ 24 bytes _ 3000 nodes = 72000 bytes, 72000 bytes \* 3600 almost 100000000 bytes = 10^8 bytes = 100MB
   1 hour data will have 100MB

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
