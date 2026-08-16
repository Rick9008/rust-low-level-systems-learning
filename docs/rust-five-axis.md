# Rust 的五個核心面向：Ownership、XOR、Send/Sync、Ordering 與 Pin

這篇用五個彼此獨立的面向，整理 Rust 在「擁有資料、修改資料，以及跨執行緒共享資料」時會遇到的問題。

先記住一個原則：**不同問題需要不同工具。** `Arc` 可以處理共享所有權，卻不能讓 `RefCell` 變成執行緒安全；`Mutex` 可以保護並行修改，卻不能解決自引用結構在移動後失效的問題。遇到錯誤時，先判斷它屬於哪一個面向，再選工具。

如果你是 Rust 初學者，建議先讀第 1、2 軸。第 3、4 軸與多執行緒有關，第 5 軸則通常要等到接觸 async 或底層資料結構時才會用到。

## 五軸總覽

| 面向 | 要回答的問題 | 常見工具 | 通常由誰發現錯誤 |
|---|---|---|---|
| 1. Ownership（所有權） | 誰擁有資料？引用是否比資料活得更久？ | `Box`、`Rc`、`Arc`、`Weak` | 編譯器 |
| 2. XOR（借用規則） | 同一份資料可不可以同時被讀取或修改？ | `Cell`、`RefCell`、`Mutex`、arena | 編譯器或執行期檢查 |
| 3. `Send` / `Sync` | 值能否移到其他執行緒？引用能否跨執行緒共享？ | auto trait、`Arc`、`unsafe impl` | 編譯器；錯誤的 `unsafe impl` 除外 |
| 4. Ordering（記憶體順序） | 一個執行緒的寫入，何時能被另一個執行緒看見？ | `Relaxed`、`Acquire`、`Release`、`SeqCst` | 通常不會自動被發現 |
| 5. `Pin`（固定位置） | 物件被移動後，內部指標或引用是否會失效？ | `Pin`、`PhantomPinned` | 編譯器透過 `Pin` API 限制操作 |

前四軸最容易混淆的地方是：**「能共享所有權」不等於「能安全修改」，「能跨執行緒」也不等於「執行緒之間已正確同步」。**

## 1. Ownership：誰擁有資料

Rust 的每個值都有擁有者。擁有者離開作用域時，值就會被釋放。借用 `&T` 或 `&mut T` 不會取得所有權，因此引用不能比原本的值活得更久。

常見智慧指標的用途如下：

- `Box<T>`：一個擁有者，資料放在 heap 上。
- `Rc<T>`：同一執行緒內可以有多個擁有者。
- `Arc<T>`：多個執行緒可以共同持有所有權。
- `Weak<T>`：不增加強引用計數，常用來避免循環引用。

`Arc<T>` 只處理「誰擁有資料」。它不會自動讓 `T` 的內部修改變安全。例如 `Arc<RefCell<T>>` 仍然不能安全地跨執行緒共享。

## 2. XOR：共享與可變只能擇一

Rust 的借用規則常被簡寫成 XOR（exclusive or）：在同一段時間內，你可以擁有：

- 任意數量的不可變引用 `&T`；或
- 一個可變引用 `&mut T`。

兩種情況不能同時存在。這條規則避免「一邊讀、一邊改」造成引用失效或資料競爭。

有時候我們確實需要透過 `&T` 修改內部狀態，這稱為 **interior mutability（內部可變性）**。不同型別只是把檢查責任放在不同位置：

| 型別 | 如何保證安全 | `Sync`？ | 適合情境 |
|---|---|---|---|
| `Cell<T>` | 不借出內部引用，以整個值進行 `get` / `set` | 否 | 單執行緒、小型且常為 `Copy` 的值 |
| `RefCell<T>` | 執行期檢查借用規則；違規時 panic | 否 | 單執行緒中的共享可變狀態 |
| `Mutex<T>` | 同一時間只讓一個執行緒取得資料 | 是，條件是 `T: Send` | 跨執行緒修改資料的常見預設選擇 |
| `RwLock<T>` | 允許多個讀者，或一個寫者 | 是，條件是 `T: Send + Sync` | 讀取遠多於寫入的情境 |
| `UnsafeCell<T>` | 不提供高階安全檢查，由實作者自行維護規則 | 否 | 實作鎖、lock-free 結構等底層元件 |

`UnsafeCell<T>` 是 Rust 內部可變性的底層原語，`Cell`、`RefCell` 與許多同步型別都建立在它之上。直接使用它不代表程式自然就是安全的，而是表示安全性必須由實作者證明。

## 3. `Send` 與 `Sync`：能否跨執行緒

`Send` 與 `Sync` 是兩個 marker trait。大多數情況下，編譯器會根據型別的欄位自動判斷，不需要手動實作。

- `T: Send`：`T` 的所有權可以安全地移到另一個執行緒。
- `T: Sync`：多個執行緒可以安全地同時持有 `&T`。

兩者有一個重要關係：

**`T: Sync` 等價於 `&T: Send`。**

直覺上，如果 `T` 的共享引用可以送到另一個執行緒，那麼多個執行緒就能同時透過 `&T` 存取它，因此 `T` 必須是 `Sync`。

### 常見型別的推導

表格中的「有條件」表示結果取決於內部型別 `T`。

| 型別 | `Send` | `Sync` | 原因 |
|---|---:|---:|---|
| `Rc<T>` | 否 | 否 | 引用計數不是 atomic，不能由多個執行緒共同操作 |
| `Arc<T>` | 有條件 | 有條件 | 通常需要 `T: Send + Sync`；`Arc` 只讓所有權可共享 |
| `Cell<T>` | 有條件 | 否 | 可以把整個值移交給其他執行緒，但不能被多執行緒同時共享 |
| `RefCell<T>` | 有條件 | 否 | 借用計數不是執行緒安全的 |
| `Mutex<T>` | `T: Send` | `T: Send` | 鎖讓同一時間只有一個執行緒能存取內部資料 |
| `RwLock<T>` | `T: Send` | `T: Send + Sync` | 多個 read guard 可能同時提供 `&T` |
| `MutexGuard<'_, T>` | 否 | `T: Sync` | 標準函式庫要求 guard 在上鎖的執行緒中解鎖 |
| `*const T` / `*mut T` | 否 | 否 | raw pointer 沒有自動的執行緒安全保證 |
| `&T` | `T: Sync` | `T: Sync` | 共享引用能否跨執行緒取決於 `T: Sync` |
| `&mut T` | `T: Send` | `T: Sync` | 獨占引用的移交與共享分別受 `Send`、`Sync` 約束 |

一個值得比較的例子是 `Cell<i32>`。它是 `Send`，但不是 `Sync`：

- `Mutex<Cell<i32>>` 可以是 `Sync`，因為鎖確保同一時間只有一個執行緒操作這個值。
- `RwLock<Cell<i32>>` 不是 `Sync`，因為多個讀者可能同時取得 `&Cell<i32>`，而 `Cell<i32>` 本身不能被安全共享。

這也說明了「移交」與「共享」的差別。`Mutex<T>` 保護的是一次只把存取權交給一個執行緒，所以只要求 `T: Send`；它不要求內部的 `T` 本身能被多執行緒同時共享。

### 如何說明一個 `unsafe impl`

手動實作 `Send` 或 `Sync` 時，編譯器會相信你的承諾，因此必須能清楚回答三件事：

1. 每個執行緒會讀寫哪些欄位或記憶體位置？
2. 哪個不變量保證這些操作不會互相衝突？
3. 泛型界限為什麼足夠，例如為什麼只需要 `T: Send`？

以 SPSC（single-producer, single-consumer）ring buffer 為例：producer 只發布新資料，consumer 只取走已發布的資料；兩者透過 `head` 與 `tail` 協調可用的 slot。值是從 producer **移交**給 consumer，而不是同時以 `&T` 共享，因此實作通常需要的是 `T: Send`。但這個結論只有在 slot 不重疊與記憶體排序都正確時才成立。

repo 中可對照 [SPSC ring 實作](../reference/src/concurrency/spsc_ring/core_impl.rs)與 [async 同步實作](../reference/src/runtime/async_sync.rs)。

## 4. Ordering：執行緒之間何時看得到資料

atomic 操作除了保證單次讀寫不會被撕裂，還需要指定 memory ordering。Ordering 回答的不是「誰擁有資料」，而是「其他執行緒何時保證看得到先前的寫入」。

常見順序如下：

- `Relaxed`：只保證該 atomic 操作本身的原子性，不建立其他資料的同步關係。
- `Release`：發布這個操作之前的寫入。
- `Acquire`：取得與它配對的 `Release` 所發布的寫入。
- `AcqRel`：同時具有 `Acquire` 與 `Release` 的效果，常用於 read-modify-write 操作。
- `SeqCst`：提供最容易推理的全域順序，但限制也最強。

典型的發布流程是：

```rust
// producer
write_data();
ready.store(true, Ordering::Release);

// consumer
if ready.load(Ordering::Acquire) {
    read_data();
}
```

當 consumer 的 `Acquire` load 讀到 producer 以 `Release` store 寫入的值時，兩者建立 `synchronizes-with` 關係。producer 在 `Release` 之前的寫入，會 happens-before consumer 在 `Acquire` 之後的讀取。

如果兩邊都使用 `Relaxed`，atomic 旗標本身仍然安全，但它不能保證普通資料已經對 consumer 可見。這類錯誤通常可以編譯，也可能長時間通過一般測試，因此需要格外謹慎。repo 使用 [loom](https://github.com/tokio-rs/loom) 測試可能的執行緒交錯，相關範例可看 [SPSC ring](../reference/src/concurrency/spsc_ring/core_impl.rs)與 [signal pipeline](../reference/src/concurrency/signal_pipeline.rs)。

初學階段不必急著把所有操作放寬成最低成本的 ordering。先使用容易證明正確的做法，再針對每次放寬回答：「哪個 `Release` 與哪個 `Acquire` 配對？它保護了哪些資料？」

## 5. `Pin`：固定物件的位置

一般 Rust 值可以被移動。移動會改變值所在的記憶體位址，多數型別不受影響；但如果物件內部保存了指向自己的指標，移動後該指標就可能失效。

`Pin<P>` 的用途是透過 API 保證：當被包住的值屬於 `!Unpin` 時，不會再以安全方式把它移出目前位置。`PhantomPinned` 可用來讓自訂型別成為 `!Unpin`。

`Pin` 常出現在 async，是因為編譯器產生的某些 future 可能在內部保存跨越 `.await` 的自引用。這也是 `Future::poll` 的接收者為 `Pin<&mut Self>` 的原因。

初學時先記住：

- 大多數一般型別都是 `Unpin`，使用 `Pin` 時沒有額外限制。
- `Pin` 主要保護的是「移動會造成問題」的 `!Unpin` 型別。
- `Pin` 不負責執行緒安全，也不取代 `Mutex` 或 atomic ordering。

repo 的 executor 練習會接觸到 `Pin`，可從 [executor drill](../drills/src/runtime/executor.rs)開始看。

## Edition 2024 的兩個注意事項

- `gen` 在 Rust 2024 edition 中是保留關鍵字。變數可改名為 `generation` 或 `generations`。
- `if let` 等暫時值的 drop scope 在 edition 2024 有調整。持有 `MutexGuard` 時尤其要留意鎖實際在哪一行被釋放，範例見 [thread-safe spectrum](concurrency/thread-safe-spectrum.md)。

## 在這個 repo 中如何練習

建議依照以下順序，把概念和程式碼接起來：

1. Ownership 與 XOR：先做 `dsu`、`tree`、`lru`，熟悉借用與資料結構拆分。
2. `Send` / `Sync`：閱讀 `async_sync` 與 `spsc_ring`，觀察型別界限如何推導。
3. Ordering：對照 `spsc_ring` 和 `signal_pipeline` 的 atomic 操作與測試。
4. `Pin`：最後閱讀 `executor`，理解 `Future::poll` 為何需要 `Pin<&mut Self>`。

需要圖解與互動推演時，可開啟 [Rust 五軸互動版](../html_p/rust-five-axis.html)。互動版保留較多進階內容；第一次學習時，不需要一次讀完。
