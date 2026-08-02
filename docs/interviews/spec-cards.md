# Spec-heavy 認題卡(7 張)——題幹

R1 題型的 15–20m 縮版,練「長英文 spec → clarify → state 表」的前 10 分鐘,不寫完整 code。

**流程(每張)**:讀 Context(spec 故意有洞)→ 寫下你要問的 **≥3 個 clarify 問題** → **30 秒英文定界出聲** → 紙上畫 **state 表**(誰持有什麼 state、事件怎麼路由)→ 開 [spec-cards-answers.md](spec-cards-answers.md) 對答案。⚠ 沒講完不准開答案鍵。

對應肌肉:SP=signal_pipeline|FP=c(framer)|FR=e2(fd_registry)|TA=f(aggregator)|TQ=h(timer)|HW-L=l|HW-M=m。

---

## Card SP — Sensor interrupt pipeline

**Context.** A sensor raises an interrupt when its hardware FIFO crosses a watermark. Your ISR runs in interrupt context. Samples must reach a logging thread. Implement `isr()` and `worker_loop()`.

**Provided API:**
```rust
fn read_fifo() -> Option<Sample>;        // ISR context only
fn ring_try_push(s: Sample) -> Result<(), Full>;
fn wake_worker();
fn sleep_until_woken();                   // worker side
fn log(s: Sample);                        // slow; worker side only
```

> **批改 2026-07-31(重做,15:56–16:15 ≈ 19m,錶內)**:五驗收 ①✓ ②✓ ③✗ ④✓ ⑤⚠,過線。
> ✓ 洞①關(沒提 drop-oldest;Full → 丟新+計數,與 try_push-only 一致;freshness 問題主動問);
> ✓ 洞②關(口頭「丟要留帳」→ atomic counter 活到定界);✓ 缺 pop 直接從 API 面抓到(上次靠試寫);
> ✓ assume 槽首現(OOM 假設出聲)——但用錯對象:assume 蓋環境未知,不蓋自己造的洞;
> ✗ 洞③換皮復發:VecDeque 無界暫存=把有界管線改無界,過載走向 OOM 而非受控丟棄;
>   「緊了從 vecdeque 丟」是第二丟點沒接計數。修法:上界,或不開。
> ⚠ 持有權:counter 掛 ISR 欄下;正解=shared static,ISR 只寫、零持有。
> 醒睡:recheck-before-sleep 補上(7/30 缺的半);wake 語意(sticky/合併)沒問——recheck 正確性懸在此。
> ISR 細節:Err(Full) 重試兩次=中斷內空轉+FIFO 樣本擱淺沒入帳;正確=計數後繼續抽乾。
> 下次釘子:③的變形要認得(任何「再加一層 buffer/thread/map」先問:spec 的有界性還在嗎)+ 喚醒語意進 clarify 必問清單。
> 知識洞補記(賽後討論挖出):ISR≠thread,是「劫持一顆核」——單核時 ISR 執行期間全部 thread 凍結
> (worker 零進度,retry 必敗);SMP 時才可能真並行,但 IRQ 可能剛好落在 worker 的核。
> ISR 不能 sleep=沒有 task 身分可掛起;不能拿 thread 的鎖=單核必死鎖。try_push 的形狀是被這模型逼出來的。
> 形狀 / try_push 去向)✓;靠腦內試寫抓到 API 缺 pop ✓——最好的找洞方式。
> 三洞:① full policy 三度選 drop-oldest——ISR 側只有 try_push,這套 API **做不出**
> drop-oldest,可實作政策 = drop-newest + dropped 計數;② 面試官口頭需求「丟要留帳」
> 沒進最終設計——**口頭答覆也是 spec**,API 沒欄位就宣告缺件(同 ring_try_pop 的處理);
> ③ worker 側多開 thread+queue = scope creep(當場自己收回 ✓)。
> 定界宣言的 assume 槽空白(用法:把未經確認的預設講出口給面試官否決的機會);
> state 表「ISR has a queue」誤——ISR 零持有,手上只有 producer 端 + 計數。
> 醒睡紀律有摸到一半(「醒來先撈資料」有,「睡前確認空」沒說)。
> 加時延伸(本卡最大收穫):從 loop 形狀自己推到 **freshness vs completeness 是 clarify 問題**
> ——audit log(順序/完整)vs freshness feed(最新至上)是兩個不同的程式;
> 後者推到底 = conflation slot,不是 queue。「該問什麼」從試寫裡長出來,不是從檢查表。

## Card FP — Frame parser with heartbeat

**Context.** A device streams bytes over a link: length-prefixed frames, and it is expected to send a heartbeat frame periodically. Detect link death. Implement `poll_loop()`.

**Provided API:**
```rust
fn read_bytes(buf: &mut [u8]) -> usize;   // non-blocking, 0 = nothing now
fn now_ms() -> u64;
fn on_frame(payload: &[u8]);
fn on_link_down();
```

## Card FR — Device event registry

**Context.** Drivers register callbacks for device events; they may also unregister at any time. Events arrive from a queue and must be dispatched to the current owner of that device id. Implement `register()`, `unregister()`, `dispatch_loop()`.

**Provided API:**
```rust
fn next_event() -> Option<(DeviceId, Payload)>;   // queued, maybe stale
fn wait_event();
```

## Card TA — Telemetry aggregator

**Context.** Sensors push `(sensor_id, timestamp, value)` samples. Emit per-sensor min/max/avg over 1-second windows. Implement `run()`.

**Provided API:**
```rust
fn get_sample() -> Option<(SensorId, u64, f32)>;
fn emit(id: SensorId, window_start: u64, stats: Stats);
```

## Card TQ — Timer service

**Context.** Components schedule callbacks to fire after a delay, and may cancel them. Implement `schedule()`, `cancel()`, `run_loop()`.

**Provided API:**
```rust
fn now() -> u64;
fn park_until(deadline: u64);             // may wake early
```

## Card HW-L — MMIO command queue(doorbell + completion ring)

**Context.** You talk to a hardware accelerator through a memory-mapped submission ring and a completion ring. To submit: write a descriptor into the ring, then write the new tail to the doorbell register. Completions appear in the completion ring. Implement `submit(cmd)` and `poll_completions()`.

**Provided API:**
```rust
fn mmio_write(reg: Reg, val: u64);
fn mmio_read(reg: Reg) -> u64;
// SUBMIT_RING[i], COMPLETION_RING[i]: descriptor slots in shared memory
// Reg::Doorbell, Reg::CompletionHead
```

## Card HW-M — Engine watchdog

**Context.** Extend the R1 DMA dispatcher: engines occasionally hang and never report done. Requests must still complete. Implement timeout handling.

**Provided API:** R1 的六個 API + `fn now() -> u64;`(`wait_event()` 改為 `wait_event_timeout(ms)`)

## Card AG-R — Interconnect route planner(algo 系)

**Context.** A rack's accelerators are wired through switches — a weighted graph. Each link has a bandwidth (GB/s). Plan a route for a large tensor transfer from chip `a` to chip `b`: the route's usable bandwidth is the **minimum** link bandwidth along the path. Implement `route(a, b) -> Vec<NodeId>`.

**Provided API:**
```rust
fn neighbors(node: u32) -> &[(u32, u64)];   // (peer, bandwidth_gbps)
```

## Card AG-T — Telemetry aggregation tree(algo 系)

**Context.** Sensors report into leaf collectors; collectors forward up a tree to the root. Every collector has a max fan-in `F`. Collector `d` just died. Re-home its children (whole subtrees, do not split them) onto surviving collectors without exceeding any fan-in. Implement `rehome(dead) -> Vec<(child, new_parent)>`.

**Provided API:**
```rust
fn children(c: u32) -> &[u32];
fn parent(c: u32) -> Option<u32>;
fn fan_in(c: u32) -> usize;      // current, < F means slack
```
