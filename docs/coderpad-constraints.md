# CoderPad Rust 環境限制(面試約束)

面試在 CoderPad 上進行。這份文件記錄已確認的環境限制,以及每條限制對練習方式的
實際影響。repo 的模組分級(README「學習路徑」)與 `rehearsals/` 的彩排規則都以此為準。

> 2026-07-15 實測更新:版本與 crate 清單以**登入 pad 親測**為準。
> CoderPad 官方 languages 頁面(寫 Rust 1.59 / 2021 Edition)已過時,勿引用。

## 限制與影響

### 1. 單檔

整份解答活在一個編輯器 buffer 裡:沒有模組樹、沒有 `mod` 檔案分割、沒有
`tests/` 目錄。測試跟實作擠在同一個檔案。

**影響:**
- 練習時就用單檔結構:實作在上,`#[cfg(test)] mod tests` 在下。
  `rehearsals/` 的規則「自己的測試寫在 `src/<name>.rs` 底部」就是在模擬這件事。
- 不要依賴「拆檔整理思路」的習慣——面試時沒有這個選項。

### 2. Cargo 有,但 crate 清單固定:有 tokio,無 libc / mio

環境跑的是 Cargo,依賴是預先固定的一組,不能自己加。實測清單:
**tokio**、serde / serde_json、rand、rayon、regex、reqwest、chrono、anyhow、
thiserror、itertools、bitflags、url、uuid(另列有 core)。
**實測確認沒有**:libc、mio;crossbeam、loom、nix 也不在清單上。

**影響:**
- 併發基本盤仍是 std:`std::thread` / `std::sync`(Mutex、Condvar、Arc、mpsc)/
  `std::sync::atomic`——本 repo 的 std-only 練法完全對口。
- **tokio 可用 → async 題是可考的**(async TCP、timeout、`tokio::sync::mpsc`、
  `select!`)。repo 的 `executor` 模組管 runtime internals 概念;
  idiomatic tokio 用法是另一塊肌肉,要另外練。
- **raw syscall 實測可用**(2026-07-15 pad 親測):自寫 `unsafe extern "C"`
  宣告 `epoll_create1` / `eventfd` / `close`,連結成功、拿得到 fd;
  `TcpListener::bind` 與 `thread::spawn` 也正常。也就是說 epoll **技術上
  做得到**——本 repo `epoll_sys` 那套「不靠 libc crate」的做法在 pad 上成立。
- 但 epoll 一族(`epoll_sys`、`event_loop`、`tcp_echo`、`file_io_offload`)
  **仍不是 live coding 題**:單檔 + 45 分鐘內手搓 `epoll_event`
  (`repr(C, packed)` 的坑)+ 完整 event loop 是壞賭注,何況 tokio 就在清單裡。
  它們維持 deep-dive 定位:概念題彈藥、看懂 tokio 底下發生什麼。
  **場上不要掏 FFI**——正確動作見下方「Abstract the Noise」節。
- 沒有 loom / proptest:正確性只能靠自己當場寫的測試 + 腦內 dry-run。
  平時練習時 loom 幫你「證明」的那些 interleaving 直覺,面試時要內化成
  「我知道這裡為什麼對」的口頭論證。

### 3. Toolchain:Rust 1.92.0(2024 Edition)——實測,不算舊

pad 上實際顯示 `Running Rust 1.92.0 (2024 Edition)`。

**影響:**
- 現代語法照用:`let-else`、`std::thread::scope`、edition 2024 語法全部都在,
  不需要 MSRV 降級寫法。
- 本 repo workspace 就是 edition 2024,`rehearsals/` 跟著 workspace 走,
  語法層與 pad 對齊;整個 workspace 用 `cargo +1.92.0 test` 驗證過可編譯全綠。
- 反向的唯一風險:本機 toolchain 若比 1.92 新,別用比 1.92 還新的 std API
  (要驗就 `rustup toolchain install 1.92.0` 後 `cargo +1.92.0 test --workspace`)。
- 快速指紋:貼一個空的 `unsafe extern "C" {}`——edition 2024 **要求** `unsafe`
  (RFC 3484),而 1.82 以前的 toolchain 看到這語法直接報錯。
  一行就能驗出環境新舊。

### 4. 有 Run 按鈕 → "dry-run before you Run" 是字面意思

CoderPad 有 Run 按鈕,隨時可以編譯執行。誘惑是寫兩行按一次,用編譯器當思考的
拐杖——在計時面試裡這是時間黑洞,而且面試官看得到你每一次手忙腳亂的 Run。

**影響:**
- 實測一次 Run(編譯 + 執行)約 7 秒:edit-run-edit 迴圈的真實成本是
  「7 秒 × 你按的次數」,在計時面試裡很貴。
- 寫完一段核心邏輯,先在紙上/註解裡把 boundary case 手 trace 一遍
  (空、單元素、滿、wrap、切斷點),**然後才按 Run**。
- 這正是本 repo 5 pillars 的 [Dry-Run] 那一環;`rehearsals/` 計時彩排時
  請把「先 dry-run 再 Run」當成硬規則執行。

## 面試動作:epoll 用 stub 帶過(Abstract the Noise)

「不依賴外部函式庫」的目的,是看你知道 thread pool / queue **裡面**有什麼——
別 import rayon / tokio / crossbeam;它不是在測你會不會寫 libc binding。
epoll 相對於主結構(pool + ring)就是 rubric 說的那個次要 JSON parser:
定義一個 API stub,然後往前走。

```rust
/// 底層是 epoll_wait;面試只需要這個 shape。
trait Poller {
    fn register(&mut self, fd: RawFd, token: u64);
    fn wait(&mut self, out: &mut Vec<(u64, Ready)>, timeout_ms: i32) -> usize;
}
```

台詞:「底層是 epoll,`epoll_event.data` 那個 u64 就是我的 token,需要的話可以展開。」
三行 + 一句話,剩下的時間花在他們真正在評的東西上。**現場手搓 FFI 是負分動作**——
時間燒在沒人評分的地方(2026-07-16 JD 攻略分析定案)。

## 太大的題目:定檔程序(一顆用寫的,其餘用講的)

上一節是 epoll 的特例;通則是:**任何 45 分鐘寫不完的題目,都在考定界,
不在考手速**。escalation ladder(`rehearsals/README.md`)給了 runtime 題的
階梯;這裡是可重複的三步程序,任何大題都跑它:

1. **找心臟**。判準三條,同時成立才是心臟:
   (a) 題目其他部分都是它的**變奏**(寫出它,其餘用講的都有掛靠點);
   (b) 它是**被評分的核心**(不是 plumbing);
   (c) 25–30 分鐘寫得完。
2. **宣言簽約**(0–5 分鐘內):「45 分鐘我寫 X,Y 和 Z 我用 stub + 講架構,
   時間剩我補 Y。」——面試官要嘛點頭要嘛當場改範圍,兩種都比悶頭寫贏。
3. **其餘定檔**:每樣東西標「寫 / 用 / 講 / stub」,講的部分各配一句台詞。

### 實例一:「build a runtime」(六樣定檔)

| 東西 | 檔次 | 一句話 |
|---|---|---|
| **block_on + Waker + Delay** | ✍️ 寫(25–30 分) | Waker 協議是心臟;Delay 的計時 thread 就是微型 reactor;park **token 語意**必口述 |
| async fn | 用 + 講 | 狀態機是編譯器生的,沒人手寫;講 async fn → 狀態機 → Pin 那條鏈 |
| spawn / 多 task | 講(~3 分) | run queue + `Arc<Task>` + scheduled bit;「run queue 是唯一敢 unbounded 的地方:wake 不可阻塞不可丟,量被 scheduled bit 鎖在 #tasks」 |
| reactor | 講 + `Poller` stub | interest table:token → waker;std 沒有多路等待原語,這層是 tokio(mio→epoll)接手 |
| thread pool | 一句話 | 「骨架換 payload:worker 迴圈從 `job()` 換成 `task.poll()`」 |
| work-stealing / io_uring | 收尾 2 分 | 轉折點各一句 |

為什麼心臟是 block_on:spawn = 很多個 parked 的 block_on 排進 run queue;
reactor = Delay 的計時 thread 泛化成 epoll——**其餘全是它的變奏**(判準 a)。
可執行對照:`reference/src/runtime/mini_runtime.rs`(V0 O(n) scan → V1 epoll,
runtime 一行不改)。

### 實例二:「build a server」(規模決定心臟)

先問規模,答案直接改變哪顆用寫:

- **百級連線** → thread-per-connection 合法(百 × 2 MiB 付得起):
  `TcpListener` accept 迴圈 + 每連線 spawn = **薄殼 20 行,直接寫**,
  心臟是 framer / 每連線狀態機。
- **千級以上** → event loop 出場,但出場方式是上一節的 `Poller` stub
  ——**跟 runtime 題的 reactor 是同一個 stub,一套三行走天下**。

注意彩排題的 API 都刻意切在 IO 邊界上(c 題是 `feed(&[u8])`,e2 是查表,
f 是 `record/stats`)——真題多半也這樣切;沒切的話,用宣言自己切。

### Stub 句庫(嘴巴的 snippet)

- 定界:*"In 45 minutes I'll write the framer and per-connection state;
  the IO loop I'll stub behind a `Poller` trait and describe."*
- Runtime 定界:*"I'll build single-threaded `block_on` plus a `Delay`
  future first — the Waker protocol is the heart of the runtime.
  Spawn and the IO reactor I'll describe; if time remains I add spawn."*
- Stub 收場:*"Underneath this is epoll — the u64 in `epoll_event.data`
  is my token. Happy to expand it, but the graded part is the framer,
  so let me build that."*

## 一句話總結

CoderPad = 單檔 + 固定 crate 清單(std 基本盤 + tokio,無 libc / mio)+
新 toolchain(1.92 / 2024 Edition)+ 有 Run 按鈕(一次 ~7 秒)。
能考的是:鎖/條件變數、atomic、執行緒生命週期、ring/framing/index-based
資料結構,以及 tokio 層級的 async 題;raw syscall 實測連得上、epoll 技術上
可行,但時限內手搓不划算——維持深讀定位,場上以 `Poller` stub 三行帶過。
