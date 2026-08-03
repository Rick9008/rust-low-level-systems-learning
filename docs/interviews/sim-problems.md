# 全場模擬題本 i–n(spec-heavy,45m 計時)——virtual onsite 準備

**題幹與 harness 介面一律英文**——面試全英文,讀英文 spec 本身就是練習。中文對照(review/複盤用):[sim-problems-zh.md](sim-problems-zh.md)。

**運作方式**:開跑時只讀本檔該題的 Phase 1;Claude 當面試官(拿著 [sim-interviewer-guide.md](sim-interviewer-guide.md),⚠ 你在跑題時不准開)。clarify 用打字來回,拿到的答案就是 spec;**Phase 1 被面試官驗收後才會給 Phase 2**。

**Harness 已入庫**:`rehearsals/src/sim_{i,j,k,l,m}_*.rs`。上半「題目給的介面」= 你在 pad 上會看到的東西,可讀;**下半 mock/SimBus 實作區藏著 clarify 答案,跑題前不准細讀**。作答直接寫在該檔的作答區 + 檔尾自寫測試,`cargo test -p rehearsals sim_<x>` 跑你的測試;參考測試(`tests/sim_*_test.rs`)與 sol 跑完才開。

時間預算:clarify ≤10m → Phase 1 ~20m → Phase 2 ~15m → 收尾宣言。

---

## Sim i — DMA dispatcher v2

**Phase 1 (given at start):**

> You are writing the dispatch layer for a DMA subsystem with **6 DMA engines** (ids 0–5). DMA requests arrive from upstream; each request covers `block_nums` consecutive blocks starting at `block_start_pos`. An engine processes **one block at a time**. Your job: receive requests, feed blocks to engines, and report each request when **all** of its blocks are done.
>
> ```rust
> struct DmaRequest { request_id: u64, block_nums: u32, block_start_pos: u64 }
> fn get_dma_request() -> Option<DmaRequest>;
> fn send_dma_request_to_engine(engine_id: u32, block_num: u32, block_start_pos: u64);
> fn get_dma_result_done() -> Option<u32>;   // engine id that just finished
> fn wait_event();
> fn submit_dma_request_result_done(request_id: u64);
> ```
>
> Implement `fn run()`. Ask any questions you need.

**Phase 2**:面試官驗收 Phase 1 後口頭給。

## Sim j — Sensor interrupt pipeline

**Phase 1:**

> A sensor raises an interrupt when its hardware FIFO crosses a watermark. Your ISR runs in **interrupt context**. Samples must reach a logging thread without stalling the interrupt path.
>
> ```rust
> fn read_fifo() -> Option<Sample>;              // ISR context only
> fn ring_try_push(s: Sample) -> Result<(), Full>;
> fn ring_try_pop() -> Option<Sample>;
> fn wake_worker();
> fn sleep_until_woken();                        // worker side
> fn log(s: Sample);                             // slow; worker side only
> ```
>
> Implement `fn isr()` and `fn worker_loop()`. Ask any questions you need.

## Sim k — Per-core telemetry fan-in

**Phase 1:**

> A machine has **N worker cores**, each producing telemetry records. Records must reach a single aggregator thread that writes them out. Producers must never block on the aggregator.
>
> ```rust
> fn core_count() -> usize;
> fn spsc_new(cap: usize) -> (Producer, Consumer);   // per-core channel
> fn produce_hook(core_id: usize, f: impl FnMut(Record));  // your producer-side code
> fn write_out(r: Record);                            // aggregator side, slow
> fn park(); fn unpark_aggregator();
> ```
>
> Implement the producer hook and `fn aggregator_loop()`. Ask any questions you need.

## Sim l — MMIO command queue

**Phase 1:**

> You drive a hardware accelerator through a memory-mapped **submission ring** and a **completion ring**. To submit: write a descriptor into the next submission slot, then write the new tail index to the **doorbell register**. The device consumes descriptors and posts completions to the completion ring, advancing a completion tail you can read.
>
> ```rust
> fn mmio_write(reg: Reg, val: u64);
> fn mmio_read(reg: Reg) -> u64;      // Reg::SubmitHead, Reg::Doorbell, Reg::CompTail
> fn slot_write(ring: Ring, idx: usize, d: Descriptor);
> fn slot_read(ring: Ring, idx: usize) -> Descriptor;
> fn barrier();                        // full memory barrier to the device
> ```
>
> Implement `fn submit(cmd: Command) -> Result<(), Full>` and `fn poll_completions(on_done: impl FnMut(Command))`. Ask any questions you need.

## Sim n — Priority job scheduler

**Phase 1:**

> A compute node has **4 worker slots**. Jobs arrive from upstream, each with a priority (higher = more urgent). Assign jobs to free workers — most urgent runnable job first — and report each job when it completes.
>
> ```rust
> struct Job { job_id: u64, priority: u8, deps: Vec<u64> }   // deps: always empty for now
> fn get_job() -> Option<Job>;
> fn assign_job_to_worker(worker_id: u32, job_id: u64);
> fn get_worker_done() -> Option<u32>;    // worker id that just finished
> fn wait_event();
> fn submit_job_done(job_id: u64);
> ```
>
> Implement `fn run()`. Ask any questions you need.

## Sim m — Engine watchdog(R1 延伸)

**Phase 1:**

> Same setting as the DMA dispatcher, but engines occasionally **hang and never report done**. Requests must still complete. You now have:
>
> ```rust
> fn now_ms() -> u64;
> fn wait_event_timeout(ms: u64);     // replaces wait_event()
> // plus the six R1 APIs
> ```
>
> Extend your dispatcher so a hung engine cannot stall a request forever. Ask any questions you need.

## Sim o — Boot-order planner(algo 系;7 題制首發,8/2 深夜新增)

**形式註記**:本題走 drills 填空(`drills/src/ds/boot_planner.rs`),無 rehearsal harness;考的是「演算法穿硬體皮」——但 Big-O 與圖論直覺要在硬體場景裡拿得出來。

**Phase 1:**

> You are writing the provisioning planner for a rack: `n` nodes must boot, and dependencies say "a must be fully up before b starts" (the BMC before its host, storage before the scheduler). Booting node `v` takes `boot_ms[v]`. Compute a boot plan:
>
> ```rust
> struct BootPlan { waves: Vec<Vec<u32>>, makespan_ms: u64, critical_path: Vec<u32> }
> fn plan_boot(n: usize, deps: &[(u32, u32)], boot_ms: &[u64]) -> Result<BootPlan, Cycle>;
> ```
>
> `waves[i]` = nodes that may boot simultaneously in round `i`. If the dependency graph has a cycle, **report the nodes on it** — ops needs to know *which* machines are waiting on each other, not just a boolean. Ask any questions you need.

**Phase 2:**

> Mid-rollout, node `f` dies. Which nodes can no longer boot?
> `fn blast_radius(n: usize, deps: &[(u32, u32)], failed: u32) -> Vec<u32>`
