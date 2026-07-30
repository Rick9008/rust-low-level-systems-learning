# Recognition scripts(EN)——「Description → 開口」的對分底稿

**用法(7/27 taper ①九題型掃描)**:讀 `PROMPTS_EN.md` 題幹 → **先自己英文出聲**
(30 秒定界 → clarify 問題 → 做法枚舉 → trade-off 收尾)→ **講完才准開本檔對分**,
記 ✓/⚠/✗。這份是口述的 `sol_*`——先講才開,規矩同 code 答案。
在公司(不能出聲):定界句與 trade-off 兩句改筆寫,回家 23:00 前補出聲快掃。

寫作基準:a/b/c/e2 詳(主菜),d/f/g/h 簡(recognition 級)。
傷疤句(_scar_)全部來自你自己的彩排實錄——講它們就是在講「finding your own bug」。

---

## 通用模板(五步,任何題先跑這個)

1. **Restate & scope**:_"Let me restate to make sure I've got it: we need a ___ that ___.
   Before I start, let me make sure I understand the constraints."_
2. **Clarify(≥3 問,問完宣告定界)**:_"I'll assume ___ unless you tell me otherwise."_
3. **Enumerate**:_"I see two (three) ways to do this: A ___, B ___."_
4. **Pick + justify**:_"I'll go with A because ___; the main cost is ___."_
5. **Trade-off close(硬動作:Big-O 出聲 + ≥2 個沒選的解法)**:
   _"This gives us O(___) per op. The alternatives were ___ — I'd switch if ___."_

---

## a · ring_drop_oldest(認題:"continuous stream" / "most recent N" / sensor)

**Scope**:_"This is a fixed-capacity ring buffer with a drop-oldest overflow policy —
the buffer absorbs bursts, and when full we evict the oldest reading instead of
blocking the producer. Network-packet-flow shape."_

**Clarify**:
- _"When full — drop oldest, drop newest, or block? The prompt says evict oldest — confirming."_
- _"Does the eviction counter need to be exact under concurrency, or is approximate OK?"_
- _"Part 2: exactly one producer and one consumer? That decides how much synchronization I need."_

**Approaches**:
- _"Head/tail indices into a fixed `Vec` — the classic. Empty-vs-full is ambiguous when
  `head == tail`, so I keep an explicit `len` (single-threaded) or use absolute wrapping
  counters with a power-of-two mask (the SPSC way)."_
- _"A `VecDeque` with `pop_front` on overflow would work, but it hides the mechanics
  the interviewer wants to see — I'll mention it and hand-roll."_

**Close**:_"Push, pop, and evict are all O(1), memory is O(capacity). Alternatives:
blocking (backpressure — right when data must not be lost), unbounded (memory risk).
Drop-oldest is right here because the newest reading is the valuable one."_
_Concurrent twist_:_"Under SPSC, drop-oldest makes the producer act as a consumer —
two poppers — so the clean SPSC split breaks; that's a policy-forces-structure point."_

**Scars**:pop 判空用 `head == tail`(滿=空二義 → len>cap+FIFO 毀)|`drop_cnt` 整條忘 ++|
Part 2 擅自把 contract 改成阻塞 pop(clarify miss)。

## b · pool_graceful_shutdown(認題:"concurrently" / "health checks" / "no external libraries")

**Scope**:_"A fixed thread pool with a work queue, and the interesting part is the
shutdown contract: every accepted job runs to completion, `shutdown()` blocks until
they're done, late submits are rejected, and calling it twice is safe."_

**Clarify**:
- _"Graceful means drain-the-queue, not just finish-in-flight jobs — right?"_
- _"Is the queue bounded? What should `submit` do when it's full?"_
- _"What if a job panics — take the worker down, or survive it?"_
- _"On shutdown, are late submits rejected (job handed back), silently dropped, or a caller bug? — default: reject + return, nothing lost silently. (Graceful = already-queued jobs finish; late submits are a **separate** policy.)"_

**Approaches**:_"`Mutex<VecDeque> + Condvar` for the queue; a `shutdown` flag folded
into the same predicate. Channels would also work, but hand-rolling shows the
condvar discipline this question is really about."_

**Close(兩條件句是招牌,背到能脫口)**:
_"A worker **exits** only when `shutdown && queue is empty`; it **sleeps** only when
`the queue is empty && not shutdown`. Those two predicates are the whole problem.
Everything is O(1) per operation; shutdown is O(outstanding jobs)."_

**Scars(六個,全親手抓的)**:worker 見 flag 即退、不清 queue|wait predicate 漏查
shutdown → 空佇列 hang|repeated shutdown 連鎖 panic|被喚醒後盲 `unwrap` pop →
poison 連環爆(被 `let _ = join()` 吞掉)|shutdown 側 store+notify 不拿鎖 →
lost-wakeup(修法:notify 進鎖,loom 裁決)|拿著鎖跑 job → 整池串行(0.40s→0.10s)。

## c · frame_parser_heartbeat(認題:"byte stream" / "protocol" / "frames")

**Scope**:_"This is wire-protocol framing: TCP gives us a byte stream with no message
boundaries, so I keep an internal buffer, and on every `feed` I loop: if a complete
frame is buffered, cut it; otherwise keep the partial bytes for next time."_

**Clarify(第一問永遠是它)**:
- _"Does `len` include the header itself, or payload only? That off-by-one is the
  whole protocol."_(本題:payload only)
- _"Max frame size? A malicious length would otherwise make me reserve gigabytes."_
- _"Malformed input possible, or trusted peer?"_(本題:trusted)

**Approaches**:
- _"Accumulate into a `Vec<u8>`, loop `try_decode` from the front, `drain` consumed
  bytes. Simple, O(n) amortized."_
- _"An explicit two-state machine (reading-header / reading-payload) avoids
  re-scanning the header, at the cost of more states to get wrong. For a 4-byte
  header the buffer version is fine — I'd state that and move on."_

**Close**:_"Amortized O(1) per byte, memory bounded by max frame size. Heartbeat is
just `len == 0` falling out of the same path — no special case. If frames were huge
I'd switch to the state machine and stream the payload instead of buffering it."_

**核心三行(默寫級)**:`let end = at.checked_add(4)?` →
`u32::from_be_bytes(buf.get(at..end)?.try_into().unwrap())` → BE = network order。

**Scars**:(留位——c#1 7/23 晚、c#2 7/26 的洞收在這)

## d · tokio_frame_server(認題:tokio 可用 / "many devices" / idle timeout)

**Scope**:_"Task-per-connection with tokio: an accept loop, `tokio::spawn` per
connection, and each task reuses the problem-c framer on its own buffer. Idle
handling: wrap the read in `tokio::time::timeout(idle_timeout, ...)` — any bytes,
including heartbeats, reset it by construction."_

**Clarify**:_"On idle timeout — just close, or send a warning frame first? Echo
back-pressure: if the peer stops reading, is it OK that the task blocks on write?"_

**Close**:_"Tasks are cheap (KBs, not MBs) so task-per-connection scales where
thread-per-connection dies. The std fallback is the same skeleton with threads —
fine for hundreds of devices, and I'd say so rather than hand-roll an event loop
in 45 minutes."_(std 六行骨架:`examples/tcp_skeleton_std.rs`)

## e · event_registry(認題:"event id" / "handlers" / "thousands of signals")

**Scope**:_"A dispatch table: handlers hang on event ids, dispatch runs them in
registration order, and each handler votes Keep or Remove after running."_

**Clarify(第一問)**:_"Are ids dense or sparse? Dense small ints → `Vec<Vec<Handler>>`;
sparse/u64 → `HashMap`. That's the whole storage decision."_

**Close**:_"Dispatch is O(handlers on that id); Remove via in-place `retain` keeps
order and is O(k) — fine because the caller guarantees no re-entrancy. The trap I'd
name: removing while iterating — I collect fates first or use `retain`, never index-shift
mid-loop."_

## e2 · fd_registry(Q4 進階,JD sleeper——這段背到逐字)

**Scope**:_"The kernel hands back one u64 per event, fds get recycled, and stale
events may still be queued — so this is a **generational slot map**: the token packs
`(generation << 32) | slot_index`, each slot remembers its current generation, and a
mismatch means the event belongs to a dead connection — return None, don't touch the
new tenant."_

**Clarify**:_"Confirming fd recycling and stale queued events — that's what the
generation guards. Do tokens need to survive process restart? (No → in-memory gen is fine.)"_

**Close(JD 招牌句)**:_"An **O(1) generational slot map beats an O(n) scan** on every
event, and it **rejects stale tokens for free** — the generation check is one compare.
Free-list reuse keeps memory O(live fds); generation bump uses `wrapping_add` so
wraparound is defined."_

**Scars**:`len -= 1` 逃出 `is_some` 守衛 → 偽 token 靜默腐化 len、len=0 underflow|
mask 寫 `(1<<31)-1` 少一 bit → fd ≥ 2³¹ alias(修:`tok as u32` 截斷即 mask)|
edition 2024:參數名 `gen` 是保留字,用 `generation`。

## f · telemetry_aggregator(認題:"can't store them all" / "aggregate" / "windows")

**Scope**:_"Streaming aggregation into a fixed ring of time buckets — memory is
O(num_windows), **independent of sample count**; that sentence is the whole design."_

**Clarify**:_"Window size? Can timestamps arrive out of order — and how late is
still acceptable? What happens on a jump into the future?"_

**Close**:_"`record` and `stats` are O(1): bucket = `epoch % num_windows`, and a
bucket owned by an older epoch gets reset on reuse. Skipped windows are empty by
definition; too-old samples are rejected. The alternative — storing samples and
scanning — is O(n) memory and exactly what the prompt forbids."_

## g · bounded_channel(認題:"producers block when full")

**Scope**:_"A bounded MPSC channel: `Mutex<VecDeque>` plus **two** condvars —
not-full for producers, not-empty for the consumer — and the close semantics do the
real work: receiver gone → `send` returns the value back in the error; all senders
gone and buffer drained → `recv` returns None."_

**Clarify**:_"Capacity? On full — block, or would try_send / drop-oldest serve the
caller better? Who closes first in practice?"_

**Close**:_"O(1) per op. One condvar with notify_all also works but wakes the wrong
side under contention; two condvars target the wakeup. Sender count via
`Arc` clone/drop bookkeeping — last sender drop must notify the sleeping consumer,
or it sleeps forever. Lock-free? That's the SPSC ring when it's 1-to-1; MPMC needs
per-slot sequence numbers — I'd say the words, not write it."_

## h · timer_queue(認題:"periodic" / "interval" / "what runs next")

**Scope**:_"A min-heap of deadlines: `BinaryHeap<Reverse<(deadline, id, interval)>>`
— the tuple gives (deadline, id) ordering for free. `next_deadline` is `peek`, and
the caller **parks until that instant instead of polling**."_

**Clarify**:_"How many timers? Precision requirements? What should happen when we
wake up late — skip missed periods or catch up?"_

**Close**:_"schedule/pop are O(log n), peek O(1). Two traps: reschedule from the
**old deadline**, not `now`, or every firing drifts; and a `pop_due` loop naturally
catches up after a stall because the rescheduled deadline can still be ≤ now.
At huge N I'd move to a hashed timer wheel — amortized O(1), trading precision
for throughput. And the condvar version of waiting is `wait_timeout` with a
re-checked predicate — spurious and early wakeups are harmless by construction."_

---

## sim i–n(R2 spec-heavy 題型)——定界底稿不在本檔

計時場**後**才讀:`examples/sol_sim_*.rs` 檔頭(30 秒定界句 + 取捨句的出處,
8/5 taper「六題定界唸一輪」就是唸它)與 `reference` 對應模組檔頭(5-pillar 詳解;
模組對照表見 [`README.md`](README.md) 進度狀態表下方)。
場前唯一安全的前導:`html_p/r2-onsite-visual-guide.html` 的概念前導卡(刻意不含解法)。

---

**收尾自檢(每題講完問自己)**:Big-O 有出聲嗎?有講 ≥2 個沒選的解法嗎?
傷疤有講成 "I've been bitten by ___, so I ___" 嗎?——那是 finding-your-own-bug
訊號的口述版。
