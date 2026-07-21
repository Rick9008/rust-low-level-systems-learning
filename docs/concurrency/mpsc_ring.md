# mpsc_ring 設計取捨(Vyukov 單消費退化)

對應程式碼:`reference/src/concurrency/mpsc_ring/`(與 `mpmc_ring/core_impl.rs`
逐行對照著讀)。前置閱讀:[mpmc_ring](mpmc_ring.md) 的退化表——本模組就是
那張表「MPSC 那一列」的實體。定位:reference-only 讀物(drill 由 mpmc_ring
第 5 問覆蓋,不另出)。

## 看點一句話:head 連 atomic 都不是

單消費買到的不只是「pop 免 CAS」。Vyukov 協定裡 **producer 從不讀 head**
——滿的判定走槽位 seq(`dif < 0` = 上一圈沒消化)。所以單 consumer 下,
head 是消費端的**私有狀態**,只是剛好住在共享結構裡:本實作把它宣告成
`UnsafeCell<usize>`,不是 `AtomicUsize`。「資料被共享」與「資料被並發存取」
是兩件事——這個欄位是最小的實證,而 loom 的 UnsafeCell 存取追蹤就是裁判
(把 `Consumer` 改成 Clone、開兩個 consumer,loom 當場抓包)。

## producer 側為什麼一字不差

縫(佔位→發布)在生產側:兩個 producer 搶同一個 tail 的問題,跟 consumer
有幾個毫無關係。所以 CAS 取號 + per-slot seq 一個都省不掉,
`try_push` 與 mpmc_ring 逐行相同。這正是退化律的內容:
**哪端是「多」,哪端的機制原封不動;哪端是「單」,那端才有東西可拆。**

## 誠實邊界(繼承自 producer 側)

`try_pop` 回 None 只代表「沒有已發布的元素」——可能有 producer 佔了號
還沒發布。cap ≥ 2 的下限也原樣繼承(cap=1 三態塌縮)。

## 選型帳

| 需求 | 選擇 |
|---|---|
| 多 P 單 C + 硬容量上限(backpressure 內建、零配置) | **mpsc_ring** |
| 多 P 單 C + 絕不能擋住 push(wake 路徑) | [mpsc_list](mpsc_list.md)(unbounded、wait-free push) |
| 消費端也要多個 | [mpmc_ring](mpmc_ring.md) |
| 端點各一 | spsc_ring(效能上界) |

面試用法:寫完 SPSC 被問 "multiple producers?" → 升 Vyukov;
再被問 "still one consumer, can you simplify?" → 「pop 的 CAS 退回 plain
store,head 甚至退出共享——縫在 producer 側,seq 動不得」。
一來一回就是整張退化表。
