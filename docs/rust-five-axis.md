# Rust 五軸:Ownership × XOR × Send/Sync × Ordering × Pin

來源:`html_p/rust-five-axis.html`(互動版,推導表逐格經 rustc 實編驗證)。
這份是濃縮 + repo 對映,7/25 口述底稿。核心一句:**每個「共享」問題,先問
它卡在哪一軸——答錯軸就拿錯工具**(拿 Arc 解 XOR、拿 Mutex 解自引用,都是這種錯)。

## 五軸與攔截網(這張表是整份的重點)

| 軸 | 回答的問題 | 工具 | 誰抓你的錯 |
|---|---|---|---|
| 1 Ownership | 誰擁有?引用會 outlive 嗎? | Box/Rc/Arc/Weak | 編譯器 |
| 2 XOR(aliasing) | 要透過共享路徑改東西 | Cell/RefCell/Mutex/arena | 編譯器或執行期(panic/block)——會響 |
| 3 Send/Sync | 能不能換 thread / 被多 thread 同時 `&`? | auto trait、`unsafe impl` | 編譯器——**除非你 unsafe impl 說謊** |
| 4 Ordering | 兩條 thread 對記憶體的看法怎麼同步 | Relaxed/Acq/Rel/SeqCst | **沒有人**。編譯器不管、x86 測不出(TSO)→ 一年壞一次。這是 loom 存在的理由 |
| 5 Pin | struct 裡有指向自己的指標 | Pin/PhantomPinned | 編譯器(Pin API 收掉 `&mut`) |

軸 1/2/3/5 = 語法題(寫錯當場編不過);軸 4 = 判斷題(唯一沒有網的軸)。
**7/28 真的開火的是兩軸:Send/Sync 的辯護、ordering 的配對**——而且考法是
用英文講出來,不是看懂。

## 恆等式(吃下它,推導表不用背)

**`T: Sync ⟺ &T: Send`**——「T 可以被多 thread 同時借」和「T 的引用可以送到
別條 thread」是同一句話,這就是 Sync 的定義(std 有 blanket impl:
`unsafe impl<T: Sync + ?Sized> Send for &T {}`)。

## 推導表(逐格 rustc 實編驗證過)

| 型別 | Send | Sync | 為什麼(被問的是這欄) |
|---|---|---|---|
| `Rc<T>` | ✗ | ✗ | refcount 非 atomic;連 move 都不行——別條 thread 可能還握著同一個 Rc |
| `Arc<T>` | ✓* | ✓* | *要 `T: Send + Sync`。`Arc<RefCell<_>>` 照樣 !Sync——Arc 只是把 `&T` 發給大家 |
| `Cell<T>` | ✓ | ✗ | move 過去仍單一擁有者 ✓;`set(&self)` 兩 thread 同時 = data race 且無檢查 |
| `RefCell<T>` | ✓ | ✗ | borrow flag 是普通 Cell 不是 atomic——並行下兩個 `borrow_mut()` 雙雙成功 |
| `Mutex<T>` | ✓ | ✓ | **只要 `T: Send`,不需要 `T: Sync`**——鎖保證同時只有一人拿 `&mut T`,T 從沒被共享,只是被**移交** |
| `RwLock<T>` | ✓ | ✓ | 要 `T: Send + Sync`——read guard 真的同時發多個 `&T` 出去 |
| `MutexGuard<'_, T>` | ✗ | ✓† | 唯一常見的 Send✗/Sync✓:pthread 要求解鎖 = 上鎖那條 thread,guard 換 thread drop = UB;†Sync 要 `T: Sync` |
| `*const T` / `*mut T` | ✗ | ✗ | raw pointer 一律否——`UnsafeCell` 把 !Sync **傳染**給整個 struct(你的 ring 就是這樣死的,然後才有 unsafe impl) |
| `&T` | =T: Sync | =T: Sync | 恆等式本身 |
| `&mut T` | =T: Send | =T: Sync | 獨佔——送過去等於把 T 送過去 |

**判別對(最值得背的一格)**:同一個 `T = Cell<i32>`(Send 但 !Sync)——
`Mutex<Cell<i32>>: Sync` ✓ 編過、`RwLock<Cell<i32>>: Sync` ✗ E0277。
兩把鎖的語意差異壓成一行編譯結果。記反 Mutex 的 bound,你會把自己 ring 的
`unsafe impl` 過度約束成 `T: Send + Sync`,被問「為什麼要 Sync?」整段辯護垮掉。

## 7/28 必答題:辯護你的 unsafe impl(三段式模板)

repo 裡兩個活例子:`spsc_ring/core_impl.rs:40` 與 `async_sync.rs` 的
`unsafe impl<T: Send> Send/Sync`——bound 是 `T: Send` 不是 `T: Send + Sync`,
這個 bound 本身就是考點。照模板講,不要即興:

1. **誰碰什麼(指出存取不重疊)**:「producer 只寫 tail 和 tail 指的那格;
   consumer 只寫 head 和 head 指的那格——slot 集合永遠不相交。」
2. **誰保證不重疊**:「由 head/tail 的 acquire/release 配對保證——consumer
   看到新 tail 時,那格的資料寫 happens-before 它的讀。」
3. **為什麼 bound 是 T: Send**:「值是被**移交**(一次一格、單向),從沒被
   兩條 thread 同時 `&`——跟 `Mutex<T>: Sync 只要 T: Send` 同一個理由。」

## Interior mutability 階梯(XOR 檢查搬到哪)

| 型別 | 檢查搬到哪 | Sync? | 何時用 |
|---|---|---|---|
| `Cell<T>` | 不檢查(不交引用,整值 get/set) | ✗ | 單執行緒小 Copy 欄位 |
| `RefCell<T>` | 執行期 borrow flag(違規 panic) | ✗ | 單執行緒共享可變、原型 |
| `Mutex<T>` | 執行期 OS 鎖(違規 block) | ✓(T: Send) | 跨執行緒預設答案 |
| `RwLock<T>` | 同上,分讀寫 | ✓(T: Send+Sync) | 讀遠多於寫;要講得出 starvation |
| `UnsafeCell<T>` | **搬到你腦子裡** | ✗(自己 unsafe impl) | ring / lock-free——唯一真零成本 |

面試句:「UnsafeCell 是所有 interior mutability 的底層原語——Cell/RefCell/Mutex
內部全是它。我在 ring 直接用它,因為 XOR 已由 head/tail 的 acquire/release 保證,
再套 RefCell 是重複檢查,而且 RefCell 還 !Sync。」

## 兩個 edition 2024 陷阱(pad 實戰)

- **`gen` 是保留字**(RFC 3513):手寫 generational index 時 `let gen = ...`
  直接編不過——命名用 `generation` / `gens`(repo 的 `fd_registry` 已避開)。
- temporary drop scope 變更:見 [thread-safe-spectrum](concurrency/thread-safe-spectrum.md)
  的 `if let` + Mutex 地雷。

## repo 交叉對映

軸 2 的 arena+index 繞道 → `lru`/`tree`/`dsu` + `docs/io/fd_registry.md`;
軸 3 辯護 → `spsc_ring`、`async_sync`;軸 4 → `spsc_ring` 的 loom、
`signal_pipeline` 的 SB litmus(掛牌握手是「唯一沒有網的軸」的實戰位);
軸 5 → `executor`(pin! 兩行,別多花時間)。
