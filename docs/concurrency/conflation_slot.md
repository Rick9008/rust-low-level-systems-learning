# conflation_slot —— per-key conflation(值層/通知層分離)

對應 `reference/src/concurrency/conflation_slot.rs`(drills/challenges 有練習版)。
**完整圖解 + 三個互動 stepper + naive→optimal 階梯:`html_p/conflation-slot-stepper.html`**
(值層/通知層結構圖、lost update 確定性重現、無鎖通知順序)——本文只放 repo 整合層
與家族判準,不重複那頁的內容。

## 一條 invariant

```
slot.queued == true ⟺ key ∈ ready
```

`queued` 的唯一存在理由是通知去重(沒有它,ready 長度 = O(事件數),tier 2 的
O(K) 成果被通知層吃掉)。三種破法 = 三種經典 bug,見 stepper 頁 tab ②③。

## 家族判準(7/31 問答收錄)

**同 key 的舊資料被結構性覆蓋/合併,且這是刻意的語意——才算 conflation 家族。**

| 結構 | 算嗎 | 為什麼 |
|---|---|---|
| timer wheel | ✗ | slot 是「桶」:按到期時間索引,timer 全收不丟,一個都要 fire。解決的是按時間查找,不是過載保最新。形狀像(固定格+索引),語意不同 |
| aggregate window | 半個(表親) | per-key 摺疊 ✓,但摺疊函數是可結合 merge(sum/max——保留全部事件的統計);conflation 是 merge 退化成 `f(old,new)=new` 的特例。觸發也不同:關窗即吐 vs consumer 拉 |
| sticky wake token(sim j)/ pending signal bit | ✓ 退化版 | 單 key、payload=unit 的 conflation:N 次 wake 摺成一個 token。epoll LT readiness、dirty-rect、LWW register 同族 |

## 路由(什麼時候能用)

consumer 只要「現在的值」+ payload 是絕對快照 + key 基數有界 → 用。
翻掉任何一條就換結構:delta payload → merge 版(走回 aggregate);
要完整事件序列(audit/replay)→ durable log;key 無界 → 先加 TTL/eviction。

## 兩條工程紀律(這個模組教的)

1. **「我要來拿了」與「我拿走了」之間不能有可觀測的中間態**——recv 的
   pop + 讀值 + 清旗標必須同一個 critical section,拆開就是 lost update
   (而且多執行緒隨機測抓不到,要確定性重現或 loom)。
2. **通知可以多,不可以少**——spurious 對 conflation 是冪等的(多讀一次同值);
   lost wakeup 是資料永久遺失。所有順序決定往「寧可多通知」倒:持鎖 notify、
   先清旗標再讀值、值先可見再宣告髒。

## 成本

publish O(1) amortized / recv O(1);空間 O(K) 值層 + O(K) 通知層,
與 update 速率完全脫鉤——這就是全部的價值。producer 永不阻塞的代價是
感受不到 consumer 慢:backpressure 訊號要另外走(count 是現成量表)。
