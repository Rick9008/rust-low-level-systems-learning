# 從零開始學 Rust：Ownership 與借用

這份教材假設你從未寫過 Rust。請依照章節順序閱讀，因為每一節只使用前面已經介紹過的概念。

目前先不要打開 DSU、Tree 或 LRU。我們要先學會讀懂變數、函式、Ownership、Move 與 Borrow。

> 使用方式：每個章節預設可以收合。點擊章節標題即可展開或隱藏內容。若你的 Markdown 預覽器不支援 `<details>`，內容仍會依序顯示，不影響閱讀。

## A. Cargo、編譯、執行與測試

這一區只解釋工具、專案結構與終端指令，不講 Rust 語言規則。

<details open>
<summary><strong>A1. Rust、rustc、Cargo 和 rustup</strong></summary>


| 名稱 | 用途 | 可以先想成 |
|---|---|---|
| Rust | 程式語言 | 我們要學的語言 |
| `rustc` | Rust 編譯器 | 把 `.rs` 轉成可執行程式 |
| Cargo | 專案工具 | 建置、執行與測試 Rust 專案 |
| rustup | 工具鏈管理器 | 安裝和更新 `rustc`、Cargo |

專案中通常透過 Cargo 工作：

```powershell
cargo run
cargo test
cargo check
```

- `cargo run`：編譯並執行程式。
- `cargo test`：編譯並執行測試。
- `cargo check`：檢查程式能否編譯，通常比完整建置快。

</details>

<details>
<summary><strong>A2. 練習資料夾的結構</strong></summary>


```text
ownership_xor_lab/
├── Cargo.toml
├── README.md
├── START_HERE.md
├── examples/
│   ├── hello.rs
│   └── ownership_demo.rs
└── src/
    ├── lib.rs
    ├── ownership_xor.rs
    ├── dsu.rs
    ├── tree.rs
    └── lru.rs
```

- `Cargo.toml`：專案名稱、Rust edition 等設定。
- `examples/`：可以直接執行的完整小程式。
- `src/lib.rs`：宣告這個 library 包含哪些模組。
- `src/*.rs`：稍後要完成的練習。

Rust 程式檔的副檔名是 `.rs`。

</details>

<details>
<summary><strong>A3. 編譯並執行第一個 example</strong></summary>


開啟 [`examples/hello.rs`](examples/hello.rs)：

```rust
fn main() {
    println!("Hello, Rust!");
}
```

這段 Rust 程式的逐行語法說明放在 B2.1；本節只處理如何透過 Cargo 編譯和執行它。

執行：

```powershell
cd D:\rust_pratice\ownership_xor_lab
cargo run --example hello
```

這裡的 `hello` 不是檔案路徑，而是 **example target 的名稱**。Cargo 會依照固定的專案目錄慣例尋找檔案：

```text
指令中的 target 名稱       Cargo 尋找的檔案
hello                  ->  examples/hello.rs
ownership_demo         ->  examples/ownership_demo.rs
```

Cargo 使用檔名去掉 `.rs` 後的部分作為 target 名稱：

```text
examples/hello.rs
         └───┬──┘
           hello       <- target 名稱
              .rs      <- Rust 原始碼副檔名，不寫在指令中
```

所以指令要寫：

```powershell
cargo run --example hello
```

而不是：

```powershell
# 錯誤示意：--example 後面不是放檔案路徑
cargo run --example examples/hello.rs
```

可以把整條指令拆開理解：

```text
cargo run --example hello
│     │   │         │
│     │   │         └─ target 名稱是 hello
│     │   └─────────── target 種類是 example
│     └─────────────── 編譯後執行
└───────────────────── 使用 Cargo
```

Cargo 編譯後，Windows 可執行檔通常會放在：

```text
target/debug/examples/hello.exe
```

你不需要直接執行這個 `.exe`；`cargo run --example hello` 會替你完成「找來源檔、判斷是否需要重新編譯、產生執行檔、執行」整個流程。

#### 為什麼不是只寫 `cargo run`

Cargo 專案可以同時包含多種 target：

| 原始碼位置 | target 種類 | 執行方式 |
|---|---|---|
| `src/main.rs` | 預設 binary | `cargo run` |
| `src/bin/server.rs` | 名為 `server` 的 binary | `cargo run --bin server` |
| `examples/hello.rs` | 名為 `hello` 的 example | `cargo run --example hello` |
| `src/lib.rs` | library | 通常由其他 target 使用，不直接執行 |

這個練習專案主要是 library，入口是 `src/lib.rs`，並沒有 `src/main.rs`。它另外提供兩個 example，因此要用 `--example` 告訴 Cargo 要執行哪一個：

```powershell
cargo run --example hello
cargo run --example ownership_demo
```

結果應該包含：

```text
Hello, Rust!
```

Cargo 同時檢查 library 中尚未完成的練習，所以現在可能看到 `unused variable` warning。warning 不會阻止程式執行；看到最後的 `Hello, Rust!` 就表示成功。

</details>

<details>
<summary><strong>A4. 為什麼測試指令是 cargo test ownership_xor</strong></summary>

以下指令不是直接執行 `src/ownership_xor.rs`：

```powershell
cargo test ownership_xor
```

`cargo test` 後面的 `ownership_xor` 是 **測試名稱的篩選字串**，不是檔案路徑。

Cargo 與 Rust 會依照以下關係找到程式：

```text
Cargo.toml
└─ library target：src/lib.rs
   └─ pub mod ownership_xor;
      └─ 載入 src/ownership_xor.rs
         └─ 找到其中的 #[test]
```

測試的完整名稱會包含 module 路徑，例如：

```text
ownership_xor::tests::ownership_xor_consume_and_sum
ownership_xor::tests::ownership_xor_append_through_mutable_borrow
ownership_xor::tests::ownership_xor_largest_returns_a_borrow
ownership_xor::tests::ownership_xor_swap_two_positions
```

`cargo test ownership_xor` 只執行名稱包含 `ownership_xor` 的測試。

常用指令：

```powershell
# 列出全部測試名稱
cargo test -- --list

# 執行 Ownership/XOR 相關測試
cargo test ownership_xor

# 只執行 largest 測試
cargo test ownership_xor_largest_returns_a_borrow

# 執行專案全部測試
cargo test
```

如果目前位於 `D:\rust_pratice`，而不是專案資料夾，使用：

```powershell
cargo test --manifest-path .\ownership_xor_lab\Cargo.toml ownership_xor
```

`src/ownership_xor.rs` 沒有 `fn main()`，它是 library module，不是可以用 `cargo run` 直接啟動的 binary。

</details>

<details>
<summary><strong>A5. 格式、檢查與測試指令</strong></summary>

Cargo 可以呼叫不同工具檢查專案：

```powershell
# 只檢查格式，不修改檔案
cargo fmt -- --check

# 自動修改整個專案的 Rust 格式
cargo fmt

# 快速檢查能否編譯
cargo check

# 執行 Ownership/XOR 測試
cargo test ownership_xor

# 執行全部測試
cargo test
```

`cargo fmt -- --check` 和 `cargo fmt` 的差異：

| 指令 | 會修改檔案嗎？ | 用途 |
|---|---|---|
| `cargo fmt -- --check` | 不會 | 顯示哪些地方不符合 rustfmt |
| `cargo fmt` | 會 | 自動套用標準 Rust 排版 |

Rust 常見空格格式：

```rust
if numbers.is_empty() {
    // `{` 前有一個空格
}

for number in numbers {
    // `{` 前有一個空格
}

numbers.swap(a, b);
//              ^ 逗號後有一個空格
```

格式不影響這些程式的行為，但一致的格式能讓 review 更容易。初學時可以先自己修改，再執行 `cargo fmt -- --check` 確認；熟悉後再使用 `cargo fmt` 自動整理。

</details>

## B. Rust 語法與底層邏輯

這一區只解釋語言本身：型別、函式、記憶體、Ownership、Borrow、迴圈、Slice 與泛型。

<details>
<summary><strong>B1. 變數、mut 與整數型別</strong></summary>


### B1.1 使用 `let` 建立變數

```rust
let number = 10;
```

`let` 建立變數。Rust 變數預設不能修改：

```rust,compile_fail
let number = 10;
number = 20; // 錯誤
```

要修改變數，必須加上 `mut`：

```rust
let mut number = 10;
number = 20;
```

`mut` 是 mutable 的縮寫，表示這個變數之後允許被修改。

### B1.2 `let number = 10` 中的 `10` 是什麼型別

`10` 是 integer literal（整數常值）。它剛出現時還沒有固定型別，Rust 會根據周圍程式碼推導。

如果沒有其他線索，整數預設為 `i32`：

```rust
let number = 10; // i32
```

`i32` 的意思是：

- `i`：signed integer，可以保存負數。
- `32`：使用 32 bits。

範圍是 `-2_147_483_648` 到 `2_147_483_647`。數字中的 `_` 只方便閱讀。

可以明確指定其他型別：

```rust
let small: u8 = 10;
let temperature: i64 = 10;
let index: usize = 10;
```

- `u8`：8-bit unsigned integer，範圍 `0..=255`。
- `i64`：64-bit signed integer。
- `usize`：常用於記憶體大小、容器長度與索引。

冒號後面是 type annotation（型別註記）：

```text
let 變數名稱: 型別 = 值;
```

也可以把型別 suffix 寫在數字後面：

```rust
let small = 10u8;
let temperature = 10i64;
let index = 10usize;
```

Rust 也能從用途推導型別：

```rust
fn accept_u64(value: u64) {
    println!("{value}");
}

let number = 10;
accept_u64(number); // number 被推導為 u64
```

不同整數型別不會任意自動轉換：

```rust,compile_fail
let small: u8 = 10;
let large: u64 = small; // u8 和 u64 是不同型別
```

需要時要明確轉換：

```rust
let small: u8 = 10;
let large: u64 = u64::from(small);
```

判斷順序是：

1. 是否有 `: u8` 或 `10u8` 等明確型別？
2. 後續用途是否提供型別線索？
3. 都沒有時，整數預設為 `i32`。

</details>

<details>
<summary><strong>B2. 函式、參數與回傳值</strong></summary>


這一節只處理函式語法，暫時不談 Ownership 或引用。

### B2.1 `fn main()` 與 `println!`

```rust
fn main() {
    println!("Hello, Rust!");
}
```

逐行解釋：

- `fn` 表示定義函式。
- `main` 是 binary 或 example 開始執行的位置。
- `()` 表示這個函式沒有參數。
- `{ ... }` 是函式內容。
- `println!` 在終端印出一行文字。
- 名稱後面的 `!` 表示 `println!` 是 macro。
- Rust 的敘述通常以分號 `;` 結尾。

接著看有參數和回傳值的函式：

```rust
fn add(left: i32, right: i32) -> i32 {
    left + right
}
```

- `add`：函式名稱。
- `left: i32`：名稱為 `left`、型別為 `i32` 的參數。
- `right: i32`：第二個參數。
- `-> i32`：函式回傳 `i32`。
- 最後的 `left + right` 沒有分號，因此是回傳值。

### B2.2 Parameter 和 argument

```rust
fn print_number(number: i32) { // number 是 parameter
    println!("{number}");
}

fn main() {
    print_number(10); // 10 是 argument
}
```

- parameter（參數）：函式定義裡的名稱。
- argument（引數）：呼叫函式時傳入的實際值。

日常對話常混用這兩個詞，但閱讀文件時知道差異會比較清楚。

### B2.3 分號與回傳值

以下程式無法編譯：

```rust,compile_fail
fn add(left: i32, right: i32) -> i32 {
    left + right;
}
```

加上分號後，`left + right;` 成為普通敘述，函式沒有回傳宣告要求的 `i32`。

初學時先記住：**函式最後一個沒有分號的 expression，就是回傳值。**

### B2.4 Rust 常見的 `return` 寫法

Rust 常見的 coding style 是：

- 函式正常執行到最後：使用最後一個 expression 當回傳值，通常不寫 `return`。
- 特殊情況需要提前離開：明確寫 `return value;`。

正常走到最後的簡單函式：

```rust
fn add(left: i32, right: i32) -> i32 {
    left + right
}
```

也可以明確寫 `return`：

```rust
fn add(left: i32, right: i32) -> i32 {
    return left + right;
}
```

兩種都合法，但第一種更符合常見 Rust 風格。

需要提前回傳時，通常使用 guard clause（先處理特殊情況）：

```rust
fn at_least(value: i32, minimum: i32) -> i32 {
    if value < minimum {
        return minimum;
    }

    value
}
```

執行流程：

```text
value < minimum？
├─ 是 -> return minimum; 立即結束函式
└─ 否 -> 繼續往下，最後回傳 value
```

`return` 代表「現在就結束整個函式」，所以後面的程式不會繼續執行。

如果只是把一個值寫在 `if` 裡，卻沒有 `return`，函式仍會繼續：

```rust
fn incorrect_at_least(value: i32, minimum: i32) -> i32 {
    if value < minimum {
        minimum; // 有分號，結果被丟棄，也沒有離開函式
    }

    value
}
```

`incorrect_at_least(3, 10)` 最後仍會回傳 `3`，因為 `minimum;` 沒有讓函式停止。

也可以讓整個 `if/else` 成為最後一個 expression：

```rust
fn at_least(value: i32, minimum: i32) -> i32 {
    if value < minimum {
        minimum
    } else {
        value
    }
}
```

這裡兩個分支最後的 `minimum`、`value` 都沒有分號，因此 `if/else` 會產生一個 `i32`，再成為函式回傳值。

兩種常見形式的對照：

```rust
// 形式一：early return，先排除特殊情況
fn example(value: i32) -> i32 {
    if value < 0 {
        return 0;
    }

    value
}

// 形式二：整個 if/else 是最後的回傳 expression
fn example_with_if(value: i32) -> i32 {
    if value < 0 {
        0
    } else {
        value
    }
}
```

選擇原則：

| 情況 | 常見寫法 |
|---|---|
| 最終正常結果 | 最後一行 expression，不寫 `return`、不加分號 |
| 錯誤、空資料或特殊情況 | 使用 `return ...;` 提前離開 |
| 簡短的二選一結果 | 讓 `if/else` 本身成為 expression |

可以記成這個模板：

```text
fn function(value: i32) -> i32 {
    if 特殊條件 {
        return 提前回傳值;
    }

    // 正常處理

    最終回傳值
}
```

上面的 `特殊條件` 和 `提前回傳值` 是說明用佔位文字，不是可以直接編譯的 Rust 程式。

</details>

<details>
<summary><strong>B3. String、Vec、stack 與 heap</strong></summary>


在學 Ownership 前，需要先知道有些值會管理動態配置的記憶體。

### B3.1 `String`

```rust
let message = String::from("hello");
```

`String` 是可以增長、修改並擁有文字內容的型別。

概念上，區域變數保存管理資料，文字 bytes 放在 heap：

```text
stack                                  heap
┌────────────────────────┐            位址 0x5000
│ message: String        │            ┌───┬───┬───┬───┬───┐
│ ptr = 0x5000 ──────────┼───────────>│ h │ e │ l │ l │ o │
│ len = 5                │            └───┴───┴───┴───┴───┘
│ capacity = 5           │
└────────────────────────┘
```

- pointer：heap 資料的位址。
- length：目前使用多少 bytes。
- capacity：目前配置多少空間。

這是理解用的簡化圖。實際欄位順序與編譯器最佳化不是這裡的重點。

### B3.2 `Vec<i32>`

```rust
let mut numbers: Vec<i32> = vec![10, 20, 30];
numbers.push(40);
```

- `Vec<i32>`：保存 `i32` 的可變長度陣列。
- `vec![...]`：建立 vector 的 macro。
- `push(40)`：在尾端加入元素。
- `push` 會修改 vector，所以變數需要 `mut`。

概念圖與 `String` 類似：

```text
stack                              heap
┌─────────────────────┐           ┌────┬────┬────┬────┐
│ numbers: Vec<i32>   │──────────>│ 10 │ 20 │ 30 │ 40 │
│ ptr / len / capacity│           └────┴────┴────┴────┘
└─────────────────────┘
```

### B3.3 使用 `[]` 隨機存取元素

`Vec` 支援像 C/C++ array 或 `std::vector` 一樣使用 `[]` 存取指定位置，時間複雜度是 O(1)：

```rust
let numbers = vec![10, 20, 30, 40];

let first = numbers[0];
let third = numbers[2];

assert_eq!(first, 10);
assert_eq!(third, 30);
```

索引從 `0` 開始：

```text
索引          0       1       2       3
           ┌──────┬──────┬──────┬──────┐
numbers    │  10  │  20  │  30  │  40  │
           └──────┴──────┴──────┴──────┘
              ^               ^
          numbers[0]      numbers[2]
```

索引的型別通常是 `usize`：

```rust
let numbers = vec![10, 20, 30];
let index: usize = 1;

assert_eq!(numbers[index], 20);
```

`usize` 是 unsigned integer，因此不能使用負數索引。

如果 `Vec` 本身可以修改，就能用 `[]` 修改元素：

```rust
let mut numbers = vec![10, 20, 30];
numbers[1] = 99;

assert_eq!(numbers, vec![10, 99, 30]);
```

這裡的 `mut` 很重要：

```rust,compile_fail
let numbers = vec![10, 20, 30];
numbers[1] = 99; // 錯誤：numbers 沒有宣告成 mut
```

### B3.4 Rust 的 `[]` 會檢查邊界

長度為 `3` 的 `Vec` 只有索引 `0`、`1`、`2`：

```rust,should_panic
let numbers = vec![10, 20, 30];
let value = numbers[3]; // 執行時 panic：index out of bounds
```

Rust 的 `[]` 會做 bounds check（邊界檢查）。索引超出範圍時不會讀取未知記憶體，而是停止執行並 panic。

與 C/C++ 的簡化對照：

| 寫法 | 越界時的典型行為 |
|---|---|
| C array 的 `array[index]` | 不自動檢查，可能造成 undefined behavior |
| C++ `vector[index]` | `operator[]` 不做一般的範圍檢查，越界屬於錯誤行為 |
| C++ `vector.at(index)` | 檢查範圍，越界時丟出 exception |
| Rust `vector[index]` | 檢查範圍，越界時 panic |

如果索引可能來自使用者輸入或不確定的計算，Rust 通常改用 `.get(index)`，讓程式處理「有元素」或「沒有元素」，而不是 panic：

```rust
let numbers = vec![10, 20, 30];

let existing = numbers.get(1);  // 有值
let missing = numbers.get(100); // 沒有值，但不會 panic
```

`.get()` 的回傳型別是 `Option<&i32>`。這個型別需要先理解 slice 與 `Option`，因此完整用法放在 B12.2。

最後要區分「隨機存取」與「中間插入／刪除」的成本：

| 操作 | 常見時間複雜度 |
|---|---:|
| `numbers[index]` | O(1) |
| `numbers.push(value)` | 攤銷 O(1) |
| `numbers.pop()` | O(1) |
| `numbers.insert(index, value)` | O(n)，後方元素需要搬移 |
| `numbers.remove(index)` | O(n)，後方元素需要搬移 |

</details>

<details>
<summary><strong>B4. Ownership、Move 與 Drop</strong></summary>


### B4.1 一個值有一個 owner

```rust
let message = String::from("hello");
```

`message` 是這個 `String` 的 owner。Owner 離開作用域時，Rust 會自動執行 drop，釋放它管理的 heap 記憶體。

```rust
{
    let message = String::from("hello");
    println!("{message}");
} // message 在這裡 drop
```

### B4.2 指派 `String` 會 Move

```rust
let first = String::from("hello");
let second = first;
```

所有權從 `first` 移到 `second`。Heap 文字不需要深層複製：

```text
Move 前

first: String ───────────────> heap: "hello"

Move 後

first:  [已失效]
second: String ──────────────> heap: "hello"
```

因此不能再使用 `first`：

```rust,compile_fail
let first = String::from("hello");
let second = first;
println!("{first}"); // value borrowed here after move
```

Rust 讓舊變數失效，確保只有 `second` 負責釋放 heap，避免 double free。

</details>

<details>
<summary><strong>B5. Copy 與 Clone</strong></summary>


### B5.1 `i32` 會 Copy

```rust
let first = 10;
let second = first;
println!("{first}, {second}");
```

`i32` 實作了 `Copy`。`second = first` 會複製數值，因此兩個變數都能繼續使用：

```text
stack
┌─────────────────┐
│ first: i32 = 10 │
│ second: i32 = 10│
└─────────────────┘
```

`i32`、`bool`、`char` 等簡單型別通常會 Copy。

### B5.2 `String` 不會自動 Copy

`String` 管理 heap。如果自動複製管理資料，兩個變數會指向同一塊記憶體並嘗試釋放兩次。因此預設行為是 Move。

需要獨立副本時，明確呼叫 `clone()`：

```rust
let first = String::from("hello");
let second = first.clone();
println!("{first}, {second}");
```

此時 heap 上有兩份獨立文字。`clone()` 可能配置與複製資料，所以 Rust 要求明確寫出成本。

</details>

<details>
<summary><strong>B6. 函式引數與 Ownership</strong></summary>


Rust 的函式參數會接收一個值。結果是 Copy 還是 Move，取決於型別。

### B6.1 傳入 `i32` 會 Copy

```rust
fn show_number(value: i32) {
    println!("{value}");
}

let number = 10;
show_number(number);
println!("{number}"); // 仍可使用
```

```text
main stack                    show_number stack
number: i32 = 10  --Copy-->   value: i32 = 10
```

### B6.2 傳入 `String` 會 Move

```rust,compile_fail
fn take_text(text: String) {
    println!("{text}");
}

let message = String::from("hello");
take_text(message);
println!("{message}"); // 所有權已移入函式
```

```text
main stack                  take_text stack             heap
message: [已失效] --Move--> text: String ─────────────> "hello"
```

`take_text` 結束時，參數 `text` 被 drop，heap 文字也被釋放。

如果函式只需要讀取資料，拿走所有權通常不是我們想要的行為。這時要使用 borrow。

</details>

<details>
<summary><strong>B7. &String：共享、唯讀借用</strong></summary>


```rust
fn read_text(text: &String) {
    println!("{text}");
}

let message = String::from("hello");
read_text(&message);
println!("仍可使用：{message}");
```

定義和呼叫兩邊都出現 `&`：

```text
fn read_text(text: &String)
                   ^ 接收 String 的共享引用

read_text(&message)
          ^ 建立 message 的共享引用
```

所有權沒有移動：

```text
main stack                       read_text stack              heap
┌────────────────────┐           ┌──────────────────┐         ┌─────────┐
│ message: String    │<──────────│ text: &String    │         │ hello   │
│ ptr ───────────────┼───────────────────────────────────────>│         │
└────────────────────┘           │ 只能讀取         │         └─────────┘
       owner                     └──────────────────┘
                                      borrower
```

`text`：

- 不擁有 `String`；
- 不負責釋放 heap；
- 只能讀取，不能修改；
- 不能比 `message` 活得更久。

函式結束後引用消失，`message` 仍是 owner。

</details>

<details>
<summary><strong>B8. &mut String：可變、獨占借用</strong></summary>


```rust
fn append_world(text: &mut String) {
    text.push_str(" world");
}

let mut message = String::from("hello");
append_world(&mut message);
println!("{message}");
```

這裡有三個相關位置：

- `let mut message`：變數允許修改。
- `text: &mut String`：函式接收可變引用。
- `&mut message`：呼叫時建立可變引用。

借用期間：

```text
main stack                       append_world stack            heap
message: String <─────────────── text: &mut String ──修改───> "hello world"
[所有權仍在，但暫時不能使用]     [暫時取得獨占存取權]
```

函式結束後，可變借用結束，`message` 再次可以使用。

`&mut` 不只表示 mutable，也表示 exclusive access（獨占存取）。

</details>

<details>
<summary><strong>B9. XOR 借用規則</strong></summary>


同一段時間內，同一份資料只能處於以下其中一種狀態：

1. 有任意數量的共享引用，只能讀取。
2. 只有一個可變引用，可以修改。

多個讀者可以並存：

```rust
let message = String::from("hello");
let reader_one = &message;
let reader_two = &message;
println!("{reader_one}, {reader_two}");
```

```text
reader_one: &String ───┐
                      ├──> message: String ───> heap
reader_two: &String ───┘
```

但共享讀取與修改不能重疊：

```rust,compile_fail
let mut message = String::from("hello");
let reader = &message;
let writer = &mut message;

println!("{reader}");
writer.push('!');
```

也不能同時有兩個可變引用：

```rust,compile_fail
let mut message = String::from("hello");
let writer_one = &mut message;
let writer_two = &mut message;

writer_one.push('!');
writer_two.push('?');
```

如果其中一個 writer 讓 `String` 重新配置 heap，其他引用可能指向失效的舊位址。Rust 在編譯期禁止這種狀態。

記憶口訣：**多人同時唯讀，或一人獨占修改。**

</details>

<details>
<summary><strong>B10. 迴圈與 Vec 走訪</strong></summary>


Rust 主要有三種迴圈：`loop`、`while` 和 `for`。

### B10.1 `loop`：重複直到主動停止

`loop` 會一直重複執行區塊，通常搭配 `break` 停止：

```rust
let mut count = 0;

loop {
    println!("count = {count}");
    count += 1;

    if count == 3 {
        break;
    }
}
```

執行順序：

```text
count = 0 -> 印出 -> 加成 1 -> 不 break
count = 1 -> 印出 -> 加成 2 -> 不 break
count = 2 -> 印出 -> 加成 3 -> break
```

結果：

```text
count = 0
count = 1
count = 2
```

如果沒有 `break` 或其他離開方式，`loop` 會成為無限迴圈。

`break` 也可以把值帶出迴圈：

```rust
let mut number = 0;

let result = loop {
    number += 1;

    if number == 3 {
        break number * 10;
    }
};

assert_eq!(result, 30);
```

這裡 `break number * 10` 讓整個 `loop` expression 產生 `30`。

### B10.2 `while`：條件成立時重複

```rust
let mut count = 0;

while count < 3 {
    println!("count = {count}");
    count += 1;
}
```

每次進入迴圈前，Rust 都會檢查 `count < 3`。條件變成 `false` 時停止。

使用索引走訪 `Vec`：

```rust
let numbers = vec![10, 20, 30];
let mut index = 0;

while index < numbers.len() {
    println!("{}", numbers[index]);
    index += 1;
}
```

這可以運作，但只想依序走訪元素時，`for` 通常更簡潔，也不需要自己維護 `index`。

### B10.3 `for` 與 range

```rust
for number in 0..3 {
    println!("{number}");
}
```

`0..3` 是 range，從 `0` 開始，到 `3` 之前停止：

```text
0..3  -> 0, 1, 2
```

如果要包含結尾，使用 `..=`：

```rust
for number in 0..=3 {
    println!("{number}");
}
```

```text
0..=3 -> 0, 1, 2, 3
```

這和 slice 的 `1..4` 一樣，普通 `..` 不包含右側終點。

### B10.4 使用索引走訪 `Vec`

```rust
let numbers = vec![10, 20, 30];

for index in 0..numbers.len() {
    println!("numbers[{index}] = {}", numbers[index]);
}
```

執行過程：

```text
index = 0 -> numbers[0] = 10
index = 1 -> numbers[1] = 20
index = 2 -> numbers[2] = 30
```

因為 range 是 `0..numbers.len()`，不包含 `len()`，所以最後一個索引是 `len() - 1`，不會越界。

需要索引來存取其他資料時，這種寫法很有用。但只是讀取每個元素時，直接走訪通常更清楚。

### B10.5 唯讀走訪：`for value in &numbers`

```rust
let numbers = vec![10, 20, 30];

for value in &numbers {
    println!("{value}");
}

println!("走訪後仍可使用：{numbers:?}");
```

`&numbers` 共享借用整個 `Vec`，所以：

- 不會取得 `Vec` 的所有權。
- 迴圈中的 `value` 是元素的共享引用。
- 走訪期間只能讀取元素。
- 迴圈結束後仍可使用 `numbers`。

概念圖：

```text
numbers: Vec<i32> ──> [10, 20, 30]

第 1 圈 value: &i32 ──> 10
第 2 圈 value: &i32 ──> 20
第 3 圈 value: &i32 ──> 30
```

對 `Vec<i32>` 而言，每一圈中 `value` 的型別是 `&i32`。

下面兩種寫法效果相同：

```rust
for value in &numbers {
    println!("{value}");
}

for value in numbers.iter() {
    println!("{value}");
}
```

初學時優先使用較短的 `&numbers` 即可。

### B10.6 可修改走訪：`for value in &mut numbers`

```rust
let mut numbers = vec![10, 20, 30];

for value in &mut numbers {
    *value *= 2;
}

assert_eq!(numbers, vec![20, 40, 60]);
```

`&mut numbers` 可變、獨占借用整個 `Vec`。每一圈的 `value` 是 `&mut i32`：

```text
第 1 圈 value: &mut i32 ──> 10 -> 改成 20
第 2 圈 value: &mut i32 ──> 20 -> 改成 40
第 3 圈 value: &mut i32 ──> 30 -> 改成 60
```

`value` 是引用，所以要用 `*value` 取得並修改它指向的 `i32`：

```text
value       = &mut i32
*value      = 被指向的 i32
*value *= 2 = 把實際元素乘以 2
```

下面兩種寫法效果相同：

```rust
for value in &mut numbers {
    *value *= 2;
}

for value in numbers.iter_mut() {
    *value *= 2;
}
```

### B10.7 取得所有權的走訪：`for value in numbers`

```rust,compile_fail
let numbers = vec![10, 20, 30];

for value in numbers {
    println!("{value}");
}

println!("{numbers:?}"); // 錯誤：numbers 已被移入迴圈
```

這裡沒有 `&`。`for` 取得 `numbers` 的所有權，逐一取出元素；迴圈結束後，原本的 `Vec` 已被消耗，不能再使用。

```text
走訪前：numbers owns [10, 20, 30]

第 1 圈：value owns 10
第 2 圈：value owns 20
第 3 圈：value owns 30

走訪後：numbers 已被消耗
```

這也可以明確寫成：

```rust
for value in numbers.into_iter() {
    println!("{value}");
}
```

### B10.8 同時取得索引和值：`.enumerate()`

```rust
let numbers = vec![10, 20, 30];

for (index, value) in numbers.iter().enumerate() {
    println!("numbers[{index}] = {value}");
}
```

`.enumerate()` 每一圈提供一組 `(index, value)`：

```text
(0, &10)
(1, &20)
(2, &30)
```

`(index, value)` 稱為 tuple pattern，它把每一組中的兩個值分別取出。

### B10.9 `break` 與 `continue`

- `break`：立刻結束整個迴圈。
- `continue`：跳過本次剩餘內容，開始下一圈。

```rust
for number in 0..10 {
    if number == 2 {
        continue; // 不印 2
    }

    if number == 5 {
        break; // 到 5 時結束
    }

    println!("{number}");
}
```

輸出：

```text
0
1
3
4
```

### B10.10 走訪方式總表

| 寫法 | 每個 `value` | 是否取得 `Vec` 所有權 | 迴圈後能否使用原 `Vec` |
|---|---|---|---|
| `for value in numbers` | 元素本身 | 是 | 否 |
| `for value in &numbers` | 共享引用 | 否，只讀借用 | 可以 |
| `for value in &mut numbers` | 可變引用 | 否，可變借用 | 可以 |
| `for index in 0..numbers.len()` | 自己用 `[]` 取得 | 否 | 可以 |

初學時可以這樣選：

```text
只要讀每個元素       -> for value in &numbers
要修改每個元素       -> for value in &mut numbers
需要索引和值         -> numbers.iter().enumerate()
確定不再需要原 Vec   -> for value in numbers
```

</details>

<details>
<summary><strong>B11. 與 C/C++ 傳參方式的對照</strong></summary>


現在已經理解 Copy、Move 與 Borrow，才適合做語言對照。

### B11.1 C/C++ 與 Rust 的參數語法

C++：

```cpp
void print_number(int number);
```

Rust：

```rust
fn print_number(number: i32) {
    println!("{number}");
}
```

C/C++ 把型別放在名稱前，Rust 使用 `名稱: 型別`。

### B11.2 Pass by value

Rust 可以說所有引數都傳入一個值。傳入引用時，被傳入的值就是引用本身。

更實用的是判斷函式取得什麼權限：

| 意圖 | Rust | C++ | 結果 |
|---|---|---|---|
| 複製整數 | `value: i32` | `int value` | 複製數值 |
| 取得字串所有權 | `value: String` | `std::string value` 搭配 `std::move` | Rust 原變數失效 |
| 唯讀借用 | `value: &String` | `const std::string& value` | 不取得所有權 |
| 可修改借用 | `value: &mut String` | `std::string& value` | 修改原物件 |

C++ 把 lvalue `std::string` 傳給 value parameter 時通常會複製；Rust 傳入 `String` 預設會 Move，不會隱含昂貴的 `clone()`。

Rust 呼叫引用函式時也要明確寫 `&`：

```rust
read_text(&message);
append_world(&mut message);
```

C++ 呼叫 `const T&` 或 `T&` 參數時通常不在呼叫處寫 `&`。

### B11.3 C pointer

C 沒有 C++ reference，只有 pass by value。C 常把 pointer 的位址值複製進函式，模擬 pass by reference：

```c
void set_zero(int* value) {
    *value = 0;
}

int number = 10;
set_zero(&number);
```

Rust 對應的安全寫法：

```rust
fn set_zero(value: &mut i32) {
    *value = 0;
}

let mut number = 10;
set_zero(&mut number);
```

差異是：

- C pointer 可以是 null、dangling，也可以和其他 writable pointer 重疊。
- Safe Rust 的引用必須有效且非 null。
- `&mut` 在借用期間還必須獨占。
- Rust 編譯器會檢查引用不能比原資料活得更久。

Rust 也有 `*const i32`、`*mut i32` raw pointer，但 dereference 通常需要 `unsafe`。初學階段先不要使用。

</details>

<details>
<summary><strong>B12. Slice 與 Option</strong></summary>


### B12.1 Slice 借用連續資料

`&[i32]` 表示唯讀借用一段連續的 `i32`。概念上，一個 slice reference 包含兩項資訊：

```text
&[i32]
├── pointer：第一個元素的位置
└── length：從該位置開始共有幾個元素
```

它不需要擁有或複製元素。

先建立一個 `Vec<i32>`：

```rust
let numbers = vec![10, 20, 30, 40, 50];
```

元素在概念記憶體中的排列如下：

```text
原 Vec 的索引       0       1       2       3       4
                 ┌──────┬──────┬──────┬──────┬──────┐
heap 中的元素     │  10  │  20  │  30  │  40  │  50  │
                 └──────┴──────┴──────┴──────┴──────┘
```

使用 `&numbers[1..4]` 借用其中一段：

```rust
let middle: &[i32] = &numbers[1..4];
```

`1..4` 表示：

- 從原 `Vec` 的索引 `1` 開始。
- 到索引 `4` 之前停止，索引 `4` 本身不包含在內。
- 因此借到 `[20, 30, 40]`，長度是 `3`。

概念圖：

```text
原 Vec 的索引       0       1       2       3       4
                 ┌──────┬──────┬──────┬──────┬──────┐
heap 中的元素     │  10  │  20  │  30  │  40  │  50  │
                 └──────┴──────┴──────┴──────┴──────┘
                            ^
                            │ pointer：從原索引 1 開始
                 middle ────┘ length：3

middle 看到的索引            0       1       2
middle 看到的值              20      30      40
```

Slice 自己的索引會從 `0` 重新開始：

```rust
assert_eq!(middle[0], 20);
assert_eq!(middle[1], 30);
assert_eq!(middle[2], 40);
assert_eq!(middle.len(), 3);
```

常見範圍寫法：

| 寫法 | 意思 | 上例的結果 |
|---|---|---|
| `&numbers[1..4]` | 索引 1 到 4 之前 | `[20, 30, 40]` |
| `&numbers[..3]` | 開頭到索引 3 之前 | `[10, 20, 30]` |
| `&numbers[2..]` | 索引 2 到結尾 | `[30, 40, 50]` |
| `&numbers[..]` | 完整範圍 | `[10, 20, 30, 40, 50]` |

範圍超出原資料邊界時，程式會在執行期 panic。例如長度為 5 時，`&numbers[1..6]` 就是無效範圍。

回到原本的函式範例：

```rust
fn print_length(numbers: &[i32]) {
    println!("{}", numbers.len());
}

let numbers = vec![10, 20, 30, 40, 50];

print_length(&numbers);       // 借用完整 Vec，長度是 5
print_length(&numbers[1..4]); // 借用部分資料，長度是 3
```

`print_length(&numbers)` 中，Rust 會把 `&Vec<i32>` 自動轉成完整的 `&[i32]`。這個動作稱為 deref coercion；在這裡可以先把它理解成等同於：

```rust
print_length(&numbers[..]);
```

原本的簡短範例沒有寫出 `[..]`，所以「起始位置和長度」看起來像消失了。實際上完整 slice 的 pointer 指向第一個元素，length 使用 `Vec` 目前的元素數量。

```text
&numbers
等同於這裡的完整 slice &numbers[..]

pointer ──> 原索引 0
length  = 5
```

#### 參數已經是 reference，為什麼有時還要寫 `&`

先看 C++：

```cpp
void fun(std::vector<int>& a) {
    a.push_back(40);
    std::cout << a[0];
}
```

Rust 中接近的形式是：

```rust
fn fun(a: &mut Vec<i32>) {
    a.push(40);
    println!("{}", a[0]);
}
```

進入 Rust 函式後，`a` 已經是 reference。呼叫 method、取得長度、使用索引時，Rust 會自動 dereference，因此不需要每次都再寫 `&`：

```rust
fn inspect(numbers: &[i32]) {
    println!("{}", numbers.len()); // 不需要 &
    println!("{}", numbers[0]);    // 不需要 &

    for number in numbers {        // 完整走訪也不需要 &
        println!("{number}");
    }
}
```

但以下兩種操作是在建立一個 **新的 reference**，因此需要寫 `&`：

```rust
fn inspect(numbers: &[i32]) {
    // 建立「第一個元素」的 reference。
    let first: &i32 = &numbers[0];

    // 建立「從索引 1 到結尾」的 sub-slice reference。
    let rest: &[i32] = &numbers[1..];

    println!("{first}");
    println!("{rest:?}");
}
```

型別變化：

```text
numbers           : &[i32]  原本已經是完整 slice 的 reference
numbers[0]        : i32     索引位置上的元素
&numbers[0]       : &i32    新建立的元素 reference
numbers[1..]      : [i32]   選出的 slice 區域
&numbers[1..]     : &[i32]  新建立的 sub-slice reference
```

所以 `largest` 中：

```rust
let mut max = &numbers[0];

for number in &numbers[1..] {
    if *number > *max {
        max = number;
    }
}
```

- `numbers` 已經是 `&[i32]`。
- `&numbers[0]` 是再借出第一個元素，讓 `max` 成為 `&i32`。
- `&numbers[1..]` 是再借出剩餘區段，形成新的 `&[i32]`。
- 迴圈中的 `number` 是該 sub-slice 每個元素的 `&i32`。

如果走訪完整 slice，不需要額外的 `&`：

```rust
for number in numbers {
    println!("{number}");
}
```

如果要跳過第一個元素，也可以使用 iterator，不直接寫 sub-slice borrow：

```rust
for number in numbers.iter().skip(1) {
    println!("{number}");
}
```

C++ 和 Rust 的元素 reference 對照：

```cpp
const int& first = a[0];
//       ^ 在變數宣告中表示 first 是 reference
```

```rust
let first: &i32 = &numbers[0];
//         ^      ^ 建立 borrow
//         │
//         └─ first 的型別是 reference
```

可以記成：

```text
參數 numbers 已經是 &[i32]
├─ numbers.len()       -> 一般使用，不用再加 &
├─ numbers[0]          -> 讀取元素，不用再加 &
├─ for x in numbers    -> 完整走訪，不用再加 &
├─ &numbers[0]         -> 建立元素 reference
└─ &numbers[1..]       -> 建立 sub-slice reference
```

重點不是「Rust reference 每次使用都要加 `&`」，而是：**原 reference 可以直接使用；要從它再借出一個元素或子區段時，才明確寫出新的 borrow。**

`&mut [i32]` 則是可修改、獨占借用一段連續資料：


```rust
fn swap_first_two(numbers: &mut [i32]) {
    numbers.swap(0, 1);
}

let mut numbers = vec![10, 20, 30, 40];
swap_first_two(&mut numbers[1..3]);

assert_eq!(numbers, vec![10, 30, 20, 40]);
```

傳入的 slice 是原索引 `1..3`，內容為 `[20, 30]`。函式中的索引 `0`、`1` 對應原 `Vec` 的索引 `1`、`2`，因此只交換這兩個元素。

總結：Slice 不擁有元素，只借用原資料，並以 **pointer + length** 描述「從哪個元素開始、連續包含多少個元素」。原本的 `Vec` 仍然是 owner。

### B12.2 `Option` 表示可能有值

空 slice 沒有第一個元素，所以不能保證回傳 `&i32`：

```rust
fn first(numbers: &[i32]) -> Option<&i32> {
    numbers.first()
}
```

- `Some(value)`：有值。
- `None`：沒有值。

```rust
let numbers = vec![10, 20];
assert_eq!(first(&numbers), Some(&10));

let empty: Vec<i32> = vec![];
assert_eq!(first(&empty), None);
```

如果要安全地修改第一個元素，可以使用 `.first_mut()`：

```rust
fn clear_first(numbers: &mut [i32]) {
    // first_mut() 嘗試取得第一個元素的可變引用。
    // 非空 slice -> Some(&mut 第一個元素)
    // 空 slice   -> None
    if let Some(first) = numbers.first_mut() {
        // first 的型別是 &mut i32。
        // *first 表示該引用所指向的實際 i32。
        *first = 0;
    }
}
```

`.first_mut()` 是 slice 提供的 method（方法）。

先說明 **function signature（函式簽名）**：它是函式名稱、參數型別與回傳型別的組合，不包含 `{ ... }` 裡的實作內容。例如：

```rust
fn add(left: i32, right: i32) -> i32
```

這個 signature 告訴我們：

- 函式名稱是 `add`。
- 接收兩個 `i32`。
- 回傳一個 `i32`。

下面不是 `.first_mut()` 在標準函式庫中的原始寫法，而是為了方便理解，把它改寫成「一般函式形式」：

```rust
fn first_mut(numbers: &mut [i32]) -> Option<&mut i32>
```

這個改寫刻意做了兩件事：

1. 把點號前面的 `numbers` 改成普通參數。
2. 把通用的元素型別固定成目前範例使用的 `i32`。

意思是：

- 輸入 `&mut [i32]`：獨占借用一個可修改的 `i32` slice。
- 回傳 `Option<...>`：因為 slice 可能是空的。
- `Some(&mut i32)`：非空時，借出第一個元素的可變引用。
- `None`：空 slice 沒有第一個元素。

實際呼叫方法時，`numbers` 寫在點號前面：

```rust
numbers.first_mut()
// 可以先理解成一般函式形式的：first_mut(numbers)
```

標準函式庫中的真正形式概念上更接近：

```rust
fn first_mut(&mut self) -> Option<&mut T>
```

- `self` 代表點號前面的 slice，也就是這裡的 `numbers`。
- `&mut self` 表示 method 會可變、獨占借用該 slice。
- `T` 代表 slice 的元素型別；本例的 `T` 是 `i32`。
- `Option<&mut T>` 表示可能取得第一個元素的可變引用，也可能因為 slice 為空而得到 `None`。

因此這三種視角描述的是同一件事：

```text
實際呼叫：       numbers.first_mut()
method 形式：     fn first_mut(&mut self) -> Option<&mut T>
本例改寫形式：   fn first_mut(numbers: &mut [i32]) -> Option<&mut i32>
```

`if let Some(first) = ...` 會檢查並拆開 `Option`：

```text
numbers.first_mut()
        │
        ├─ slice 非空 -> Some(&mut numbers[0])
        │                    │
        │                    └─ 名稱 first 接住這個 &mut i32
        │
        └─ slice 為空 -> None
                          │
                          └─ 不進入 if 區塊
```

因此程式不需要先手動檢查 `numbers.len() > 0`。

#### 為什麼不能直接寫 `numbers[0] = 0`

以下寫法對非空 slice 有效：

```rust
fn clear_first_by_index(numbers: &mut [i32]) {
    numbers[0] = 0;
}
```

但傳入空 slice 時，索引 `0` 不存在，程式會 panic：

```rust,should_panic
let mut empty: Vec<i32> = vec![];
clear_first_by_index(&mut empty);
```

`.first_mut()` 把「可能沒有第一個元素」明確表示成 `Option`，呼叫者必須處理 `Some` 或 `None`，因此不會因為空 slice 而 panic。

#### `first` 為什麼要加 `*`

在 `Some` 分支中：

```rust
first: &mut i32
```

`first` 本身是引用，可以想成保存了第一個元素的位置：

```text
first: &mut i32 ─────────> numbers[0]: i32
                              目前的值
```

`*first` 是 dereference，表示「沿著引用找到它指向的實際值」：

```text
first      = 可變引用
*first     = 引用指向的 i32
*first = 0 = 把該 i32 修改為 0
```

完整執行範例：

```rust
let mut numbers = vec![10, 20, 30];
clear_first(&mut numbers);
assert_eq!(numbers, vec![0, 20, 30]);

let mut empty: Vec<i32> = vec![];
clear_first(&mut empty); // first_mut() 回傳 None，什麼也不做
assert!(empty.is_empty());
```

`.first()` 與 `.first_mut()` 的差異如下：

| 方法 | 接收的借用 | 回傳型別 | 可以修改元素嗎？ |
|---|---|---|---|
| `.first()` | `&[i32]` | `Option<&i32>` | 不可以 |
| `.first_mut()` | `&mut [i32]` | `Option<&mut i32>` | 可以 |

兩個方法都使用 `Option` 安全處理空 slice；差別在於前者提供共享引用，後者提供獨占的可變引用。

</details>

<details>
<summary><strong>B13. 泛型中的 T 代表什麼</strong></summary>


現在已經看過 `Vec<i32>`、`Option<&i32>` 和引用，可以開始理解 `T`。

`T` 通常代表 Type。它不是內建型別，而是「稍後代入的型別」的名稱。

```rust
fn identity<T>(value: T) -> T {
    value
}
```

```text
fn identity<T>(value: T) -> T
            ^        ^      ^
            │        │      └─ 回傳同一個 T
            │        └──────── value 的型別是 T
            └───────────────── 宣告型別參數 T
```

呼叫時，`T` 會被具體型別取代：

```rust
let number = identity(10);                   // T = i32
let text = identity(String::from("hello")); // T = String
```

`T` 在同一次呼叫中必須一致：

```rust
fn choose_first<T>(first: T, _second: T) -> T {
    first
}
```

```rust,compile_fail
choose_first(10, String::from("hello"));
```

這一次的 `T` 不可能同時是 `i32` 和 `String`。

需要不同型別時，宣告多個參數：

```rust
fn keep_left<L, R>(left: L, _right: R) -> L {
    left
}
```

### B13.1 `Vec<T>` 與 `Option<T>`

```text
Vec<T>    = 保存 T 的可變長度陣列
Option<T> = Some(T) 或 None
&T        = 共享借用一個 T
&mut T    = 可變、獨占借用一個 T
```

具體使用：

```rust
let numbers: Vec<i32> = vec![10, 20];
let maybe_number: Option<i32> = Some(10);
```

真正使用 `T` 的函式必須先宣告 `<T>`：

```rust
fn read<T>(_value: &T) {}
```

只寫以下內容會找不到 `T`：

```rust,compile_fail
fn read(_value: &T) {}
```

### B13.2 `K`、`V` 與其他名稱

型別參數不一定叫 `T`：

| 名稱 | 常見意思 |
|---|---|
| `T`、`U` | 一般型別 |
| `K` | Key |
| `V` | Value |
| `E` | Error |

LRU 使用兩個型別：

```rust
pub struct LruCache<K, V> {
    // K 是 key 型別，V 是 value 型別
}
```

### B13.3 Trait bound

泛型不表示可以對 `T` 做任何操作。要比較大小，必須要求 `T` 支援排序：

```rust
fn smaller<T: Ord>(left: T, right: T) -> T {
    if left <= right { left } else { right }
}
```

`T: Ord` 是 trait bound，表示 `T` 必須實作 `Ord`。

這和 C++ template 概念相近：

```cpp
template <typename T>
T identity(T value) {
    return value;
}
```

Rust 通常在編譯期把泛型具體化。`T` 不是執行期才猜測內容的萬用盒子。

記住：**`T` 是型別佔位名稱；`<T>` 宣告它，呼叫時再由具體型別取代。**

</details>

## C. 實作練習、測試與進度

這一區把前面的知識用在實際程式碼，並提供完成標準。

<details>
<summary><strong>C1. 執行 Ownership 完整範例</strong></summary>


開啟 [`examples/ownership_demo.rs`](examples/ownership_demo.rs)，執行：

```powershell
cargo run --example ownership_demo
```

範例展示：

1. 傳入 `Vec<i32>` 會 Move。
2. 傳入 `&[i32]` 只借來讀。
3. 傳入 `&mut Vec<i32>` 可以修改原資料。
4. 多個共享引用可以並存。

先照原樣執行，再一次取消一行註解，觀察 Move 或 Borrow 的第一個編譯錯誤。

</details>

<details>
<summary><strong>C2. 測試與 todo!()</strong></summary>


```rust
#[test]
fn two_plus_two_is_four() {
    assert_eq!(2 + 2, 4);
}
```

- `#[test]`：標記測試函式。
- `assert_eq!(actual, expected)`：比較實際值與預期值。

練習中的 `todo!()` 可以通過編譯，但執行到該行會 panic。這是刻意留下的空格。

```powershell
cargo test ownership_xor
```

一開始測試失敗很正常。每次只完成一個函式，讓失敗數量逐步減少。

</details>

<details>
<summary><strong>C3. 第一關的解題順序</strong></summary>


開啟 [`src/ownership_xor.rs`](src/ownership_xor.rs)，依序完成：

1. `consume_and_sum`：函式取得 `Vec` 所有權。
2. `append_number`：用可變引用修改 vector。
3. `largest`：借用 slice 並回傳元素引用。
4. `swap_positions`：透過可變 slice 修改兩個位置。

第一題可以先使用最直白的寫法：

```rust
pub fn consume_and_sum(numbers: Vec<i32>) -> i32 {
    let mut total = 0;

    for number in numbers {
        total += number;
    }

    total
}
```

- 函式取得 `numbers` 的所有權。
- `total` 會修改，所以使用 `mut`。
- `for` 逐一取出 vector 中的數值。
- 最後的 `total` 沒有分號，因此成為回傳值。

### C3.1 目前 `ownership_xor.rs` 的檢查結果

這次執行：

```powershell
cargo test ownership_xor
```

結果是四個測試全部通過：

```text
4 passed; 0 failed
```

這表示目前四個函式在現有測試案例下行為正確。以下建議主要改善命名、格式與測試覆蓋，不是修正失敗的演算法。

### C3.2 各函式 review

| 函式 | 目前做得好的地方 | 可以改進的地方 |
|---|---|---|
| `consume_and_sum` | 正確取得 `Vec` 所有權，`for number in numbers` 清楚展示 consuming iteration | 完成後移除舊的 `//todo!` 註解 |
| `append_number` | 正確使用 `&mut Vec<i32>`，呼叫 `push` 修改原資料 | 完成後移除舊的 `//todo!` 註解 |
| `largest` | 正確處理空 slice、以第一個元素初始化、跳過第一次自我比較，並回傳原元素引用 | `max_index` 不是索引，應改成 `max` 或 `max_value`；補空格與邊界測試 |
| `swap_positions` | 使用標準函式庫 `swap`，不用自己建立兩個重疊的可變引用 | `numbers.swap(a,b)` 應排成 `numbers.swap(a, b)`；補同索引案例 |

### C3.3 `largest` 的命名

目前：

```rust
let mut max_index = &numbers[0];
```

`max_index` 這個名稱通常表示 `usize` 索引，例如 `0`、`1`、`2`。但目前實際型別是：

```text
max_index: &i32
```

它保存的是最大元素的引用，不是索引。因此更準確的名稱是：

```rust
let mut max = &numbers[0];
```

搭配比較後，程式意圖會更直接：

```rust
for number in &numbers[1..] {
    if *number > *max {
        max = number;
    }
}

Some(max)
```

目前的比較：

```rust
if *max_index < *number
```

邏輯也是正確的。改成 `if *number > *max` 只是閱讀順序更接近「如果新數字比目前最大值大」。

### C3.4 已完成的 `todo!()` 註解

目前函式中仍保留：

```rust
//todo!("走訪 numbers 並回傳總和")
```

這些註解在實作完成後已失去用途，建議直接刪除。若想保留學習紀錄，可以改成解釋「為什麼這樣寫」的註解，例如：

```rust
// 從索引 1 開始，避免第一個元素和自己比較。
for number in &numbers[1..] {
    // ...
}
```

好的註解應說明原因或不明顯的限制，而不是保留已完成的待辦事項。

### C3.5 格式改善

`cargo fmt -- --check` 指出的 `ownership_xor.rs` 格式差異是：

```rust
// 修改前
if numbers.is_empty(){
for number in &numbers[1..]{
if *max_index < *number{
numbers.swap(a,b);

// 建議格式
if numbers.is_empty() {
for number in &numbers[1..] {
if *number > *max {
numbers.swap(a, b);
```

上面只展示單行差異，不是完整、可直接編譯的函式。相關工具指令統一放在 A5。

### C3.6 建議補上的測試

現有 `largest` 測試涵蓋一般資料和空 slice。還可以增加：

```rust
#[test]
fn largest_handles_negative_single_and_duplicate_values() {
    assert_eq!(largest(&[-10, -5, -20]), Some(&-5));
    assert_eq!(largest(&[7]), Some(&7));
    assert_eq!(largest(&[9, 9, 3]), Some(&9));
}
```

這三個案例分別確認：

- 全負數時不會錯用 `0` 或 `-1` 當初始最大值。
- 只有一個元素時，`numbers[1..]` 是空 slice，仍能回傳第一個元素。
- 最大值重複時，仍回傳正確值。

`swap_positions` 可以增加同索引測試：

```rust
#[test]
fn swapping_the_same_position_is_a_no_op() {
    let mut numbers = vec![10, 20, 30];
    swap_positions(&mut numbers, 1, 1);
    assert_eq!(numbers, vec![10, 20, 30]);
}
```

`swap` 的索引越界時會 panic。這不是目前函式的 bug，而是 API contract；呼叫者必須傳入有效索引。如果未來需求是「索引無效時不要 panic」，就需要改變函式回傳型別和規格。

### C3.7 可選的進階寫法

`consume_and_sum` 也可以用 iterator：

```rust
pub fn consume_and_sum(numbers: Vec<i32>) -> i32 {
    numbers.into_iter().sum()
}
```

但目前的 `for` 版本對學習 Ownership 更好，因為可以清楚看到 `numbers` 被移入迴圈。現階段不需要為了縮短程式而改成 iterator。

</details>

<details>
<summary><strong>C4. 進入 DSU 前的確認</strong></summary>


完成下面的基礎確認後，依序閱讀 [`DSU_START_HERE.md`](DSU_START_HERE.md)。那份教材會從 `struct`、`impl`、`Self`、`&mut self` 和 `Vec<usize>` 開始，使用陣列與記憶體圖解說明 DSU，不會直接提供完整答案。


能回答以下問題後，再開始 [`src/dsu.rs`](src/dsu.rs)：

- `let` 與 `let mut` 有什麼差別？
- `let number = 10` 為什麼通常是 `i32`？
- `String` 指派給另一個變數時，什麼是 Move？
- `i32` 的 Copy 與 `String::clone()` 有何不同？
- `&String` 與 `&mut String` 有什麼差別？
- 為什麼不能同時有兩個可變引用？
- `0..3` 與 `0..=3` 分別會產生哪些數字？
- `for value in numbers`、`&numbers`、`&mut numbers` 對所有權和借用有何不同？
- `Vec<i32>` 與 `&[i32]` 分別代表擁有和借用嗎？
- `Option<&i32>` 的 `Some` 和 `None` 是什麼？
- 泛型 `T` 是什麼？

</details>

<details>
<summary><strong>C5. 建議學習節奏</strong></summary>


第一次約 30 分鐘：

1. 跑 `cargo run --example hello`。
2. 讀 A1 至 A3，以及 B1 至 B2。
3. 自己修改幾個整數型別和 `add` 函式。

第二次約 45 分鐘：

1. 讀 B3 至 B6。
2. 理解 stack、heap、Move、Copy 與 Clone。
3. 暫時不要進入借用。

第三次約 45 分鐘：

1. 讀 B7 至 B10。
2. 跑 `cargo run --example ownership_demo`。
3. 觀察 `&`、`&mut`、XOR 與三種 `for` 走訪方式。

第四次約 45 分鐘：

1. 讀 B11 至 B13。
2. 複習 C/C++ 對照、slice、`Option` 與泛型 `T`。
3. 自己寫一個唯讀走訪和一個可修改走訪。

第五次約 45 分鐘：

1. 讀 C1 至 C3。
2. 完成 `consume_and_sum`、`append_number`。
3. 執行 `cargo test ownership_xor`。

第六次再完成 `largest`、`swap_positions`。第 0 關全部通過後才開始 DSU；Tree 和 LRU 暫時不要打開。

第七次先完整閱讀 [`DSU_START_HERE.md`](DSU_START_HERE.md) 的 D1 至 D7，只實作 `find`。確認 path compression 測試通過後，再閱讀 D8 至 D12 並實作 `union`。

</details>
