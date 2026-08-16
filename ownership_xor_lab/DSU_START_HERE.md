# 從零看懂 DSU：Rust 先備知識與圖解

這份教材接在 [`START_HERE.md`](START_HERE.md) 的 Ownership 暖身之後。目標不是立刻背出 DSU，而是先看懂 [`src/dsu.rs`](src/dsu.rs) 中已經出現的每一種 Rust 語法，以及這些語法在記憶體中代表什麼。

> 建議方式：依序展開各節。讀完一節就回到 `dsu.rs` 找到相同語法。D1 到 D8 都看懂後，才開始填 `find`；`find` 通過後才填 `union`。

## 學習地圖

```text
struct 擁有欄位
      ↓
impl、Self、self
      ↓
Vec、usize、索引讀寫
      ↓
parent 陣列表示森林
      ↓
find 找 root + path compression
      ↓
union by rank + components
```

現階段不需要 `Rc`、`RefCell`、指標、遞迴、lifetime 標註或泛型。這份 DSU 刻意只用 `Vec` 和索引，讓每一份資料都有清楚的 owner。

<details open>
<summary><strong>D1. DSU 到底要解決什麼問題</strong></summary>

DSU 是 **Disjoint Set Union**，也常叫 **Union-Find**。它維護數個互不重疊的集合，主要回答兩種問題：

1. `find(x)`：`x` 屬於哪一組？
2. `union(x, y)`：把 `x` 和 `y` 所屬的兩組合併。

假設一開始有六個元素，每個元素各自一組：

```text
{0}  {1}  {2}  {3}  {4}  {5}
```

執行：

```text
union(0, 1)
union(1, 2)
union(3, 4)
```

集合會變成：

```text
{0, 1, 2}  {3, 4}  {5}
```

因此：

```text
connected(0, 2) == true
connected(0, 3) == false
components()    == 3
```

`connected(x, y)` 不必逐一比較整個集合，只要判斷兩者的 root 是否相同：

```rust
self.find(x) == self.find(y)
```

</details>
<details>
<summary><strong>D2. struct：一個 Dsu 值擁有哪些資料</strong></summary>

`dsu.rs` 開頭是：

```rust
pub struct Dsu {
    parent: Vec<usize>,
    rank: Vec<u8>,
    components: usize,
}
```

`struct` 把數個相關欄位組成一個新型別。可以先用 C/C++ 對照：

```cpp
struct Dsu {
    std::vector<std::size_t> parent;
    std::vector<std::uint8_t> rank;
    std::size_t components;
};
```

Rust 中的 `Dsu` 值是三個欄位的 owner：

```text
Stack 上的 Dsu
┌──────────────────────────────┐
│ parent: Vec 描述資訊 ─────────────> Heap 上的 parent 元素
│ rank:   Vec 描述資訊 ─────────────> Heap 上的 rank 元素
│ components: usize            │
└──────────────────────────────┘
```

這裡沒有讓節點互相保存 Rust reference。`parent` 儲存的是數字索引，所以不需要處理「節點引用是否活得夠久」。

### 三個欄位各自代表什麼

| 欄位 | 型別 | 用途 |
|---|---|---|
| `parent` | `Vec<usize>` | `parent[x]` 記錄 x 的父節點索引 |
| `rank` | `Vec<u8>` | root 的樹高估計，用來決定合併方向 |
| `components` | `usize` | 目前還有幾個集合 |

欄位前面沒有 `pub`，表示其他 module 不能直接修改它們。外部程式應透過 `new`、`find`、`union`、`connected` 與 `components` 操作 DSU。

</details>

<details>
<summary><strong>D3. usize、u8 與 bool</strong></summary>

### `usize`：索引與長度使用的整數

Rust 的 slice 與 `Vec` 使用 `usize` 作為索引：

```rust
let index: usize = 2;
let value = numbers[index];
```

因此 DSU 的元素編號、`parent` 內容、`components` 和 `size` 都使用 `usize`。它是無號整數，不能表示負數。

```text
parent: Vec<usize>
            └──── 每個元素都是另一個位置的索引
```

### `u8`：0 到 255 的小型無號整數

`rank` 只需要保存很小的樹高估計，所以使用 `u8`：

```rust
rank: Vec<u8>
```

這不是 DSU 的必要規定；使用 `usize` 也能實作。選擇 `u8` 只是表達「這是一個很小的非負數」。

### `bool`：是否真的合併

`union` 的回傳型別是 `bool`：

```rust
pub fn union(&mut self, x: usize, y: usize) -> bool
```

- 回傳 `true`：原本是不同集合，這次真的合併了。
- 回傳 `false`：原本已在同一集合，沒有改變資料。

</details>

<details>
<summary><strong>D4. impl、Self、self、&self 與 &mut self</strong></summary>

### `impl Dsu`

```rust
impl Dsu {
    // 與 Dsu 有關的函式和 method 放在這裡
}
```

`impl Dsu` 表示接下來這組函式屬於 `Dsu` 的實作。它與 C++ 在 class/struct 中定義 constructor 和 member function 的用途相近。

### `Self` 是目前正在 impl 的型別

在 `impl Dsu` 裡：

```text
Self 代表 Dsu
```

所以：

```rust
pub fn new(size: usize) -> Self
```

等價於寫：

```rust
pub fn new(size: usize) -> Dsu
```

`new` 沒有 `self` 參數，因為呼叫它時 Dsu 還不存在。它負責建立並回傳一個新的 Dsu：

```rust
let mut dsu = Dsu::new(6);
```

### `self` 是呼叫 method 的那個值

```rust
dsu.components()
```

進入 method 後，`self` 就代表 `dsu`。

| 參數 | 意思 | C++ 粗略對照 | 能否修改欄位 |
|---|---|---|---|
| `self` | 取得整個值的所有權 | value receiver | 可以，但呼叫後原變數通常不能再用 |
| `&self` | 共享借用目前值 | `const` member function 的 `this` | 不行 |
| `&mut self` | 獨占、可變借用目前值 | non-const member function 的 `this` | 可以 |

因此：

```rust
pub fn components(&self) -> usize
```

只讀取計數，不必修改 Dsu；而：

```rust
pub fn find(&mut self, x: usize) -> usize
```

需要在 path compression 時改寫 `parent`，所以必須取得 `&mut self`。

呼叫 method 時不用手動寫 `&mut dsu`：

```rust
let root = dsu.find(0);
```

Rust 會依 method 簽名自動借用成 `&mut dsu`。但變數本身仍必須宣告為 `mut`：

```rust
let mut dsu = Dsu::new(6);
```

</details>

<details>
<summary><strong>D5. 看懂 Dsu::new：range、collect 與 vec! 巨集</strong></summary>

完整的 constructor 已經寫好：

```rust
pub fn new(size: usize) -> Self {
    Self {
        parent: (0..size).collect(),
        rank: vec![0; size],
        components: size,
    }
}
```

### `Self { ... }` 建立 struct 值

```rust
Self {
    parent: ...,
    rank: ...,
    components: ...,
}
```

每個欄位都必須取得一個符合型別的初始值。最後沒有分號，因此整個 `Self { ... }` 是 `new` 的回傳值。

### `(0..size).collect()` 建立 parent

若 `size == 5`：

```text
0..size           產生 0, 1, 2, 3, 4
.collect()         收集成 Vec<usize>
parent             [0, 1, 2, 3, 4]
```

`0..size` 不包含 `size`。Rust 能從欄位型別 `parent: Vec<usize>` 推論 `.collect()` 要建立 `Vec<usize>`。

一開始每個元素的 parent 都是自己，表示每個元素各自是一棵只有一個節點的樹：

```text
索引 x       0  1  2  3  4
parent[x]    0  1  2  3  4
```

### `vec![0; size]` 建立 rank

這個形式表示「建立 `size` 個 0」：

```rust
let rank = vec![0; 5];
// rank == [0, 0, 0, 0, 0]
```

不要和列出元素的形式混淆：

```rust
vec![0; 5]     // [0, 0, 0, 0, 0]
vec![0, 5]     // [0, 5]
```

</details>

<details>
<summary><strong>D6. parent 陣列如何表示樹與 root</strong></summary>

DSU 不需要真的建立 pointer-based tree。每個位置只保存父節點的索引。

假設：

```rust
parent = vec![1, 2, 2, 4, 4];
```

逐格解讀：

```text
parent[0] == 1    0 的父節點是 1
parent[1] == 2    1 的父節點是 2
parent[2] == 2    2 的父節點是自己，所以 2 是 root
parent[3] == 4    3 的父節點是 4
parent[4] == 4    4 的父節點是自己，所以 4 是 root
```

畫成森林：

```text
0 -> 1 -> 2       3 -> 4
          ^            ^
          root         root
```

判斷 root 的條件是：

```rust
self.parent[current] == current
```

`self.parent[current]` 會先索引 `parent`，取得目前位置保存的父節點編號。

### 索引讀取與索引寫入

```rust
let next = self.parent[current]; // 讀取，usize 會 Copy
self.parent[current] = root;     // 修改 Vec 中的一格
```

因為 `usize` 實作了 `Copy`，讀出 `next` 只複製一個小整數，不會把 `parent` 或 Dsu 的所有權移走。

### 邊界條件

如果 `x >= self.parent.len()`，使用 `self.parent[x]` 會 panic。這份入門練習把「呼叫者提供有效索引」當成 API 前提，暫時不額外回傳 `Option` 或 `Result`。

</details>

<details>
<summary><strong>D7. find：先找 root，再壓縮路徑</strong></summary>

`find(x)` 有兩個工作：

1. 沿著 parent 一直往上，找到 root。
2. 把路徑上的節點改成直接指向 root。

題目要求使用「兩趟式」寫法，因為它能把讀取和修改拆開，對初學 Ownership 與 Borrow 較清楚。

### 第一趟：只找 root

初始狀態：

```text
x = 0

0 -> 1 -> 2 -> 3 -> 4
                    ^
                    root，因為 parent[4] == 4
```

使用一個 `current` 索引向上移動：

```text
current: 0 -> 1 -> 2 -> 3 -> 4
```

當 `parent[current] == current` 時停止，此時 `current` 就是 root。可以把它另存成名稱更清楚的 `root`。

### 第二趟：修改 parent

找到 `root == 4` 後，再從原本的 `x == 0` 走一次：

```text
修改前：0 -> 1 -> 2 -> 3 -> 4
修改後：0 ─┐
         1 ─┼──────────────> 4
         2 ─┤                ^ root
         3 ─┘
```

對應陣列：

```text
修改前 parent = [1, 2, 3, 4, 4]
修改後 parent = [4, 4, 4, 4, 4]
```

這叫 **path compression（路徑壓縮）**。下次查詢 0 時只需走一步。

### 為什麼修改前要先保存 next

第二趟若直接把 `parent[current]` 改成 root，原本「下一站在哪裡」的資訊就消失了。因此順序必須是：

```text
1. next = 原本的 parent[current]
2. parent[current] = root
3. current = next
```

這也示範一個重要 Rust 習慣：先把需要的 `Copy` 資料讀進區域變數，讓讀取結束，再修改容器。

### `find` 為何回傳 usize 而不是 &usize

root 是一個小型的索引值，`usize` 可以便宜地 Copy：

```rust
pub fn find(&mut self, x: usize) -> usize
```

直接回傳數字不會讓外部持續借用 Dsu。若回傳 `&usize`，引用會綁住 `self.parent` 的借用，反而妨礙接下來修改 DSU。

</details>

<details>
<summary><strong>D8. union：先完成兩次 find，再修改欄位</strong></summary>

`union(x, y)` 的概念步驟是：

```text
1. 找 x 的 root
2. 找 y 的 root
3. root 相同：回傳 false
4. root 不同：依 rank 決定接合方向
5. components 減 1
6. 回傳 true
```

### 為什麼兩次 find 要分成兩行

`find` 需要 `&mut self`。初學時應讓每次可變借用在一個 statement 內結束：

```rust
let root_x = self.find(x);
let root_y = self.find(y);
```

現在 `root_x`、`root_y` 都只是獨立的 `usize`，接下來可以安全讀寫 `self.rank` 和 `self.parent`。

不要急著把所有操作塞進一行。Rust 中將操作拆成小步驟，通常能同時改善可讀性與借用範圍。

### union by rank

假設兩棵樹的 root 不同：

```text
rank[root_x] < rank[root_y]   把 root_x 接到 root_y
rank[root_x] > rank[root_y]   把 root_y 接到 root_x
rank 相同                     任選一邊當新 root，並把新 root 的 rank 加 1
```

目標是把較矮的樹接到較高的樹，避免形成很長的鏈。

```text
較矮                 較高
  1                    4
 / \                  /|\
0   2                3 5 6

把 1 接到 4，而不是把 4 接到 1。
```

### components 何時減少

只有兩個不同 root 真的合併時：

```rust
self.components -= 1;
```

如果 `root_x == root_y`，它們本來就在同一集合，不能減少 `components`。

### rank 不是集合大小

`rank` 是樹高的估計，不等於元素數量。path compression 之後實際樹高可能下降，但不需要同步降低 rank；它仍足以協助選擇合併方向。

</details>

<details>
<summary><strong>D9. 讀懂 connected 與 components</strong></summary>

### `components`

```rust
pub fn components(&self) -> usize {
    self.components
}
```

- `&self`：只借來讀。
- `self.components`：讀取欄位。
- `usize` 是 Copy，所以回傳一份數字，不會移走欄位。
- 最後一行沒有分號，所以是回傳值。

### `connected`

```rust
pub fn connected(&mut self, x: usize, y: usize) -> bool {
    self.find(x) == self.find(y)
}
```

它比較兩個 root 是否相同。雖然語意上只是查詢，但 `find` 會進行 path compression，因此 `connected` 也必須使用 `&mut self`。

可以先把概念拆開理解成：

```rust
let root_x = self.find(x);
let root_y = self.find(y);
root_x == root_y
```

最後的比較式產生 `bool`，而且沒有分號，所以成為回傳值。

</details>

<details>
<summary><strong>D10. 如何閱讀 DSU 的三個測試</strong></summary>

測試不是只有驗證答案，也是在描述函式規格。

### 測試 1：基本合併與連通性

```rust
let mut dsu = Dsu::new(6);
assert!(dsu.union(0, 1));
assert!(dsu.union(1, 2));
```

`assert!(condition)` 要求條件是 `true`。這裡表示第一次合併不同集合時，`union` 必須回傳 `true`。

### 測試 2：重複合併不應改變集合數

```rust
assert!(!dsu.union(1, 0));
assert!(!dsu.union(2, 2));
```

`!` 是布林 NOT。這兩次都沒有真的合併，因此預期 `union` 回傳 `false`。

### 測試 3：確認 path compression

```rust
dsu.parent = vec![1, 2, 3, 4, 4];
assert_eq!(dsu.find(0), 4);
assert_eq!(dsu.parent, vec![4, 4, 4, 4, 4]);
```

這個測試不只要求找出 root `4`，還要求路徑上的所有 parent 都被壓縮。若只完成「找 root」而沒有第二趟修改，第一個 `assert_eq!` 會成功，第二個會失敗。

</details>

<details>
<summary><strong>D11. 建議實作順序與測試指令</strong></summary>

### 第一步：只實作 find

先把問題拆成兩趟：

```text
第一趟：從 x 找到 root，不修改 parent
第二趟：從 x 再走一次，把經過的位置改指向 root
最後：回傳 root
```

只執行 path compression 測試：

```powershell
cd D:\rust_pratice\ownership_xor_lab
cargo test dsu_find_compresses_the_path
```

### 第二步：實作 union

先分開取得兩個 root，再依序處理：

```text
相同 root
rank 左小於右
rank 左大於右
rank 相同
```

執行全部 DSU 測試：

```powershell
cargo test dsu
```

### 第三步：格式與完整回歸測試

```powershell
cargo fmt
cargo test
```

先看第一個編譯錯誤，不要同時追後面所有錯誤。後面的錯誤常常只是第一個錯誤連帶造成的。

</details>

<details>
<summary><strong>D12. 開始實作前的自我檢查</strong></summary>

能用自己的話回答以下問題，就可以開始 `find`：

- `Dsu` 是哪三個欄位的 owner？
- 為什麼 `parent` 的元素型別是 `usize`？
- `parent[x] == x` 為什麼表示 x 是 root？
- `0..size` 是否包含 `size`？
- `vec![0; size]` 與 `vec![0, size]` 有何差別？
- `Self`、`self`、`&self`、`&mut self` 分別是什麼？
- 為什麼 `find` 必須接收 `&mut self`？
- path compression 的第二趟為什麼要先保存 `next`？
- 為什麼 `find` 回傳 `usize`，而不是 `&usize`？
- 兩個 root 相同時，為什麼 `components` 不能減 1？
- rank 相同並合併後，哪一邊的 rank 要加 1？

不確定時回到對應小節，不需要一次背起來。實作時能知道去哪裡查，比背語法更重要。

</details>
