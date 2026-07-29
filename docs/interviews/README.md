# 面試紀錄(跨機器 memory)

這個資料夾是面試進度的**唯一真相來源**——Claude 的本機 memory 不跨機器,所有面試紀錄、回饋、下一階段計畫一律寫這裡,commit + push 後家機才看得到。

## 目前狀態(更新於 2026-07-29)

| 輪次 | 日期 | 結果 |
|---|---|---|
| R1 coding(DMA dispatcher) | 2026-07-28 | ✅ 過,feedback 正向 → [紀錄](2026-07-28-tps-round1-dma.md) |
| coding ×2 | 待定 | — |
| technical deep dive | 待定 | — |
| culture fit talk | 待定 | — |

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

deep dive / culture fit 的準備計畫等日期確定後排(culture fit 底稿:7/26 已寫的三條經驗故事)。
