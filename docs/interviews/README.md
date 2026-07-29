# 面試紀錄(跨機器 memory)

這個資料夾是面試進度的**唯一真相來源**——Claude 的本機 memory 不跨機器,所有面試紀錄、回饋、下一階段計畫一律寫這裡,commit + push 後家機才看得到。

## 目前狀態(更新於 2026-07-29)

| 輪次 | 日期 | 結果 |
|---|---|---|
| R1 coding(DMA dispatcher) | 2026-07-28 | ✅ 過,feedback 正向 → [紀錄](2026-07-28-tps-round1-dma.md) |
| coding #2 | **建議 8/11(一)早上**(日期自選,待敲) | — |
| coding #3 | **建議 8/12(二)早上**(連兩天=一次早起梯度打兩場) | — |
| technical deep dive(履歷/過去專案) | 排 coding 之後 | — |
| culture fit talk | 排 coding 之後 | — |

R1 前的衝刺計畫在 `../../SCHEDULE.md`(7/16→7/28,已結案)。

## 下一階段練習方向:spec-heavy 新題型

R1 實測的題型:**長英文 spec(有洞)+ 一堆 provided API + 實作一個 fn,面試官是唯一 oracle**。
與 a–h 彩排的差別:重心從「手搓結構」移到「clarify 出隱藏 spec」+「多重 state 的 event loop 骨架」。R1 證明 clarify 已過線(面試官點名稱讚);真正的洞是**時間不夠 → code 有漏洞**,要練的是「40 分鐘內把 spec 轉成 state 表 + 骨架」的節奏。

練法:Claude 出題(英文 spec 故意埋洞 + 可編譯的 fake API harness),live clarify(打字來回,白天可做)→ 計時 45m 寫 → review 20m。

候選模擬題(JD 軸:telemetry / 硬體訊號 / event loop / lockless):

| # | 題 | 練什麼 |
|---|---|---|
| i | DMA dispatcher v2(R1 重做 + pipeline 多 request) | per-request state、done 路由、亂序 |
| j | ISR → bottom-half pipeline | ISR 限制(不能 alloc/block)、SPSC 交棒、overflow policy |
| k | 多核 ISR / per-CPU queue fan-in | 多核多緒 race 避免、MPSC、聚合 |
| l | MMIO command queue(doorbell + completion ring) | head/tail 座標系實戰、polling vs IRQ |
| m | engine watchdog / timeout | event loop 的第三種 state:時間(deadline、retry/reroute) |

## 逐日計畫(7/30 → 8/12,總量 ~14h,照 v9 時間模型)

原則:白天打字場 = 模擬題(clarify 打字來回不用出聲);晚上出聲場 = deep dive 口述 + culture fit 唸稿,每晚只排一件出聲事。

| 日期 | 白天(公司,~90m) | 晚上(23:30–02:00,出聲) |
|---|---|---|
| 7/30 三 | 🔴 i:DMA v2(clarify 20 + 寫 45 + review 20) | culture fit 稿:寫自我介紹 60–90s + why-us(打字也可移白天) |
| 7/31 四 | 🔴 j:ISR → bottom-half | deep dive 口述 #1:專案一(問題→限制→設計→trade-off→數字) |
| 8/1 五 | 🔴 k:多核 per-CPU fan-in | culture fit 稿出聲唸 #1 + Claude 模擬追問 |
| 8/2 六 | 🔴 l:MMIO command queue + deep dive 專案二寫底稿(週末 8h) | deep dive 口述 #2 |
| 8/3 日 | 🔴 m:engine watchdog + culture fit 三條故事(7/26)改英文稿 | 休或補滑帳 |
| 8/4 一 | i–m review 掃洞,挑最弱兩題 | deep dive 口述 #3(最弱的那個專案重講) |
| 8/5 二 | 🔴 重打 weakest #1 | culture fit 出聲唸 #2(conflict/failure 兩題) |
| 8/6 三 | 修洞 + 輕量 | (起床梯度開始:每天提早 30m) |
| 8/7 四 | 輕量:骨架默寫抽查 | deep dive 全串口述一遍(15m/專案) |
| 8/8 五 | 🔴 重打 weakest #2 | culture fit 全串一遍 |
| 8/9 六 | 回憶掃描(不碰新題) | 08:15 起 |
| 8/10 日 | taper:檢查表 + 時間預算背誦 + 早上動線彩排 | 07:30 起,00:00 熄燈 |
| **8/11 一** | **coding #2(早上)** | 當天紀錄寫進本資料夾 |
| **8/12 二** | **coding #3(早上)** | 同上 |

culture fit 英文稿範圍:自我介紹 60–90s、why this company、conflict、failure、proudest project、想問他們的 3 個問題;底稿 = 7/26 的三條經驗故事。稿子檔案:`culture-fit-script.md`(寫完放本資料夾)。

閥門:白天被正職吃掉 → 砍當天晚上 culture fit(deep dive 不砍);模擬題超時 = 挖到洞,記洞不記違規。
