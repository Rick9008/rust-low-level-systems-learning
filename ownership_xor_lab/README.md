# Ownership 與 XOR 練習區

這是一個獨立的 Rust 入門專案。目標不是背語法，而是透過四個階段理解：

- 值由誰擁有，以及函式呼叫時有沒有移動所有權。
- `&T` 只能讀，`&mut T` 可以修改。
- 同一時間可以有多個 `&T`，或一個 `&mut T`，但不能兩者並存。
- 複雜資料結構如何用 `Vec`、索引與小函式避開借用衝突。

> 如果你完全沒有接觸過 Rust，請先讀 [`START_HERE.md`](START_HERE.md)。不要直接打開 DSU。那份教材會從編譯器、Cargo、變數與函式開始逐行解釋。

## 目前的環境

建立這個資料夾時，電腦上找不到 `rustc` 與 `cargo`。練習檔案已準備好，但要執行測試前仍需安裝 Rust。

## 在 Windows 安裝 Rust

1. 前往 [Rust 官方安裝頁](https://www.rust-lang.org/tools/install)。
2. 下載並執行 `rustup-init.exe`，使用畫面提供的預設安裝選項。
3. 安裝完成後，關閉並重新開啟 PowerShell 或 Codex。
4. 執行以下指令確認環境：

```powershell
rustc --version
cargo --version
```

如果安裝程式提示缺少 Visual Studio C++ Build Tools，依提示安裝「Desktop development with C++」工作負載，再重新執行 Rust 安裝程式。

編輯器可選擇 VS Code，並安裝 `rust-analyzer` extension。這不是執行練習的必要條件，但會即時顯示型別與借用錯誤。

## 練習順序

### 開始前：跑懂兩個完整範例

安裝環境後，先執行：

```powershell
cd D:\rust_pratice\ownership_xor_lab
cargo run --example hello
cargo run --example ownership_demo
```

這兩個 example 沒有 `todo!()`，應該可以直接成功。請搭配 [`START_HERE.md`](START_HERE.md) 閱讀每一行。

### 0. Ownership 與借用暖身

開啟 [`src/ownership_xor.rs`](src/ownership_xor.rs)，完成四個 `todo!()`。

```powershell
cd D:\rust_pratice\ownership_xor_lab
cargo test ownership_xor
```

這一關先熟悉 `Vec<T>`、`&[T]`、`&mut Vec<T>` 與回傳引用。

### 1. DSU

先閱讀 [`DSU_START_HERE.md`](DSU_START_HERE.md)。這份先備教材會從 `struct`、`impl`、`Self`、`&mut self`、索引式森林、path compression 與 union by rank 開始圖解，再帶你閱讀測試。

讀完後開啟 [`src/dsu.rs`](src/dsu.rs)，依序完成 `find`、`union`。

```powershell
cargo test dsu
```

這一關練習由 struct 擁有多個 `Vec`，以及如何縮短 mutable borrow 的存活範圍。

### 2. Arena Tree

開啟 [`src/tree.rs`](src/tree.rs)，依序完成 `insert`、`inorder`。

```powershell
cargo test tree
```

所有節點都由一個 `Vec` 擁有，節點之間只保存索引。這能避開 `Rc<RefCell<_>>`，也比較容易觀察所有權。

### 3. LRU Cache

開啟 [`src/lru.rs`](src/lru.rs)，依序完成 `detach`、`push_front`、`get`、`put`。

```powershell
cargo test lru
```

這一關把 `HashMap` 與 index-based 雙向鏈表組合起來，是前三關的綜合題。請先在紙上畫出節點的 `prev`、`next`，再修改程式。

## 建議的解題方式

每次只處理一個 `todo!()`。先讀函式上方的規格，再執行該階段測試。看到編譯錯誤時先看第一個錯誤，特別注意：

- `value moved here`：值的所有權已被移走。
- `cannot borrow ... as mutable`：目前只有不可變引用，或另一個借用仍然存活。
- `cannot borrow ... more than once`：同一時間建立了兩個重疊的 mutable borrow。
- `does not live long enough`：引用活得比它指向的值更久。

卡住時先回到 [`../docs/rust-five-axis.md`](../docs/rust-five-axis.md) 的 Ownership 與 XOR 兩節。完成自己的版本後，再對照 `../reference/src/ds/`，不要一開始就看答案。

## 完成標準

以下指令全部通過，就完成這組練習：

```powershell
cargo test
```
