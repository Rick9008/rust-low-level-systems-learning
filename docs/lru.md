# lru 設計取捨

對應程式碼:`reference/src/lru.rs`。相關:[tree](tree.md)(index-based vs Rc<RefCell> 的完整對照)。

## 為什麼一定是「HashMap + 雙向鏈表」

| 只用一個 | 缺什麼 |
|---|---|
| 只有 HashMap | 找 LRU 受害者要掃全表 O(n) |
| 只有鏈表 | 定位 key O(n) |
| HashMap + `VecDeque` 時間戳 | promote 要從 deque 中間拔元素 O(n) |

get/put 都要 O(1) ⇒ map 給 O(1) 定位、雙向鏈表給 O(1) unlink/push_front。
兩者透過「map 的 value = 鏈表節點位置」黏起來。

## Rust 特有的決策:鏈表怎麼表示

`Rc<RefCell<Node>>` 版:prev 得用 `Weak`(否則環狀 refcount 洩漏)、
每步 `.borrow_mut()` 執行期檢查、每節點獨立 heap alloc。
**index-based**(本實作):節點放 `Vec`,prev/next 是 `usize`——
借用檢查零阻力、locality 好、無 unsafe。這是 Rust 面試寫 LRU 的標準答案。

要點:`nodes` 只覆寫、不 remove ⇒ 索引永遠有效。淘汰時 `mem::replace`
原地換入新節點,舊 (K,V) 完整取出歸還 caller(淘汰 callback 的關注點分離)。

## key 存兩份

map 的 key + node 裡的 key(淘汰 tail 時要反查 map 刪條目)。
K: Clone 是最便宜的解;省 clone 的路(map key 用 `Rc<K>`、hashbrown raw entry)
複雜度都更高。面試直接聲明這個 trade-off 往前走。

## 哨兵 NIL = usize::MAX

`Option<usize>` 更「Rust」,但 prev/next 各多一個 discriminant、unlink 裡
四處 unwrap;NIL 讓 unlink 是四個平坦分支。代價:一條隱形規則(MAX 保留),
用 `debug_assert` 守。兩種都能過面試,重點是講得出取捨。

## 沒做的:remove()

remove 會在 `nodes` 留洞,需要 free list 管理空槽——這正是
[arena_lockfree](arena_lockfree.md) 的主題(free list + 世代標記)。
LeetCode 版 LRU 沒有 remove,先收斂範圍。

## Production 對照

`lru` crate(同構,索引版)、`hashlink`(LinkedHashMap)、
caffeine/moka(並發 + TinyLFU 準入策略,遠超面試範圍)。

## 互動教材

[artifacts/lru.html](artifacts/lru.html) —— HashMap 與 arena 並排、逐步同步:
`get` 命中時看 `prev`/`next` 欄位被逐格改寫(unlink + push_front),
`put` 滿載時看 tail 被淘汰、map 條目移除、arena 槽位原地回收。
「指標改寫(本次)」計數器把 O(1) 釘死:無論 cache 多大,每次操作都不超過 7 次寫。
