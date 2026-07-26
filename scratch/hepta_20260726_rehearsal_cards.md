# Hepta 卡草稿——7/26 三場計時彩排沉澱(6 卡)

> 目標白板:「Rust Low Level Notes」。上板後把卡片 ID 記回本檔各卡標題下。
> 讀法:每張 1 分鐘——當時錯什麼、修了什麼、一句處方。

## 卡1【Notes】bounded_channel 的三課(g#1 首跑)

<!-- ID: -->

**錯什麼**:oracle 4 紅同根——`recv` 裡兩個 `sender_cnt==0 → return None` early return,蓋過「佇列還有貨」;最後一個 sender 掉了、buffer 有貨,recv 回 None 丟資料。合約(drain 完才 None)在骨架註解+clarify 答覆講過**兩次**仍掉。

**修什麼**:early return 全刪。`wait_while(empty && senders>0)` 醒來之後,**佇列自己就是答案**——`pop_front()` 回傳的 `Option` 天然編碼了「有貨=Some / 空(只可能因 cnt==0)=None」。

**處方**:
1. 等待條件寫進 predicate 的語意,醒來後**不要再用旁路變數改判**。
2. `SendError(T)` 的存在理由:receiver 死了要**把貨原封還你**——API 簽名即合約。
3. clarify 答案到手要複誦回去:"so recv drains, then None — noted."

## 卡2【Notes】沒 join 的斷言不是斷言(g#1 空測試)

<!-- ID: -->

**錯什麼**:`boundary_test` spawn 後 handle 直接丟。assert 活在孤兒執行緒:panic 被 harness 吞、test fn 秒退、process 結束時孤兒被殺——**它 10/10 綠是因為它什麼都沒驗**。更狠:它想驗的場景(sender 卡滿→receiver drop→Err)在當時的 code 是真死鎖,被沒 join 完美遮住。

**處方**:測試裡每個 `spawn` 必接 `handle.join().unwrap()`;斷言的排程依賴(誰先誰後)用 join 建 happens-before,不靠運氣。**「綠」的證據力 = 斷言真的被執行過。**

## 卡3【Notes】驗牌經濟學:lazy validation(f#1 far-jump 鬼資料)

<!-- ID: -->

**錯什麼**:大跳窗(未來 ts 前進多格)後,查被跳過的 window 回了 `Some`——slot 裡躺著同餘舊 epoch 的資料(6%2 == 2%2 撞 slot 0,鬼=epoch 2 的 sum)。同餘鬼資料家族第 3 現身。

**修什麼**:query 補 `bucket.epoch != asked_epoch → None`。**桶不是你的,牌對了才是你的**——record 落桶驗牌重置、query 讀桶驗牌拒答,兩扇門缺一鬼就進。

**Trade-off 一句**:lazy validation(掛 epoch 牌)讓 record **嚴格 O(1)**(跳窗零清掃),代價=每次存取多一個比較;eager 清掃版最壞掃 `min(jump, num_windows)` 桶。lazy 勝。

**驗證法**:回放——把驗牌條件改壞→自寫紅測要咬住(鬼現形)→還原綠。

## 卡4【Notes】漏讀家族三案與警報器

<!-- ID: -->

同一族三案,死法都是「需求在紙上,code 裡沒有」:

1. **d#1**:idle_timeout 整條蒸發——clarify 沒問到的需求恰是掉的需求。
2. **g#1**:drain 合約答了還掉——問到了、答了、還是掉。
3. **e 快寫**:`handler_count(&self, id)` 回了全域計數——**簽名裡的參數沒被用到 = 漏讀警報器**(它還會以 unused warning 的形式自己叫)。

**處方**:動筆前 30 秒,clarify 清單對讀需求清單;寫完每個 fn 掃一眼簽名——每個參數都該有下落。

## 卡5【Memory/口述】Trade-off 收尾三拍公式(7/26 定版)

<!-- ID: -->

40 分鐘那格站起來,30–45 秒,三拍:

1. **價格**:我的方案值多少——Big-O **每個字母當場指認**("Feed is O(m+p) — m new bytes copied in, p payload bytes completed this call")。
2. **沒走的路 ≥2 條,每條用「軸」開頭**:面試官聽的是軸不是功能——"Copy vs complexity: VecDeque kills the compaction copy, but the 4-byte header needs contiguous bytes…"、"Allocation vs lifetime coupling: borrowed slices are zero-copy but tie frame lifetimes to my buffer…"。
3. **有效範圍**:自己劃邊界,搶在追問前——"This assumes a trusted peer. Untrusted, I'd checked_add and cap len."

三拍對面試官心裡三個問句:how much / why this / when not。靈魂字:**measured**(零拷貝那張牌等 profiler 量到再打)。

## 卡6【Notes】head/tail 座標系:list 家族 vs ring 家族

<!-- ID: -->

**現象**:spsc 空白 #3 整檔把 head/tail 對調(head=寫游標)——內部自洽所以零 bug,但這是「交換位子」干擾第 5 現身。

**兩套座標系都是真的**:
- **Vyukov node-based list**(mpsc_list):producer `XCHG(&head, n)` 掛新節點——**head=寫入端**。
- **ring / FIFO 教科書系**(本 repo reference、Michael-Scott、`VecDeque` push_back/pop_front):隊尾加入隊首服務——**tail=寫入端**。

**處方**:同一家族固定一套(ring 跟 reference 的 tail-write);上場開寫前一句話釘死("head = next to consume, tail = next to write"),或乾脆 `read_idx`/`write_idx`(kfifo 的 in/out、folly 同款)——零歧義還省一次 clarify。
