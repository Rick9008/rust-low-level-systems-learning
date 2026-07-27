# 練習題:Test Scheduler(toposort 快寫,7/27 晚,非計時/無 oracle/40m 硬上限)

> 來源:coffee chat 情報——「很多個 test 要決定執行順序」。寫在 `scratch/toposort_20260727.rs`,
> 編過 + 自寫 smoke 即收工。卡住 → 跟 Claude 講到哪卡住,口述解完就好,不硬寫。

## Prompt(interview 風格,英文照讀)

You are building a test runner. Each test has a unique name and may declare
dependencies: `("integration_db", ["unit_db", "unit_config"])` means
`integration_db` must run **after** both dependencies. Given a list of
`(name, deps)` pairs, return a valid execution order.

- If multiple tests are runnable, any valid order is fine (clarify point!).
- If the dependencies contain a cycle, report it as an error.

## 建議 API(簽名自訂,這只是錨)

```rust
fn schedule(tests: &[(String, Vec<String>)]) -> Result<Vec<String>, CycleError>;
```

## Clarify points(動筆前自答,場上要問出聲)

1. dep 指向不存在的 test —— 報錯還是忽略?(自訂,講出來就好)
2. 順序不唯一時要不要決定性(deterministic)輸出?—— 追問層:ready 佇列換 `BinaryHeap` 就有字典序/優先權,O(V log V)。
3. 規模?——V=tests、E=deps,Kahn 是 O(V+E),千級 test 毫無壓力。

## 唯一需要的 idiom(解掉「Rust 寫 graph」恐懼的那把鑰匙)

**不要造節點結構、不要存參照。**兩張表 + 一條佇列,全部擁有型資料:

- `HashMap<String, Vec<String>>`:正向鄰接(誰完成後解鎖誰)
- `HashMap<String, usize>`:每個 test 還剩幾個沒跑完的 dep(in-degree)
- `VecDeque<String>`:in-degree == 0 的 ready 佇列

演算法(Kahn):pop ready → 推進結果 → 它解鎖的每個人 in-degree −1 → 歸零者入佇列。
**結果長度 < test 總數 ⇒ 有環**(剩下的人互相等)。

## 自寫 smoke(手算得出答案的兩三組)

1. 線性:c←b←a → `[a, b, c]`
2. 菱形:d 依賴 b,c;b,c 依賴 a → a 開頭、d 結尾、bc 順序自由
3. 環:a↔b → Err

## 口述追問預備(寫完講 3 句,不寫 code)

- 環的**內容**怎麼回報?(剩餘未輸出的節點集合就是環+被環堵住的人)
- test 之間可平行:同一波 in-degree 歸零的就是**可平行的一批**——輸出 `Vec<Vec<String>>`(levels)只差三行。追問「怎麼跑」就接 thread pool(b 題)——**兩題在這裡握手**。
- 增量重跑(改了一個 test 只重跑下游)= 反向鄰接 BFS。
