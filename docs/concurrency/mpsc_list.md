# mpsc_list 設計取捨

對應程式碼:`reference/src/concurrency/mpsc_list/`(`mod.rs` 教學殼 + `core_impl.rs` 演算法)。
前置閱讀:[mpmc_ring](mpmc_ring.md)(佔位/發布分家的第一課)。
互動教材:`html_p/mpsc-interleaving-stepper.html`(縫的逐步交錯)。

## 這是 tokio 的遠端 wake queue,不是玩具

runtime 的跨執行緒 wake 有一條硬需求:**wake 端絕不能被卡住**。
wake 發生在別人的執行緒(timer 執行緒、IO 執行緒、另一個 worker),
它不能等鎖、不能因為佇列滿而失敗、更不能丟事件(丟 wake = task 永眠)。
Vyukov intrusive MPSC 的 push 恰好兩步——一個 `swap` + 一個 `store`,
**wait-free、無失敗路徑**——這就是 tokio 選它收 remote wake 的原因。
unbounded 不是偷懶:記憶體帳轉嫁給上游(task 數有界 ⇒ 佇列自然有界)。

## push 的兩步,順序就是全部

```text
1. prev = tail.swap(node, AcqRel)   // 佔位:全世界立刻看到新 tail
   ────── 縫 ──────                  // prev.next 還是 null,鏈是斷的
2. (*prev).next.store(node, Release) // 發布:consumer 從此走得到
```

反過來(先接鏈再 swap)兩個 producer 會同時寫同一個 `prev.next`,
後者蓋掉前者、整條鏈斷——與 mpmc_ring 同一堂課:**多寫者必須先佔位**。

## 縫是顯式 API:`PopResult::Inconsistent`

consumer 走到 `next == null` 時有兩種世界:`tail == head`(真空)或
`tail != head`(有人佔了位還沒發布)。mpmc_ring 把這道縫藏進「空」的
判斷裡(dif<0 一律 None);這裡把它做成第三個回值,因為 caller
(runtime)知道怎麼處置:**yield 後重試**——producer 只差一個 store,
下一眼幾乎必然接上。這也是面試的展示點:同一道縫,兩種 API 哲學
(隱藏 vs 顯式),取決於 caller 有沒有能力做出比「當作空」更好的決策。

## 單 consumer 買到的兩件事

1. **head 無競爭**:pop 全程無 CAS,O(1) 純指標走。
2. **免 reclamation**:unbounded lock-free 的真 boss 是「誰能安全 free 節點」
   (epoch/hazard pointer 整套機器都為它存在)。這裡型別系統直接拆掉問題:
   `Consumer` 不可 Clone、pop 拿 `&mut self` ⇒ 只有一個釋放者;
   它只釋放「已越過」的節點,而越過 prev 的前提是看到 `prev.next` 非 null
   ——即 producer 對 prev 的最後一筆寫入之後。沒有執行緒會碰已 free 的記憶體,
   一行 hazard pointer 都不用寫。

反向推論:想要多 consumer?head 就有多寫者、釋放者就不唯一,
reclamation 問題原地復活——這就是「MPSC 比 MPMC 簡單一半」的那一半。

## stub 節點:用一個常駐節點換掉所有 null 分支

空佇列若 head/tail 都是 null,push 要同時設兩個指標——單一 swap 做不到,
被迫上 CAS 迴圈。stub 讓串列**永遠非空**:push 恆為「swap + 接鏈」,
pop 恆為「看 next」。被消費的節點值取走後原地變成新 stub——
節點的身份(stub/資料)是狀態,不是型別。

## 選型帳

| 對手 | 它贏在 | 它輸在 |
|---|---|---|
| mpmc_ring | push wait-free(無 CAS 重試)、unbounded 不丟 | 每 push 一次 heap 配置(~20–50ns);pop 有縫 |
| `Mutex<VecDeque>` | 高競爭 push(swap 一次 vs futex 排隊)、push 端絕不阻塞 | 低競爭時鎖版更簡單、連續記憶體 cache 更好 |
| crossbeam channel | 零依賴、機制透明(面試可手寫) | 生產環境請直接用 crossbeam/tokio 的 |

與 mpmc_ring 相同的誠實邊界:producer 卡在縫裡,consumer 拿不到後面
所有元素——lockless,不是正式 lock-free。

## 在本 repo 的位置

`mini_runtime` 的 run queue 是 `Mutex<VecDeque>`(冷路徑,正確的第一步);
本模組是它的 lock-free 升級選項——什麼時候值得換、runtime 其他元件怎麼換,
整張地圖見 `html_p/runtime-lockfree-upgrade-map.html`。
