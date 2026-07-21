# rust-low-level-systems-learning

std-only 的 low-level systems 面試學習教材:concurrency、event loop、binary protocol、
面試核心資料結構。三層難度並存——**讀 `reference/` → 填 `drills/` → 從頭寫 `challenges/`**——
同一主題按自己的節奏往上爬。

## 三層怎麼用

| crate | 給你什麼 | 你做什麼 |
|---|---|---|
| `reference/` | 完整實作 + 測試 + 教學註解 | 讀。建 mental model、查 API、寫完 challenge 後 diff 對答案 |
| `drills/` | 相同模組樹,核心函式挖空成 `todo!("spec: ...")`,骨架與 helper 都在 | 讀 spec、紙上 dry-run boundary、填核心邏輯、轉綠 |
| `challenges/` | 只有 public API 簽名 + 測試檔 + 面試 prompt 風格題目 | 從空白開始整個自己寫,轉綠後 diff `reference/` |

`challenges/` 對應面試 live coding 的真實條件(空白起點)。execution 的缺口只有靠
`drills/` 和 `challenges/` 補;純讀 `reference/` 不能取代手搓。

三個 crate 的模組樹鏡射 `docs/` 的四分類:`ds/`(單執行緒資料結構)·
`concurrency/`(鎖與 lock-free)· `runtime/`(async internals;`async` 是保留字,
docs 端叫 `docs/async/`)· `io/`(event loop 與軟硬體橋接);
`iter_mutate` / `inplace_leetcode` 是語言慣用法,留在 crate 根。

## 學習路徑(依面試環境分兩級)

面試在 CoderPad:單檔、Cargo 只有固定 crate 清單——實測 Rust 1.92(2024 Edition),
**有 tokio、無 libc / mio**(細節見 [`docs/coderpad-constraints.md`](docs/coderpad-constraints.md))。
自寫 `unsafe extern "C"` 綁 raw syscall 實測**連得上**,epoll 技術上做得到;
但單檔 + 45 分鐘手搓 epoll loop 是壞賭注、tokio 又在清單裡,
所以 **epoll 一族仍分在 deep-dive**——拿來回答 readiness model / event loop
概念題、看懂 tokio 底下發生什麼。場上遇到 epoll,正確動作是三行 `Poller` trait
stub 帶過(Abstract the Noise,見 coderpad-constraints)——**不要現場掏 FFI,
那是負分動作**:時間會燒在沒人評分的地方。

### 【TPS 直接相關 — 優先】

CoderPad 做得了、面試會考。**這個編號清單就是建議閱讀順序**,
每個模組走三層:讀 `reference/` → 填 `drills/` → 有 ★ 的從 `challenges/` 空白手搓:

1. `iter_mutate`:邊迭代邊修改的六形狀(iter_mut / 寫指標 / retain_mut /
   先收集再動手 / mem::take / split_at_mut)——之後每一題的手感基礎
2. `bounded_queue`:Mutex + Condvar 的 predicate-wait、close 語意、滿/空邊界
3. `thread_pool`:worker 醒來先查 stop、drop 時 join 全部、graceful shutdown
4. `ring_buffer`:bounded ring 的 head/tail/len 算術與 wrap 邊界
5. `spsc_ring`:兩個 atomic index + acquire/release、power-of-2 mask、
   `#[repr(align(64))]` 防 false sharing ★
6. `executor`:mini `block_on`,`std::task::Wake` + Arc 做 Waker、
   thread::park/unpark 的 token 語意(wake 先於 park 不丟)★
7. `lru`:HashMap<K, index> + 放在 Vec 裡的 index-based 雙向鏈表,O(1) get/put ★
8. `fd_registry`:generational slot map——`(gen<<32)|fd` token、stale event 防禦
   (JD 點名的 "event registry" sleeper;彩排題 e2)
9. `hw_bridge` 的 **protocol + framer**:wire format `[u32 len(BE)][u8 opcode][payload]`、
   `try_decode` + `FrameReader` 的 read-buffer parse loop
   (半個 frame / 多個 frame 正確切分)★
10. `dsu`:union-find,path compression + union by rank,α(n) ★
11. `sharded_map`:per-shard Mutex 降鎖競爭,shard 選擇與整體不變量 ★
12. `signal_pipeline`:JD 本尊圖——訊號源 → SPSC → spin-then-park 消費;
    掛牌握手是 SeqCst 的實戰位(練完 5 再來)★

次優先(CoderPad 做得了、一般面試常見,但非 TPS 核心考點,時間有限就往後排):
`inplace_leetcode`(27/75/80/88/189 五道 in-place 題,`iter_mutate` 的實戰應用)、
`graph`(BFS / DFS / Kahn's topo / Dijkstra)、`trie`、`tree`;
**lock-free 佇列家族續集**(spsc_ring 的後續題保險,內線情報說沒考過但練過不虧):
`mpmc_ring`(Vyukov bounded MPMC:CAS 取號 + per-slot seq,三層全有)、
`mpsc_list`(Vyukov intrusive MPSC = tokio 遠端 wake queue,讀+drill;
「縫」= `PopResult::Inconsistent` 顯式化)、`mpsc_ring`(單消費退化實體,
reference-only:pop 免 CAS、head 連 atomic 都不是)——runtime 元件的
lock-free 升級地圖見 `html_p/runtime-lockfree-upgrade-map.html`。

優先級練完 → `rehearsals/` 計時彩排(見下方「rehearsals 使用法」)。

### 【deep-dive 材料 — 不會考,讀懂即可】

原因如上:epoll 在 pad 上「可行但不划算」,不會是題目要求。
讀到「能把機制講清楚」為止,不必手搓:

- `arena_lockfree`:arena + generation-tagged index 的 lock-free stack,
  示範 index-ABA 與 generation 解法(`spsc_ring` 的延伸)
- `mpmc_list`:Michael–Scott unbounded MPMC(教學版)——「佔位=發布合一
  ⇒ 正式 lock-free」與 help 機制;retired 節點 Drop 才回收,把
  reclamation 問題攤開講
- `ws_deque`:Chase–Lev work-stealing deque(教學版)——SeqCst fence 的
  第二個實戰位(SB litmus);loom 抓出論文版 Relaxed 降 bottom 的洞的實錄
- `rcu_snapshot`:讀多寫少的快照發布(std 版 poor-man's ArcSwap,零 unsafe)
  ——Arc 計數 = 免費寬限期;「load 指標 + 計數 +1 非原子」= std 沒有
  AtomicArc 的原因;並發 trie/graph 的工程解就是這個形狀
- `ds_sync`:同步策略對照組——同一個結構的鎖版/無鎖版並排:
  `arena_locked`(Mutex slab,對照 arena_lockfree 看機關怎麼塌縮)、
  `dsu_lockfree`(CAS parent + 隨機 priority + path halving;loom 驗證)、
  `lru_locked`(LockedLru 單鎖精確 + ShardedLru 分片——
  「精確 LRU 的 get 是寫」的實證)、
  `list_fine`(hand-over-hand 交手鎖排序 set——coarse 與 lock-free 中間那階);
  選型帳(含鎖階梯)見 `docs/concurrency/ds_sync.md`
- `epoll_sys`:unsafe extern "C" 的最小 epoll 綁定 + 安全 wrapper(不依賴 libc crate)
- `event_loop`:register / epoll_wait / dispatch;LT 與 ET 都示範;eventfd self-wake
- `tcp_echo`:nonblocking TCP echo;write 塞住 → 緩存 + EPOLLOUT
- `file_io_offload`:file IO 用 thread pool offload(readiness vs completion)
- `mini_runtime`:executor × reactor 縫起來的 mini-tokio——`Poller` trait 兩實作
  (V0 O(n) scan(pad 可寫)→ V1 epoll),runtime 一行不改;interest table
  複用 `fd_registry`
- `async_sync`:blocking 原語 async 化——AsyncMutex + Notify(condvar 睡 →
  waker 睡的三部曲第三章;有 drill 四洞,選練)
- `hw_bridge` 的五個 server:`server_threaded`(thread-per-conn)、
  `server_evented_inline`(⚠️ 反面教材:阻塞 handler 凍住 loop)、
  `server_evented`(offload + eventfd 回程)、`server_evented_sharded`
  (shard by conn:保序 × 跨連線隔離)、`server_evented_spsc`(同 evented
  換佇列:兩條 SPSC + eventfd,買 p99.9)——handler 要做 IO 時的完整對照組
  (framer 本身在優先級,見上)

executor / event_loop / file_io_offload 這三塊 + proactor(io_uring)怎麼接成一張圖:
[`docs/async/async-runtime-anatomy.md`](docs/async/async-runtime-anatomy.md),
互動版 [`docs/artifacts/async_runtime.html`](docs/artifacts/async_runtime.html)。

兩份跨模組的口述底稿(面試選型題的骨架):
[`docs/concurrency/thread-safe-spectrum.md`](docs/concurrency/thread-safe-spectrum.md)(把 X 變
thread-safe 的七站光譜)、[`docs/rust-five-axis.md`](docs/rust-five-axis.md)
(Send/Sync 推導表 + unsafe impl 三段式辯護);互動深挖版在 `html_p/`。

(commit 歷史仍按難度分 stage,`git log --oneline --reverse` 是照 stage 走的另一種讀法;
上面的順序是把 stage 順序按面試優先級重排過的版本。)

每個主題在 `docs/` 有一份設計取捨文件(非 code 重複),各模組 doc 有交叉連結。

## 互動教材

`docs/artifacts/` 有 20 份互動教材——18 個核心模組各一份、一份跨模組的
async runtime 總圖(executor × reactor × proactor)、一間 clarify 決策室
(開場五問 → 設計即時推導);index 另收錄 `html_p/` 的 9 份深讀教材
(教材推導 + 面試追問鏈 + self-quiz,與模組頁的機制模擬互補)——
瀏覽器直接開,無需 build:

```sh
xdg-open docs/artifacts/index.html
```

它們不是圖,是**可以操作的機制**。刻意把每個模組最容易錯的那一步做成按鈕:
把 `Acquire` 降成 `Relaxed`、把 `while` 換成 `if`、ET 模式下只讀一半就停手、
拔掉 generation tag——然後看它當場壞給你看。

| 模組 | 那個按鈕會讓你看到 |
|---|---|
| `spsc_ring` | happens-before 邊消失,consumer 讀到未初始化記憶體 |
| `bounded_queue` | 被喚醒的 consumer 從空佇列取值(wakeup 是提示,不是保證) |
| `event_loop` | ET 下只讀一半 → epoll_wait 再也不通知,連線永遠卡死 |
| `arena_lockfree` | 沒有 generation tag,CAS「成功」指向一個已經死掉的節點 |
| `executor` | park 沒有 token 的話,wake 先於 park 就永遠醒不來 |
| `tree` | `Rc` 成環 → strong count 歸不了零,`Drop` 不執行 |
| `hw_bridge` | 逐 byte 餵進 FrameReader,`Ok(None)` 一直等到最後一 byte |
| `signal_pipeline` | 拔掉掛牌握手的 SeqCst fence,consumer 帶著貨睡死(x86 上真的會發生) |

## 跑起來看

測試證明程式碼是對的,但建立不了直覺。`reference/examples/` 讓你**親手打進去**——
不改 library 一行,不加任何 dependency。

```sh
# terminal 1:起 server(--threaded 或 --evented,行為相同、thread 數不同)
cargo run -p reference --example hw_bridge_server -- --evented

# terminal 2:打進去
cargo run -p reference --example hw_bridge_client -- demo       # 完整劇本
cargo run -p reference --example hw_bridge_client -- drip 200   # 一次一 byte
cargo run -p reference --example hw_bridge_client -- badop      # 語意錯:連線活著
cargo run -p reference --example hw_bridge_client -- badlen     # framing 錯:連線報廢
```

**`drip` 是這裡最值得跑的一個。** 它把 5-byte 的 Ping frame 拆成 5 次 write、
每次間隔 200ms。server 那邊整整安靜 800ms,直到最後一 byte 到齊才吐出命令:

```
  [   8µs] byte 0 = 0x00   len 還差 3 byte
  [ 200ms] byte 1 = 0x00   len 還差 2 byte
  [ 400ms] byte 2 = 0x00   len 還差 1 byte
  [ 600ms] byte 3 = 0x01   len 到齊了(=1),payload 還差 1 byte
  [ 800ms] byte 4 = 0x01   frame 完整 → server 現在才能解
  [ 801ms] <- Pong
```

單元測試裡那句 `assert_eq!(reader.next_frame(), Ok(None))`——跑一次 drip,它就有了體感。
「TCP 是 byte stream,沒有 message 邊界」不再是一句話,是螢幕上的 800ms。

其餘兩支:

```sh
cargo run -p reference --example tcp_echo_server   # 單執行緒 epoll echo,開幾個 nc 打
cargo run -p reference --example loom_vs_stress    # 見下節
```

並發模型的 trade-off 也變成可測的數字——5 條連線壓著時:

| server | thread 數 |
|---|---|
| `--evented` | **2**(event loop + command worker),不隨連線數成長 |
| `--threaded` | **6**(main + 每連線一條) |

`ls /proc/$(pgrep -f hw_bridge_server)/task | wc -l` 自己數。

## 面試對映

| 模組 | 考點 |
|---|---|
| `bounded_queue` `thread_pool` `sharded_map` | concurrency 基礎:鎖、條件變數、shutdown 語意 |
| `spsc_ring` `arena_lockfree` | concurrency 進階:memory ordering、lock-free、ABA |
| `executor` | async runtime internals:Waker、park/unpark |
| `epoll_sys` `event_loop` `tcp_echo` | event loop:readiness model、LT/ET、backpressure |
| `file_io_offload` | event loop 邊界:readiness vs completion(io_uring) |
| `hw_bridge` | 橋接軟硬體 + 定義通訊協定:binary framing、並發模型取捨 |
| `ring_buffer` `lru` `dsu` `graph` `trie` `tree` | systems-level data structures:index-based、O(1) 設計 |
| `iter_mutate` `inplace_leetcode` | 借用規則下的邊迭代邊改:六形狀、寫指標、O(1) space in-place |
| `fd_registry` | event registry(JD sleeper):fd-dense slots + generation 防 stale dispatch |

邊寫邊講的數字底稿:[`docs/cost-model.md`](docs/cost-model.md)
(ns/µs 數量級、queue 三型、poll vs epoll、並發模型轉折點)。

每個模組檔頂端的 `//!` doc 依 5 pillars 結構撰寫:
**[Clarify]**(解決什麼、constraints、規模)→ **[Abstract]**(次要關注點 stub 掉往前走)→
**[Iterate]**(naive → optimized 演進可見)→ **[Trade-offs]**(關鍵決策含 Big-O)→
**[Dry-Run]**(每個核心函式至少一個逐行手 trace 的 boundary 測試)。

## drills 使用法

```sh
cargo test -p drills -- --include-ignored   # 看哪些紅(todo 測試預設 #[ignore],workspace 保持綠)
```

1. 挑一個紅的測試,打開對應檔案,讀函式上方的 spec doc comment。
2. **先在紙上 dry-run** 測試點名的 boundary(空、單元素、滿、overflow、wrap 臨界)——呼應 pillar 5。
3. 填掉 `todo!()`,移除該測試的 `#[ignore]`。
4. `cargo test -p drills` 轉綠。

## challenges 使用法

```sh
cargo test -p challenges -- --include-ignored   # 同樣以 #[ignore] 保持 workspace 綠
```

1. 讀模組 `//!` doc——它是面試 prompt:constraints、clarify points、要實作什麼,不透露怎麼做。
2. 從 public API 簽名開始整個自己寫,轉綠。
3. `diff` 對照 `reference/` 對答案。

順序照上方「學習路徑」優先級清單的 ★ 走:`spsc_ring` → `signal_pipeline` →
`executor` → `lru` → `hw_bridge` → `dsu` → `sharded_map`。
`tcp_echo` 的 challenge 屬 deep-dive 級,可跳過。

範圍註記:`hw_bridge` challenge 聚焦 **`try_decode` + `FrameReader`**(45 分鐘可寫完的核心),
server/client 用 reference 版接起來當整合測試 harness;`tcp_echo` challenge 同理,
epoll 綁定已提供,只從頭寫 accept/read/write 迴圈。

## rehearsals 使用法(計時彩排)

`rehearsals/` 是九題計時彩排,模擬 CoderPad 條件(單檔、固定 crate 清單;
見 [`docs/coderpad-constraints.md`](docs/coderpad-constraints.md))。
題 a–c 是主菜、題 d(tokio frame server)練 idiomatic async——pad 實測有 tokio;
題 e–h(event registry / telemetry aggregator / bounded channel / timer queue)
對應題型預測的 Q4–Q7,預設做 recognition 練習(讀題 → 定界宣言 → 口述 arc);
題 e2(fd_registry,generation 防 stale event)是 JD 點名的 sleeper,建議完整跑。
題目在 [`rehearsals/README.md`](rehearsals/README.md),面試 prompt 風格、不給提示;
**彩排時題幹讀英文版 [`rehearsals/PROMPTS_EN.md`](rehearsals/PROMPTS_EN.md)**
(面試全程英文 I/O,中文版當對照)。
開場 clarify(pillar 1)另有專門練法:[`docs/clarify-playbook.md`](docs/clarify-playbook.md)
五問決策表 + [`rehearsals/clarify-cards.md`](rehearsals/clarify-cards.md) 六張情境卡
(每張 5 分鐘;答案分檔,寫完才開)。

1. 計時 45 分鐘一題。實作與**你自己寫的測試**都放 `src/<name>.rs` 同一個檔案
   (`#[cfg(test)] mod tests` 在底部)——CoderPad 就是全部擠一個 buffer。
2. **先在紙上 dry-run boundary,再跑測試**(Run 按鈕紀律,字面意思)。
3. 自己的測試轉綠後,才跑參考測試對照:

   ```sh
   cargo test -p rehearsals --test <name>_test -- --include-ignored
   ```

   參考測試含刻意建構的 boundary case(預設 `#[ignore]` 保持 workspace 綠)。
   對照重點:**你的測試漏了哪一類邊界**,下次動手前要先想到它。

## loom 是怎麼做出來的

### 先看證據

```sh
cargo run -p reference --example loom_vs_stress
```

同一份**故意寫壞**的 SPSC(acquire/release 全降級成 `Relaxed`),三種驗證方式。
以下是在這台機器上實際跑出來的:

| 回合 | 驗證方式 | 結果 |
|---|---|---|
| 1 | 真 OS thread,**2,000,000 次** push/pop | **通過**,0 個異常(486ms)。debug 與 release 都一樣 |
| 2 | loom,**同一份原始碼** | **`Causality violation: Concurrent read and write accesses`**,379 **µs** |
| 3 | loom,`reference` 出貨的正確版 | 通過(6.6ms) |

兩百萬次操作抓不到的東西,loom 用 379 微秒抓到。這個落差就是 loom 存在的全部理由。

### 為什麼壓力測試抓不到

bug 是「`Relaxed` 不建立 happens-before」:consumer 看到 `tail` 前進,
不保證看得到 producer 寫進槽位的值。這在 C11 記憶體模型裡是貨真價實的 data race。

但你的 CPU 是 x86-64,而 **x86 是 TSO**——硬體本來就不重排 store-store、不重排 load-load,
`Relaxed` 編出來跟 `Release` 是同一條 `mov`。**這個 bug 在 x86 上根本沒有物理表現**:
跑 10⁹ 次也是綠的,直到有人拿去 ARM / RISC-V(弱記憶體序)跑,或編譯器某次升級決定重排它。

random fuzz 的搜尋空間是「這台機器實際會發生的 interleaving」;
bug 藏在「C11 **允許**、但這台機器不會做」的那一區。fuzz 永遠掃不到那裡。

### loom 的四個機關

**1. 型別替換。** `loom::sync::atomic::AtomicUsize`、`loom::cell::UnsafeCell`、
`loom::sync::Arc` 的 API 跟 std 一模一樣,但它們是**假的**:每一次 load / store /
UnsafeCell 存取都會回報給 loom 的執行期。這就是為什麼被測程式碼**不能直接 `use std::sync`**
——也正是 `sync_shim.rs` 存在的理由(見下)。

**2. thread 不是 OS thread。** `loom::thread::spawn` 開的是 green thread
(loom 底層依賴 `generator` crate)。任一時刻**只有一條在跑**,由 loom 自己排程。
所以執行是**決定性的**:給定一個排程,結果每次都一樣——失敗可重現,不是海森堡 bug。

**3. 窮舉 + 回溯。** `loom::model(f)` 不是把 `f` 跑一次,是把 `f` **跑上百上千次**:
每次在某個決策點(atomic 存取、鎖、`yield_now`)走一條沒走過的分支,DFS 遍歷整棵排程樹。
獨立的操作(不同位址、兩個 load)可交換,**partial-order reduction** 把這種對稱分支剪掉,
否則是階乘爆炸。loom 的技術來源是 CDSChecker(Norris & Demsky, OOPSLA'13)。

**4. 模擬 C11 記憶體模型——這才是關鍵。** loom 的 atomic 變數存的不是「一個值」,
而是**一整段寫入歷史 + happens-before 的因果圖**。一次 `Relaxed` load,loom 會依 C11 規則
算出「哪些舊值是合法可見的」,然後**真的把過期的值回給你**。x86 硬體不會這樣做,loom 會。
同理它記錄每個 `UnsafeCell` 的存取:兩次存取之間沒有 happens-before 邊、且至少一次是寫
→ 判定 data race,當場 panic。回合 2 那句 `Causality violation` 就是這樣來的。

所以 **loom 通過 = 在該模型與 bound 內「證明」沒有這類 bug**,不是「跑很多次沒炸」。

### 工程上怎麼接:sync_shim

矛盾:library 本體要 std-only(零依賴),但 loom 要求被測程式用**它的**型別——
總不能為了測試把 production 程式碼綁上 loom。

解法在 `reference/src/sync_shim.rs`。核心演算法(`concurrency/spsc_ring/core_impl.rs`、
`concurrency/arena_lockfree/core_impl.rs`)一律寫 `use crate::sync_shim as sync`,**不直接碰 std**:

- **lib 編譯時**:`sync_shim` 是 std 型別的薄殼(`UnsafeCell` 的閉包式 API 必然內聯)
  → production 路徑零依賴、零開銷。
- **loom 測試時**(`tests/loom_*.rs`):測試 crate 自己定義一個同名 `sync_shim`
  re-export loom 型別,再用 `#[path]` include **同一份**演算法原始碼。

同一份演算法、兩套記憶體模型實例化——**loom 驗過的就是 lib 出貨的那份邏輯**,
而不是一份「為了測試而寫的相似程式碼」。`loom_vs_stress` 這個 example 把同一個機關
又用了一次(而且是三次實例化:壞版 × std、壞版 × loom、好版 × loom)。

```sh
cargo test -p reference --test loom_spsc
cargo test -p reference --test loom_arena
```

### 誠實的邊界

loom 驗的是**它模擬的那個模型**:C11 的一個子集、preemption bound 之內
(`LOOM_MAX_PREEMPTIONS`)、以及**你真的寫進 model 裡的那些操作**。
它不模擬編譯器優化,不管邏輯錯,也不會幫你檢查沒被 model 覆蓋到的 API。

代價是**指數級狀態空間**:model 必須小(2 條 thread、2–3 個操作)。
loom 測試跑超過十幾秒,通常代表模型開太大了,不是 loom 慢——
`loom_spsc` 用容量 1、兩個元素,不是偷懶,是刻意。

loom 綠燈 ≠ 程式沒 bug,而是「這段演算法的這組操作,在 C11 下沒有 interleaving /
可見性層級的錯誤」。範圍很窄——但窄得非常值錢,因為這正是人類 review 最看不出來的那一類。

`proptest`(同為 dev-only)負責另一件事:資料結構不變量,隨機生成輸入、失敗時自動縮小到
最小反例。**proptest 找邏輯錯,loom 找並發錯**,兩者不重疊。

## 品質閘門(每個 commit 都過)

```sh
cargo build --workspace
cargo test --workspace              # reference 全綠;drills/challenges 的練習測試 #[ignore]
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

另外:每個 `unsafe` 區塊上方都有 safety invariant 註解;每個 complexity 標註與實作一致。

## 誠實聲明

std-only 是**面試約束**,不是 production 建議。真實 production 會用:
tokio(async runtime)、crossbeam(channel、epoch-based reclamation)、
rayon(data parallelism)、mio(跨平台 epoll/kqueue 抽象)、bytes(zero-copy buffer)。
各模組 doc 有註明對應的 production crate。唯一的非 std 依賴是 `epoll_sys` 的
raw syscall 綁定(自寫 `unsafe extern "C"`,不依賴 libc crate)。

**允許用 crate 的話 epoll 長什麼樣?** 見 [`docs/io/epoll_libc.md`](docs/io/epoll_libc.md)——
libc 版的完整可跑實作(已編過跑過)、它省掉什麼(`repr(packed)`、errno)、
它沒省掉而且會咬你什麼(常數是 `i32` 但欄位是 `u32`,而 `EPOLLET` 是負數),
以及 `raw syscall → libc → mio → tokio` 這條線各自把什麼扛走。
