# 面試紀錄(跨機器 memory)

這個資料夾是面試進度的**唯一真相來源**——Claude 的本機 memory 不跨機器,所有面試紀錄、回饋、下一階段計畫一律寫這裡,commit + push 後家機才看得到。

## 目前狀態(更新於 2026-07-29)

| 輪次 | 日期 | 結果 |
|---|---|---|
| R1 coding(DMA dispatcher) | 2026-07-28 | ✅ 過,feedback 正向 → [紀錄](2026-07-28-tps-round1-dma.md) |
| coding #2 | **8/6(四)09:15–09:45 開場**(待 coordinator 確認) | — |
| coding #3 | **8/11(二)09:15–09:45 開場**(同上) | — |
| technical deep dive(履歷/過去專案) | **8/12(三)09:15–09:45 開場**(同上) | — |

Onsite 結構(2026-07-29 邀請信):3×45m(2 coding + 1 deep dive)+ 最後 15m recruiter debrief;**culture fit 沒有獨立場**——散在各場 behavioral 提問 + debrief,culture fit 稿的用途 = 每場開頭自介 + debrief 15 分。拆三天 + 台北早上時段已去信要求,實際時段以 Ashby 確認為準。

R1 前的衝刺計畫在 `../../SCHEDULE.md`(7/16→7/28,已結案)。

## 下一階段練習方向:spec-heavy 新題型

R1 實測的題型:**長英文 spec(有洞)+ 一堆 provided API + 實作一個 fn,面試官是唯一 oracle**。
與 a–h 彩排的差別:重心從「手搓結構」移到「clarify 出隱藏 spec」+「多重 state 的 event loop 骨架」。R1 證明 clarify 已過線(面試官點名稱讚);真正的洞是**時間不夠 → code 有漏洞**,要練的是「40 分鐘內把 spec 轉成 state 表 + 骨架」的節奏。

練法:題本 [sim-problems.md](sim-problems.md)——每題 **Phase 1 開場給、Phase 2 面試官驗收後才放**(照 R1 的漸進節奏);面試官手冊 `sim-interviewer-guide.md` **跑題中不准開**。clarify 用英文打字來回(句庫:[clarify-phrasebook-en.md](clarify-phrasebook-en.md),含聽力修復句;每晚出聲場挑一類唸 3 遍)→ 計時 45m → review 20m。

**Harness 已全部入庫(跨機器直接執行)**:`rehearsals/src/sim_{i,j,k,l,m,n}_*.rs`——**介面與 requirement 一律英文**(面試全英文,讀英文就是練習;中文對照:[sim-problems-zh.md](sim-problems-zh.md))。上半「題目給的介面」可讀,**mock/SimBus 實作區與 `rehearsals/tests/sim_*_test.rs` 跑題前不准讀**(oracle 會在你違反協定時當場 panic 開燈)。執行:作答區填實作+檔尾寫自己的測試 → `cargo test -p rehearsals sim_<x>` → 跑完拔參考測試的 `#[ignore]` 對邊界 → 開 sol 對照。**六份 sol 全備**(`rehearsals/examples/sol_sim_*.rs`,寫完才開;19/19 參考測試已用解答驗過):i 的兼 7/29 閱讀材料(`cargo run -p rehearsals --example sol_sim_i_dma`),m/n 檔頭 8/4 預讀。

候選模擬題(JD 軸:telemetry / 硬體訊號 / event loop / lockless):

**全場題(45m 計時)五題**:

| # | 題 | 練什麼 |
|---|---|---|
| i(→lite 30m) | DMA dispatcher v2(R1 修洞重做,**降級不全場**——DMA 域考過了,但 pipeline 路由的洞要親手關) | per-request state、done 路由、cancel |
| j | ISR → bottom-half pipeline | ISR 限制(不能 alloc/block)、SPSC 交棒、overflow policy |
| k | 多核 ISR / per-CPU queue fan-in | 多核多緒 race 避免、MPSC、聚合 |
| l | MMIO command queue(doorbell + completion ring) | descriptor→barrier→doorbell 鐵律、head/tail 對硬體、亂序 completion tag |
| m | engine watchdog / timeout(R1 延伸) | event loop 的第三種 state:時間;idempotency 決定敢不敢 retry |
| n(已備) | priority job scheduler + dependency DAG(材料全備好;8/6 若情報指向他型再另備/換皮,已備不浪費) | BinaryHeap+seq 破平手、indegree 入場閘、priority inversion |

**廣度用認題卡補(15–20m/張,不寫完整 code)**:[spec-cards.md](spec-cards.md) 七張,每張=讀埋洞英文 spec → 寫 ≥3 clarify 問題 → 30 秒英文定界出聲 → state 表 → 開[答案鍵](spec-cards-answers.md)(含英文稿)。SP/FP/FR/TA/TQ 五張是複習卡(衝刺期已有 a–h 肌肉:signal_pipeline/c/e2/f/h);HW-L/HW-M 兩張當 l/m 全場的 10 分鐘前導。

## 逐日計畫(7/30 → 8/12;練習時間比衝刺期少,量已壓)

原則:白天打字場 = 模擬題(clarify 打字來回不用出聲);晚上出聲場 = deep dive 口述 + culture fit 唸稿,每晚至多一件出聲事,標「(選)」的累了就砍。signal pipeline 由 j 題 + 複讀覆蓋,不另開項目。

| 日期 | 白天(公司,~90m 上限) | 晚上(出聲) |
|---|---|---|
| 7/29 三 | **休息日(輕活)**:讀 `sol_sim_i_dma.rs`(三張表 + Dry-Run 紙上走) | deep dive / culture fit 英文稿起草(打字,輕;材料先貼給 Claude) |
| 7/30 四 | i-lite:R1 修洞重做 30m(DMA 域考過,不值全場;直接寫 pipeline+cancel 版關掉 R1 的洞,不跑 clarify 全套)+ 卡 SP | culture fit 自介稿 60–90s + why-us(接 7/29 草稿) |
| 7/31 五 | 🔴 j:ISR → bottom-half(= signal_pipeline 的 spec-heavy 版)+ signal_pipeline 頁複讀 30m | deep dive 口述 #1:專案一(問題→限制→設計→trade-off→數字) |
| 8/1 六 | 🔴 k:多核 per-CPU fan-in + 卡 FP(週末塊) | culture fit 唸 #1 + 模擬追問 |
| 8/2 日 | 🔴 l:MMIO command queue(HW-L 卡當 10m 前導)+ culture fit 三條故事(7/26)改英文稿 | deep dive 口述 #2 |
| 8/3 一 | 認題卡 FR/TA/TQ + 骨架默寫抽查 15m | (選)clarify 句庫一類唸 3 遍 |
| 8/4 二 | 輕:i–l 洞複掃 + **sol_m / sol_n 檔頭預讀**(還沒跑的兩題先拿「怎麼說」,計時寫 8/8、8/9 照跑) | **08:30 起(梯度開始;9:15 場只需 07:45 起,比 R1 的 8:45 場輕)** |
| 8/5 三 | taper:不碰新題、檢查表 + 時間預算 + **六題 30 秒英文定界唸一輪**(sol 檔頭 + spec-cards-answers) | 08:00 起、00:30 熄燈 |
| **8/6 四** | **coding #2(09:15)**,07:45 起 | 當天紀錄入庫 + 洞清單 |
| 8/7 五 | 修 #2 暴露的洞(targeted) | deep dive 口述 #3 |
| 8/8 六 | 🔴 m:engine watchdog(HW-M 卡前導;吃 #2 暴露的方向) | culture fit 全串 |
| 8/9 日 | 🔴 n:priority scheduler + DAG 全場(已備;8/6 若指向他型,當天換皮或改 a–h 重打) | deep dive 全串(15m/專案) |
| 8/10 一 | taper | 早睡(8/6 後不回彈,整段維持 ≤08:30 起) |
| **8/11 二** | **coding #3(09:15)**,07:45 起 | 輕:deep dive 全串最後一遍(15m/專案,材料 8/9 前已備齊)|
| **8/12 三** | **deep dive + culture fit(09:15)**,07:45 起 | 收帳:全程紀錄入庫 |

culture fit 英文稿範圍:自我介紹 60–90s、why this company、conflict、failure、proudest project、想問他們的 3 個問題;底稿 = 7/26 的三條經驗故事。稿子檔案:`culture-fit-script.md`(寫完放本資料夾)。

閥門(時間不夠的砍序):認題卡 →(8/9)重打 → k;**i、j、l、m、兩段 taper、早起梯度不砍**。模擬題超時 = 挖到洞,記洞不記違規。
