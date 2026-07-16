# SCHEDULE.md — Etched TPS 衝刺(7/16 → 7/28)

容量:平日 5h / 週末 8h。排入約 45–50h,刻意留 slack——**寧可每天做完有餘,不要每天欠債**(R1 死因是疲勞)。

**每日鐵律**:收工前兩問——今天有打 code 嗎?有張嘴講英文嗎?兩個 yes 才算數。
**每場彩排 review 的打分順序**:pillar 1(clarify)永遠第一個打——那是你最弱的,每場彩排都是它的練習。
**原則:有彩排題覆蓋的 module,彩排就是它的 challenge,不重做。**(ring→a、pool→b、framer→c)

---

## 逐日

| 日期 | 內容(依序做) | 時數 |
|---|---|---|
| **7/16 四** | 卡#1(15m)→ `ring_buffer` drill + **手寫 wrap trace**(60m)→ Q5 aggregator 延伸,同檔(30m)→ **`iter_mutate` drill 7 洞**(75m,提前:它是後面所有東西的潤滑劑) | ~3.5h |
| **7/17 五** | 卡#2 → `thread_pool` drill 4 洞含 JobHandle(90m)→ `spsc_ring` drill + 逐 op 英文講 Ordering 理由(75m)。日讀:spsc artifact | ~3.5h |
| **7/18 六** | 卡#3 → ★`spsc` challenge 空白手搓 + diff + 跑 loom(90m)→ ★`executor` drill+challenge 含 park-token 口述 + Delay(120m)→ Q7 timer 接尾(20m)→ `hw_bridge` framer **drill**(45m;standalone challenge 砍掉,c 就是它的 challenge) | ~5h |
| **7/19 日** | 卡#4 → 🔴**a#1 ring_drop_oldest**(45m+review 30m,pillar1 先打分)→ 漏洞清單 → `fd_registry` artifact 讀 + drill 3 洞(90m,弱點提前)→ **spsc 空白 #1**(20m) | ~4.5h,晚上休 |
| **7/20 一** | 卡#5 → **修 a#1 的洞**(targeted,60–90m)→ 🔴**b#1 pool_graceful**(45+30m)。通勤:event_loop / mini_runtime 略讀(餵 executor×reactor 那句) | ~3.5h |
| **7/21 二** | 卡#6(最後一張新卡)→ 🔴**e2#1 fd_registry**(45+30m)→ 修 b#1 的洞(45m) | ~3h |
| **7/22 三** | 🔴**c#1 frame_parser_heartbeat**(45+30m)→ 修 e2#1 的洞(45m)→ **spsc 空白 #2**(20m) | ~3h |
| **7/23 四** | `signal_pipeline` drill 2 洞 + SeqCst store-buffering litmus 口述(90m,**最後一份新材料**)→ 🔴**a#2**(45+20m,驗收斂:同樣的洞有沒有復發)→ 修 c#1 的洞(30m) | ~3.5h |
| **7/24 五** | 🔴**e2#2**(45+20m)→ 🔴**d#1 tokio_frame_server**(45+20m,**只跑一遍**——「面試官說可用 crate」那條分支的保險;預設仍 std-only + 陳述假設)。日讀:P6(已排) | ~3h |
| **7/25 六** | 六張 clarify 卡**快打重來一輪**(40m)→ 🔴**c#2**(45+20m)→ 🔴**浮動 #3**:給兩遍都爆的那題(45+30m)→ deep-dive 口述錄音:ordering / Waker 鏈 / 光譜 / 選型 + executor×reactor + 五 server p99.9 讀→口述(120m) | ~5h |
| **7/26 日** | 🔴**b#2**(45+20m,累了這場先砍)→ recognition 級 e/f/g/h:讀題→30 秒定界→口述 arc(60m)→ 經驗故事 3 條寫成 bullet(40m)→ 英文句庫整份唸出聲(30m)→ **spsc 空白 #3**(20m,最後手熱檢查)→ 讀自己的 challenge code(60m) | ~4.5h,早睡 |
| **7/27 一** | **Taper。不碰新題(命令)。** 10 分鐘暖手 drill → 背時間預算(0-3/3-5/5-10/10-35/35-40/40-45)+ 五 pillar + 開場三句 → 檢查 CoderPad link / Meet / 耳機 / 水 → **早睡** | ≤1.5h |
| **7/28 二** | 8:00 暖手(小 drill 10m + pillar-5 清單 + 時間預算)→ **8:45–9:30 TPS** | — |

彩排間隔(同題 ≥3 天,近了是背答案):a 7/19→7/23|b 7/20→7/26|e2 7/21→7/24|c 7/22→7/25|d 7/24 一遍。
SPSC 空白 20 分鐘一次編過 ×3:**7/19 / 7/22 / 7/26**。

---

## 砍掉 / 降級(已裁,不用再想)

- **砍掉不練**:dsu、graph、trie、tree(doc 零訊號)
- **降級**:lru → 超前才寫|sharded_map → 讀 + 口述(跨 shard 鎖序用講的)|inplace_leetcode → 選配暖手,不進主線
- **deep-dive 清單 → 全部 post-TPS**,例外:event_loop/mini_runtime 略讀(7/20 通勤)、五 server p99.9(7/25,餵 trade-off 口述)
- framer standalone challenge → 砍(7/18 drill + 7/22 c#1 隔四天,才是真測試)

## 如果進度落後,砍的順序

① d(tokio 彩排)→ ② lru / sharded challenge + 全部次優先 → ③ signal_pipeline drill(litmus 口述保留)→ ④ b#2 → ⑤ a#2
**永不砍**:e2 兩遍、c 兩遍、spsc 空白 ×3、每日 clarify 卡、7/27 taper。
(排序邏輯:保你的弱點 e2 + 你的傷疤區 c/wrap,砍你已經最熟的 mutex/condvar 重複。)

## v8 對齊(2026-07-16 晚定,常駐規則)

- **P 編號已廢**(排程上),但 `html_p/` 的內容照用——它們有 repo 教材沒有的
  「面試追問鏈(≥3 層)+ Self-quiz」形式,當天讀完 artifact 後翻對應篇的
  追問鏈自測。日子對映:7/16→p7(ring 節)|7/17→p1(atomic/SPSC)|
  7/18→p2(executor)|7/20→p3(epoll)|7/22→p8(hw_bridge)|
  7/24→p6(telemetry,已排)|**7/25 口述底稿→p5-thread-safe-spectrum +
  rust-five-axis(這兩份 repo 沒有對應物,光譜與 Send/Sync 推導表只住在這)**。
- **7/16 產出欄**:drills/ring_buffer 綠 ✓|卡#1|drills/iter_mutate 綠|
  手寫 wrap trace 拍照|aggregator 延伸綠(**含「未來 ts 清 window」case**,
  規格照 rehearsals 題目 f 的 contract,寫在 ring_buffer 同檔)。
- **Overflow 池規則(每天適用)**:dsu / graph / trie / tree 只在三條件**全**成立時碰:
  ①當天產出欄全勾(含卡、含錄音)②明天沒欠債 ③還有力氣。
  優先序固定:spsc 空白加跑(20m)> 沒修完的彩排洞 > lru challenge >
  才輪到 dsu → graph → trie/tree,每個 timebox 25m。
  (這四題的家在 7/29 後的 coding rounds;graph 是 comfort-zone 陷阱。)
- **Google 舊 block**(277/158/588、LRU/LFU、segtree):無日期 → 每週最多 1–2 題
  維持手感;本週只挑 588 當暖手,其餘全停。

## Clarify 配方(最弱項的處方:高頻小塊,不開大 block)

- 每天 session 開場一張卡(15m 含對答案),7/17–7/22 六張走完;7/25 六張快打重來
- **五問決策表背到能默寫**(掉不掉→full policy→容量算式→shard→SLA→怎麼知道死了)——它就是你的 clarify 演算法,每張卡、每場彩排都跑它
- 每場彩排 review 第一個打分 = pillar 1

---

進度勾選:[PROGRESS.md](PROGRESS.md)(彩排計時表、clarify 卡紀錄都在那)。
