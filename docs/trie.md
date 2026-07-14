# trie 設計取捨

對應程式碼:`reference/src/trie.rs`。相關:[lru](lru.md)、[tree](tree.md)(同為 index-based 手法)。

## children 的三種存法

| 存法 | child 查找 | 每節點空間 | 適用 |
|---|---|---|---|
| **`[Option<usize>; 26]`(本實作)** | O(1) 陣列索引 | 208B 固定 | 字母表小且密(a-z) |
| `HashMap<char, usize>` | O(1) 期望 + hash 成本 | 按需 + HashMap 開銷 | 字母表大(Unicode) |
| 排序 `Vec<(char, usize)>` | O(log deg) 二分 | 最省 | 極稀疏、記憶體敏感 |

26 槽陣列在 a-z 場景是標準答案:查找零 hash 零分支預測失敗。
稀疏時它浪費(一個只有 1 個 child 的節點也佔 208B)——這是空間換時間的顯式選擇。

## arena 而非 `Box<Node>`

- 指標式:每節點一次 heap alloc;深 trie drop 時**遞迴析構**可能爆 stack。
- arena(`Vec<Node>` + usize child):配置攤平、locality、Drop 是釋放一個 Vec。
- 共享前綴不重複配置——測試用 `node_count()` 直接觀察
  (insert "app" 後 4 節點,再 insert "apple" 只 +2)。

## is_end:詞 vs 前綴之辨

`contains("appl") == false` 但 `starts_with("appl") == true`——
路徑存在只代表「有詞經過這裡」,is_end 才代表「有詞在這裡結束」。
這是 trie 面試第一坑,兩個查詢共用 `walk()`,差別只在最後一步看不看 is_end。

## 懶刪除

remove 只清 is_end,節點留在 arena。真回收需要:引用計數(每節點記
子樹詞數)或 free list——後者是 [arena_lockfree](arena_lockfree.md) 的主題。
記憶體換簡單性,對「詞典只增不減」的主流場景是零成本。

## 複雜度

insert / contains / starts_with / remove 都是 O(L),L = key 長度,
**與詞庫大小無關**——這是 trie 對 HashSet 的核心賣點(HashSet 的 hash 也是 O(L),
但做不了前綴查詢)。空間最壞 O(Σ L) 個節點 × 208B。

## Production 對照

`fst` crate(有限狀態轉換器,壓縮到極致的靜態詞典)、radix trie(路徑壓縮)、
路由表的 LPM(longest prefix match)。
