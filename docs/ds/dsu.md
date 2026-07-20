# dsu(union-find)設計取捨

對應程式碼:`reference/src/ds/dsu.rs`。

## 兩個優化各自擋什麼

- **union by rank**:不加時,`union(0,1); union(1,2); ...` 鏈狀合併把樹疊成
  高 O(n) 的竹竿,find 退化 O(n)。by rank 保證樹高 ≤ log n。
- **path compression**:find 沿路全部直掛根。單獨用攤銷 O(log n);
  與 by rank 疊加 → 攤銷 O(α(n)),α 是反 Ackermann 函數的逆,
  n = 10⁸⁰(可觀測宇宙原子數)時 α(n) = 4。實務上就是常數。

## 實作決策

- **find 用兩趟迭代,不用遞迴**:遞迴版
  `parent[x] = find(parent[x])` 只有一行,但深度 = 樹高,壓縮發生前
  第一次 find 一條 10⁶ 的鏈就爆 stack。兩趟迭代:第一趟找根、第二趟重掛。
- **rank 存 u8**:rank 只在同 rank 合併時 +1,樹高 ≥ 2^rank ⇒
  rank 到 255 需要 2^255 個元素。省 7/8 的 rank 記憶體。
- **by rank vs by size**:攤銷界相同。size 版附贈 O(1) 集合大小查詢
  (需求常見:「這個 partition 有幾台機器」);rank 版省空間。答得出差異即可。
- **components 計數器**:union 成功時遞減,O(1) 查詢集合數——
  比「掃一遍數根」O(n) 好,面試常見 follow-up。

## 壓縮後的形狀(可觀察)

find 過的路徑上,每個節點 parent 直指根(距根 ≤ 1 步)。
測試 `path_compression_flattens_chain` 直接驗證這個內部不變量——
不只測行為,也測「優化真的發生了」。

## 不支援的:un-union

拆開集合需要 rollback DSU(操作栈 + 不做壓縮)或 offline 重算。
面試先聲明「只合不拆」再往前走。

## 應用對映

Kruskal MST、網路連通性/partition 偵測、等價類合併、
percolation、accounts merge(LeetCode 721)。

## 互動教材

[artifacts/dsu.html](artifacts/dsu.html) —— 森林與 `parent[]` 並排,同步變化。
可以親手 `union` / `find`,看 `find` 爬到根之後第二趟把沿路節點逐格改寫成根;
union by rank 與 path compression 各自可開關,關掉 rank 就能建出退化鏈
(樹高 9、`find` 走 9 步),再按 `find` 看整條鏈被壓平成 1 步。
