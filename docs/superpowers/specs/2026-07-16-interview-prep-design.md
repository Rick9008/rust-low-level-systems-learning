# 面試準備材料四期計畫(2026-07-16 定案)

## 背景與依據

- 面試日 **2026-07-28**(CoderPad,實測環境見 `docs/coderpad-constraints.md`)。
- 2026-07-16 的 JD 攻略分析修正了 repo 部分定位(詳見 memory `interview-jd-strategy`):
  - 「不依賴外部函式庫」= 別 import rayon/tokio/crossbeam,**不是**手刻 epoll FFI;
    面試動作是三行 `Poller` trait stub(Abstract the Noise)。
  - "event registry" 是 JD sleeper:fd-dense `Vec<Option<T>>` slots + generation counter,
    token = `(gen << 32) | fd`;經典 bug 是 fd 回收後 stale event dispatch 到新 handler。
  - Drop vs backpressure 是 protocol 層決策(telemetry → drop/aggregate;
    RPC → backpressure;自排程工作 → bounded + blocking submit)。
  - lockless 買的是 p99.9 不是吞吐(syscall 1–3µs vs uncontended mutex ~20ns)。
- 使用者自評弱點:(1) clarify/參數化(JD 線索 → 問題 → 設計分支);(2) fd+generation registry。
- 時程原則:材料壓在 2026-07-21 前出完,之後使用者進入純彩排模式。

## Phase 1 — 文件修正(0.5 天)

1. `README.md:25`「必要時現場 demo 幾行綁定鎮場」→ Poller stub 定位(FFI 是負分動作)。
2. README 學習路徑:`iter_mutate` 進 TPS 優先清單最前(基本功)、`inplace_leetcode` 進次優先;
   面試對映表加一列;互動教材「17 份每個模組一份」改為範圍措辭。
3. 新增 `docs/iter_mutate.md`(iter_mutate + inplace_leetcode 合寫)。
4. `docs/coderpad-constraints.md` 加「Abstract the Noise」節:
   ```rust
   trait Poller {
       fn register(&mut self, fd: RawFd, token: u64);
       fn wait(&mut self, out: &mut Vec<(u64, Ready)>, timeout_ms: i32) -> usize;
   }
   ```
   加台詞:「底層是 epoll,`epoll_event.data` 的 u64 就是我的 token,要的話可以展開。」
5. `docs/cost-model.md` 加 tail 敘事:mutex holder 被 preempt → 所有 waiter 卡一個
   timeslice(ms 級);lockless 買 p99.9。
6. `rehearsals/README.md` escalation ladder 階段 3 補 Poller stub 形狀;
   CLAUDE.md 加結構註記(`inplace_leetcode` 僅 reference、`iter_mutate` 無 challenge 層)。

## Phase 2 — `fd_registry`(~2 天)

**Reference 模組** `reference/src/fd_registry.rs`:caller 指定 index 的 generational slot map
(與 slab 差在有 generation、與 slotmap 差在 key 由 caller 指定=fd):

```rust
pub struct Token(u64);              // (gen << 32) | fd
pub struct FdRegistry<T> {
    slots: Vec<Option<T>>,          // index = fd(fd 密集,array load 一次)
    gens:  Vec<u32>,                // unregister 時 bump
}
// register(fd: usize, v: T) -> Token(slots 自動增長)
// unregister(Token) -> Option<T>(gen 不符 → None;成功後 gen += 1)
// get / get_mut(Token) -> Option<&T> / Option<&mut T>(stale → None)
// len() -> usize
// Token::to_raw() -> u64 / Token::from_raw(u64) -> Token(epoll_event.data 往返)
```

Contract:`register` 撞上已佔用 slot 直接 panic(kernel 保證活著的 fd 不重號,
double-register 是 caller bug;doc 明寫)。

- 5-pillar 模組 doc;必備 boundary 測試:**fd 重用 trace**(register fd=5 → unregister →
  同 fd 再 register → 舊 token None、新 token 正常),外加空 registry、fd 跳躍增長、
  gen wrap 註記。全 O(1)。
- `docs/fd_registry.md`:HashMap(hash + pointer chase,cache 爛)vs `Vec<Option<T>>`
  (一次 array load)vs slab/slotmap 對比 + cost-model 數字;gen 32-bit wrap 的誠實邊界;
  與 `arena_lockfree` generation 防 ABA 的同構關係;event_loop interest table 交叉引用。
- **Drill** `drills/src/fd_registry.rs`:挖 register/unregister/get 三洞,spec doc comment +
  `#[ignore]` 測試。
- **Rehearsal 題卡 e2**:`rehearsals/src/fd_registry.rs`(API 簽名)、
  `rehearsals/tests/fd_registry_test.rs`(參考 boundary 測試,`#[ignore]`)、
  `rehearsals/examples/sol_fd_registry.rs`(對參考測試全綠後進 repo)、
  README 題目節 + 掃描表一列(「fd/handle recycling」→ e2 → 第一個 clarify:
  **fd 會回收嗎?dispatch 與 unregister 誰先?**)。

## Phase 3 — Clarify playbook + 情境卡 + 互動 artifact(~2 天)

- `docs/clarify-playbook.md`:五問決策表——每問 × 2-3 answer 分支 × 設計後果 × 數字:
  1. 資料掉不掉? → full policy 三分法(drop-oldest ring / aggregate / backpressure / blocking submit)
  2. 速率? → queue size = rate × 容忍落後秒數 × 樣本大小(含 1M samples/s 例)
  3. 幾個 node / rack? → thread 數與 shard 策略(per-producer ring 轉折點)
  4. SLA p50 還是 p99.9? → lock-free 值不值(tail 敘事)
  5. node 死掉怎麼知道? → heartbeat timeout → timer queue(接題 h)
  附開場句型與「問完鎖 contract、不中途改設計」紀律。
- `rehearsals/clarify-cards.md`:6 張 JD 風格情境卡(telemetry hub、RPC gateway、
  market data feed、log shipper、sensor bridge、health prober…);每張 5 分鐘練習:
  寫 clarify 問題 + 每問分支的設計後果。
- `rehearsals/clarify-answers.md`:答案卡分檔存放,沿用「不主動揭露」規則
  (CLAUDE.md 保護規則同步涵蓋)。
- `docs/artifacts/clarify_playbook.html`:互動版(選答案 → 架構隨之變:queue 型態、
  thread 數、full policy);`docs/artifacts/index.html` 收錄。

## Phase 4 — PROGRESS.md + `submit` drill(~1 天)

- 根目錄 `PROGRESS.md`(手動勾選、git 追蹤):
  - 模組 × 三層(讀 reference / drill / challenge)勾選表,初始狀態照現況填
    (如 bounded_queue drill 已完成);
  - rehearsal 計時表:欄位對齊 45 分鐘 protocol 五段(clarify / skeleton / core /
    boundary+dry-run / trade-offs)+ 一次編過?/ 哪段爆?
  - 下次複習欄(絕對日期)。
- `reference/src/thread_pool.rs` 加 `submit<T>(f) -> JobHandle<T>`:
  `Arc<Mutex<Option<thread::Result<T>>>> + Condvar` 的同步 oneshot;panic 在 `join()`
  `resume_unwind` 重拋;與 `file_io_offload::JoinFuture` 互相交叉引用
  (「同一個 one-shot rendezvous,condvar 睡 vs waker 睡」)。
- `drills/src/thread_pool.rs` 挖 submit/join 洞 + `#[ignore]` 測試;README drills 節提及。

## Phase 5 — pipeline 教材(排在 1-4 之後動工;使用者面試後再練)

材料照做、不凍結;凍結的只有使用者的練習時間(7/28 前彩排優先)。

1. handler-IO 對照組(接在 `hw_bridge`):evented server 裡 inline blocking(示範壞)→
   offload 到 pool + eventfd 回寫 → tokio 對照;drills/challenges 對應層。
2. mini-runtime,**兩階 reactor 梯子**(2026-07-16 使用者提議 O(n) 輪詢前置版):
   - **V0 scan reactor(std-only,pad 可寫)**:registry 複用 `FdRegistry<Waker>`;
     executor 無 ready task 時對全部註冊的 nonblocking fd 做 O(n) 掃描
     (`try_read` 判 `WouldBlock`),ready → `wake()`,掃完 sleep 一個 tick。
     成本:每輪 n 次 syscall + tick 延遲(接 cost-model 的 poll vs epoll 節)。
   - **V1 epoll reactor**:同一個 `Poller` trait 換 `epoll_sys` 實作,O(ready) 喚醒;
     executor 與 future 程式碼零改動——「Abstract the Noise」的可執行示範,
     trait 形狀與 coderpad-constraints 的面試 stub 完全一致。
   - executor 升級成多 task run queue(`Arc<Task>`,wake = push 回 queue)。

## 品質閘門與慣例

- 每期 1-2 個 commit,四個閘門全過(build / test / clippy -D warnings / fmt)。
- 新 drill / rehearsal 測試照慣例 `#[ignore]`,workspace 保持綠。
- sol 檔必須對參考測試全綠後才進 repo(gates 編譯 examples 防 rot)。
- 文件與註解:繁體中文;reference 模組 doc 依 5-pillar 結構;複雜度標註必須與實作一致。

## 明確不做(YAGNI)

- 不重寫 `event_loop` 去使用 `FdRegistry`(僅 docs 交叉引用)。
- 不做 io_uring / completion model 實作(維持 repo 既有聲明)。
- 不為 `iter_mutate` / `inplace_leetcode` 補互動 artifact(措辭改掉即可,面試後再議)。
- Phase 5 不搶 1-4 的工期,但材料本身照做(使用者的彩排時間 7/28 前不分給它)。
