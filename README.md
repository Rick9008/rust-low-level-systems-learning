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

## 學習路徑(依面試環境分兩級)

面試在 CoderPad:單檔、Cargo 只有固定 crate 清單——**無 libc、無 tokio、無 crossbeam**
(細節見 [`docs/coderpad-constraints.md`](docs/coderpad-constraints.md))。
沒有 libc、單檔裡也不現實手寫 `unsafe extern "C"` syscall 綁定,
所以 **epoll 一族在面試環境裡做不了**——它們降級為 deep-dive 材料,不是白學,
是拿來回答 readiness model / event loop 概念題,以及看懂真實系統。

### 【TPS 直接相關 — 優先】

CoderPad 做得了、面試會考。**這個編號清單就是建議閱讀順序**,
每個模組走三層:讀 `reference/` → 填 `drills/` → 有 ★ 的從 `challenges/` 空白手搓:

1. `bounded_queue`:Mutex + Condvar 的 predicate-wait、close 語意、滿/空邊界
2. `thread_pool`:worker 醒來先查 stop、drop 時 join 全部、graceful shutdown
3. `ring_buffer`:bounded ring 的 head/tail/len 算術與 wrap 邊界
4. `spsc_ring`:兩個 atomic index + acquire/release、power-of-2 mask、
   `#[repr(align(64))]` 防 false sharing ★
5. `executor`:mini `block_on`,`std::task::Wake` + Arc 做 Waker、
   thread::park/unpark 的 token 語意(wake 先於 park 不丟)★
6. `lru`:HashMap<K, index> + 放在 Vec 裡的 index-based 雙向鏈表,O(1) get/put ★
7. `hw_bridge` 的 **protocol + framer**:wire format `[u32 len(BE)][u8 opcode][payload]`、
   `try_decode` + `FrameReader` 的 read-buffer parse loop
   (半個 frame / 多個 frame 正確切分)★
8. `dsu`:union-find,path compression + union by rank,α(n) ★
9. `sharded_map`:per-shard Mutex 降鎖競爭,shard 選擇與整體不變量 ★

次優先(CoderPad 做得了、一般面試常見,但非 TPS 核心考點,時間有限就往後排):
`graph`(BFS / DFS / Kahn's topo / Dijkstra)、`trie`、`tree`。

### 【deep-dive 材料 — 不會考,讀懂即可】

原因如上:CoderPad 無 libc,epoll 在面試環境做不了。讀到「能把機制講清楚」為止,
不必手搓:

- `arena_lockfree`:arena + generation-tagged index 的 lock-free stack,
  示範 index-ABA 與 generation 解法(`spsc_ring` 的延伸)
- `epoll_sys`:unsafe extern "C" 的最小 epoll 綁定 + 安全 wrapper(不依賴 libc crate)
- `event_loop`:register / epoll_wait / dispatch;LT 與 ET 都示範;eventfd self-wake
- `tcp_echo`:nonblocking TCP echo;write 塞住 → 緩存 + EPOLLOUT
- `file_io_offload`:file IO 用 thread pool offload(readiness vs completion)
- `hw_bridge` 的 `server_threaded` / `server_evented`:thread-per-connection 與
  event-loop 兩種並發模型並存對照(framer 本身在優先級,見上)

(commit 歷史仍按難度分 stage,`git log --oneline --reverse` 是照 stage 走的另一種讀法;
上面的順序是把 stage 順序按面試優先級重排過的版本。)

每個主題在 `docs/` 有一份設計取捨文件(非 code 重複),各模組 doc 有交叉連結。

## 互動教材

`docs/artifacts/` 有 17 份互動教材,每個模組一份——瀏覽器直接開,無需 build:

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

順序照上方「學習路徑」優先級清單的 ★ 走:`spsc_ring` → `executor` → `lru` →
`hw_bridge` → `dsu` → `sharded_map`。`tcp_echo` 的 challenge 屬 deep-dive 級,可跳過。

範圍註記:`hw_bridge` challenge 聚焦 **`try_decode` + `FrameReader`**(45 分鐘可寫完的核心),
server/client 用 reference 版接起來當整合測試 harness;`tcp_echo` challenge 同理,
epoll 綁定已提供,只從頭寫 accept/read/write 迴圈。

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

解法在 `reference/src/sync_shim.rs`。核心演算法(`spsc_ring/core_impl.rs`、
`arena_lockfree/core_impl.rs`)一律寫 `use crate::sync_shim as sync`,**不直接碰 std**:

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

**允許用 crate 的話 epoll 長什麼樣?** 見 [`docs/epoll_libc.md`](docs/epoll_libc.md)——
libc 版的完整可跑實作(已編過跑過)、它省掉什麼(`repr(packed)`、errno)、
它沒省掉而且會咬你什麼(常數是 `i32` 但欄位是 `u32`,而 `EPOLLET` 是負數),
以及 `raw syscall → libc → mio → tokio` 這條線各自把什麼扛走。
