# iter_mutate / inplace_leetcode 設計取捨

對應程式碼:`reference/src/iter_mutate.rs`(六形狀)、`reference/src/inplace_leetcode.rs`
(五道高頻題)。drill 只有 `drills/src/iter_mutate.rs`,無 challenge 層——
這是慣用法肌肉,不是 45 分鐘的獨立題;它的考法是「藏在別題裡」。

## 為什麼借用檢查器擋你

`for x in &v { v.push(..) }` 擋的是 C++ 的 iterator invalidation:`push` 可能
realloc,`x` 變懸空指標——C++ 是 UB,Rust 把它提前到編譯期。代價是你要會
六種「合法地邊走邊改」的形狀,而不是跟編譯器吵架。

## 兩題決定用哪個工具

(1) 改的是值、還是結構(長度/容量)?(2) 需不需要同時碰多個元素?

| 需求 | 工具 | 為什麼不是別的 |
|---|---|---|
| 只改值,長度不變 | `iter_mut()` | 索引迴圈要自己管越界,還踩得到 usize underflow |
| 移除/搬移,單向掃 | 寫指標 two-pointer | 迴圈裡 `remove(i)` 是 O(n²) 且刪除後 index 全體錯位 |
| 篩選 + 改值,一趟 | `retain_mut` | `filter().collect()` 多 O(n) 空間;面試常指定 in place |
| 複雜條件刪(map/set) | 先收集 index/key 再第二趟動手 | 繞開「迭代中拿 `&mut` 容器」的借用衝突 |
| 從 `&mut self` 後面拿所有權 | `mem::take` / `mem::replace` | clone 貴;欄位換成 Default 值再重建 |
| 同時兩個 `&mut` 到不同位置 | `split_at_mut` | 借用檢查器無法證明 `i != j`;方法內部用 unsafe 擔保兩段不重疊 |

## usize underflow:貫穿所有 in-place 題的暗雷

`i - 1`(i==0)、`write - 2`(write<2):debug build panic、release build
**靜默 wrap 成 `usize::MAX`** 然後越界。邊界 guard 要在 dry-run 階段寫下,
不是等 panic。三個經典坑位:80 題的 `write - 2` 回看、75 題「從尾端換過來的
元素還沒看過、`i` 不能前進」、88 題的填寫方向。

## 為什麼反向填(88 Merge Sorted Array)

從前面填會蓋掉 `a` 還沒讀的段;反向填時「寫入位置永遠在讀取位置右邊」,
天然不覆蓋。這是「用方向選擇消滅一整類 bug」的示範——比「小心地從前面填」
高一個檔次的答案,面試值得講出聲。

## Production 對照

`retain` / `retain_mut` / `drain` 就是 std 把這些形狀 API 化的結果;
`slice::swap` 底層即 `split_at_mut` 思路。跨容器的「邊讀 A 邊寫 B」不在此範圍
——那沒有借用衝突,直接寫。

## 面試對映

這組不是獨立考題,而是寫其他題時的地基:ring buffer 的搬移、telemetry 聚合的
淘汰、registry 的 slot 回收,全都在做「邊迭代邊修改」。六形狀認不出來,
每一題都會在借用檢查器上卡五分鐘。
