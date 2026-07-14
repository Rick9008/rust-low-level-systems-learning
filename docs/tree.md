# tree:index-based arena vs `Rc<RefCell>` 設計取捨

對應程式碼:`reference/src/tree.rs`(兩版並列,同一組測試)。

## 核心問題:Rust 為什麼寫鏈式結構「難」

單一所有權 + 借用檢查,和「節點互相指來指去」天生衝突。四條出路:

1. **index-based arena(首選)**:節點放 `Vec`,「指標」是 `usize`。
   所有權永遠屬於 Vec,索引只是資料——借用檢查完全不參與。
2. `Rc<RefCell>`:執行期共享 + 執行期借用檢查。能寫,但每一步都在付稅。
3. `Box` + `Option<Box<Node>>`:純樹(無共享、無回指)可用,take/replace 舞步多。
4. unsafe 裸指標:std 的 LinkedList 這麼寫;面試不要主動選這條。

## 逐點對照(本模組兩版實測)

| | arena | Rc<RefCell> |
|---|---|---|
| 配置 | 節點連續在 Vec,一次 realloc 攤平 | 每節點一次 heap alloc + 2 refcount 字 |
| 借用檢查 | 編譯期,零執行期成本 | 執行期:borrow 衝突 = **panic**(編譯器不救) |
| 讀取 API | `inorder() -> Vec<&T>` 借用可回傳 | 只能 `Vec<T>` clone——`Ref` guard 出不了函式 |
| parent 指標 | 加一個 `usize` 欄位即可 | 必須 `Weak`,忘了就 refcount 環 → 記憶體洩漏 |
| 刪除 | 留洞,需 free list / 世代標記回收 | drop 即回收 |
| 深樹 Drop | 釋放 Vec,迭代 O(n) | 遞迴析構,深鏈可能爆 stack |
| code 手感 | match + 索引,無 guard 舞步 | borrow 區間要手動縮小(見 insert 的註解) |

## 兩版程式碼裡最值得看的三處

1. `rc_refcell::insert`:borrow 區間刻意用 block 縮小,否則持著 `Ref`
   再 `borrow_mut` 同節點直接 panic——這是 RefCell 的日常陷阱。
2. `arena::inorder` 回傳 `Vec<&T>` vs `rc_refcell::inorder` 回傳 `Vec<T>`:
   同一個函式簽名差異直接體現「借用能不能逸出」。
3. 兩版的 `height` 都用層序迭代而非遞迴:遍歷手法與結構表示正交,
   但遞迴的 stack 風險兩邊都存在,統一用迭代。

## 面試建議

預設寫 arena 版;被問到「如果節點要被多個 owner 共享呢」再談 Rc(共享)
+ RefCell(內部可變性)+ Weak(斷環),並主動指出上表的稅。
能並排講清楚,比只會寫其中一版強一個檔次。

相關:[lru](lru.md)(index-based 鏈表實戰)、[arena_lockfree](arena_lockfree.md)(arena 槽位回收 + 世代)。

## 互動教材

[artifacts/tree.html](artifacts/tree.html) —— arena 版與 `Rc<RefCell>` 版並排,同一組 insert /
inorder / drop 同時打在兩邊:左邊寫 `usize` 進 `Vec`,右邊每一步都在動 refcount
(n 個節點的 inorder = 2n 次讀改寫 + n 次 `T::clone`,計數器實時顯示)。
把 parent link 從 `Weak` 切成 `Rc` 再 `drop(tree)`:root 的 strong 從 3 掉到 2 就停住,
`Drop` 一次都沒跑,整棵樹洩漏——切回 `Weak` 才一路歸零,arena 版兩種情況都是一次 `free`。
第三個實驗:持著 `Ref` 再 `borrow_mut()` → 執行期 `already borrowed: BorrowMutError`,
而 arena 版的同一個錯誤是 `error[E0502]`,程式根本不會被產出。
