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

## 學習路徑(git log 即閱讀順序)

commit 歷史按難度遞增分階段,`git log --oneline --reverse` 就是建議閱讀順序:

**Stage 2 —— mutex/condvar 基礎**
- `bounded_queue`:Mutex + Condvar 的 predicate-wait、close 語意、滿/空邊界
- `thread_pool`:worker 醒來先查 stop、drop 時 join 全部、graceful shutdown
- `sharded_map`:per-shard Mutex 降鎖競爭,shard 選擇與整體不變量

**Stage 3 —— 單執行緒資料結構(index-based 優先)**
- `ring_buffer`:bounded ring 的 head/tail/len 算術與 wrap 邊界
- `lru`:HashMap<K, index> + 放在 Vec 裡的 index-based 雙向鏈表,O(1) get/put
- `dsu`:union-find,path compression + union by rank,α(n)
- `graph`:adjacency list;BFS / DFS / Kahn's topo / Dijkstra(BinaryHeap<Reverse>)
- `trie`:children 放 arena 的 prefix tree
- `tree`:index-based arena 版與 Rc<RefCell> 版並列,取捨對照

**Stage 4 —— atomic / lock-free(loom 驗證)**
- `spsc_ring`:兩個 atomic index + acquire/release、power-of-2 mask、
  `#[repr(align(64))]` 防 false sharing
- `arena_lockfree`:arena + generation-tagged index 的 lock-free stack,
  示範 index-ABA 與 generation 解法

**Stage 5 —— async internals from scratch**
- `executor`:mini `block_on`,`std::task::Wake` + Arc 做 Waker、
  thread::park/unpark 的 token 語意(wake 先於 park 不丟)、一個 Delay future

**Stage 6 —— event loop / IO 綜合**
- `epoll_sys`:unsafe extern "C" 的最小 epoll 綁定 + 安全 wrapper(不依賴 libc crate)
- `event_loop`:register / epoll_wait / dispatch;LT 與 ET 都示範;eventfd self-wake
- `tcp_echo`:nonblocking TCP echo;write 塞住 → 緩存 + EPOLLOUT
- `file_io_offload`:file IO 用 thread pool offload(epoll 為何不適用 regular file)

**Stage 7 —— 橋接軟硬體(JD 直擊題)**
- `hw_bridge`:binary protocol server + client。
  wire format `[u32 len(BE)][u8 opcode][payload]`、`try_decode` + `FrameReader`
  的 read-buffer parse loop(半個 frame / 多個 frame 正確切分)、
  thread-per-connection 與 event-loop 兩種 server 並存對照、sync client。

**Stage 8 / 9 —— drills 層、challenges 層**(見下方使用法)

每個主題在 `docs/` 有一份設計取捨文件(非 code 重複),各模組 doc 有交叉連結。

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

建議順序(★ = 先做):★ `spsc_ring` → ★ `executor` → ★ `lru` → ★ `hw_bridge` →
`dsu` → `sharded_map` → `tcp_echo`。

範圍註記:`hw_bridge` challenge 聚焦 **`try_decode` + `FrameReader`**(45 分鐘可寫完的核心),
server/client 用 reference 版接起來當整合測試 harness;`tcp_echo` challenge 同理,
epoll 綁定已提供,只從頭寫 accept/read/write 迴圈。

## 驗證說明:loom 是窮舉,不是 fuzz

並發 bug 不能靠 random fuzz:一個依賴特定 interleaving + 弱記憶體序可見性的 bug,
隨機跑 10⁹ 次可能一次都不觸發。loom 是 **model checker**:

- 它接管 atomic / 鎖 / thread 的排程,把(preemption bound 內)**所有可能的 interleaving
  逐一執行**,並模擬 C11 memory model——`Relaxed` 下允許的重排它真的會排給你看。
- 所以 loom 測過 = 在該模型與 bound 內**證明**正確,不是「跑很多次沒炸」。

工程上的接法:library 本體 std-only(loom 只在 `[dev-dependencies]`),
`spsc_ring` / `arena_lockfree` 的核心演算法透過 sync-shim(`#[path]` include)
在測試裡以 loom 的 atomic 型別重新實例化。直接跑即可:

```sh
cargo test -p reference --test loom_spsc
cargo test -p reference --test loom_arena
```

`proptest`(也是 dev-only)用於資料結構不變量:隨機生成輸入、失敗時自動縮小到最小反例。

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
