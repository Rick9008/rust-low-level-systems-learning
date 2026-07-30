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
**轉製層已入庫(2026-07-30)**:六題各有 reference 教學版(5-pillar 檔頭 + mock + 手 trace 測試)
+ drills 填空版(挖核心轉移函式),對照表與複打流程在 `rehearsals/README.md` 進度狀態表下方;
各檔頂防雷 banner 標開放日期——**該題計時場跑完才開**,跑完把它轉入 drills 複打當固化練習。
**圖解鏡像(2026-07-31)**:`html_p/r2-sim-walkthroughs.html`(claude.ai 鏡像在 index 頂部 R2 備戰區,🔬)
——六題各一分頁:provided API 逐函式解剖表、全景示意圖(資料流 + 區域著色 + 怎麼讀)、互動 stepper、
code 走讀、陷阱清單;**每分頁掛同款防雷門**(i 已開,j/k/l/m/n 各自計時場跑完才開),賽前仍讀作戰本前導卡。

**圖解作戰本**:`html_p/r2-onsite-visual-guide.html`(claude.ai 鏡像在 artifacts gallery,index 已收卡)——題型解剖(API 三堆分類法 + reactor 五步骨架)、sim i 三張表互動 stepper(pipeline 分水嶺劇本)、sim j–n 概念前導與 clarify 種子(刻意不含解法,跑題前讀是安全的)、英文對白道場五場景(雙語,R1 洞①的處方)。用法:各 sim 開跑前 10 分鐘讀對應前導卡;對白每晚出聲場挑一景唸 3 遍。

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
| 7/30 四 | ✅ 實績:i-lite 40m(**drills 填空版** 6/6 綠——R1 兩洞在 drill 層關掉;空白重寫層的驗證留給 8/8 的 sim m,它是 R1 的直系延伸)+ 卡 SP(超時但值;批改入 spec-cards.md;兩課:freshness vs completeness 是 clarify 問題、API 層/primitive 層分層)+ 量級 reps v3 批改入檔(R1 掉零重寫/R2 補一行/R3 忘 timeout;荒謬檢查 0/3)+ 除膜手冊上線 `html_p/capacity-four-shapes.html`(四形狀+單位攜帶+名詞卡+錨點庫;7/30 晚應用戶要求補「四形狀英文對白」節——每形狀配面試官認題句+敘算句+數字唸法卡,晚上出聲場每晚挑一形狀唸 3 遍) | 唸稿:自介 60–90s + why-us + 補收 reps 殘帳(R1 數字鏈/R3 重算,5m)。**live spec-heavy 移 7/31 白天**(7/30 晚體力不足當晚拍板;它本來就是加開項,砍序第一位) |
| 7/31 五 | 開機:**卡 SP 重做**(15–20m;針對 7/30 批改的三洞——drop-oldest 被 API 封死、口頭需求掉了計數、scope creep——加「assume 槽」「state 表持有權」兩個待修項,整張重跑驗收;開錶前先圈名詞)→ 🔴 j:ISR → bottom-half(開跑前 10 分鐘讀作戰本 j 前導卡即可;原排的 signal_pipeline 頁複讀已被 7/30 卡 SP 深討論覆蓋,砍)→ 下午:**live spec-heavy(自 7/30 晚移入)**——Claude 出全新題當面試官、不碰 j–n 域、開錶前圈名詞全部問完才計時;**建議跑 25m「clarify-only」版**:只跑到 state 表為止不寫實作,因為寫作段當天已由 sim j 練過,這場要練的是前 15 分鐘的節奏(圈名詞→clarify→定界→state 表)。**當日閥門:正職吃掉時間就先砍 live spec-heavy(滑 8/1 白天);卡 SP 重做與 sim j 不砍** | deep dive 口述 #1:**專案二(logging daemon)**——7/31 修正:照 deep-dive 稿自己的建議「開場順序二→一→三、專案二對 Etched 命中率最高」,先練它(原寫專案一,與唸稿頁排程矛盾,以此為準)|
| 8/1 六 | 🔴 k:多核 per-CPU fan-in + 卡 FP(週末塊) | culture fit 唸 #1 + 模擬追問 |
| 8/2 日 | 🔴 l:MMIO command queue(HW-L 卡當 10m 前導)+ culture fit 三條故事(7/26)改英文稿 | deep dive 口述 #2:**專案一(Active-Active HA)** |
| 8/3 一 | **sim m+n 認題輪**(7/29 加,用戶點名「上場前不能完全沒看過」:各 ~20m,Phase 1 冷讀 → ≥3 clarify 問 → 30 秒英文定界出聲 → state 表手寫;**m 加 15m 非計時骨架快寫**——R1 直系延伸、#2 命中率最高;全程不開 oracle/sol/參考測試,8/8–8/9 計時場保留改當執行練)+ 骨架默寫抽查 15m;認題卡 FR/TA/TQ 降選配讓位 | (選)clarify 句庫一類唸 3 遍 |
| 8/4 二 | 輕:i–l 洞複掃 + **sol_m / sol_n 檔頭預讀**(還沒跑的兩題先拿「怎麼說」,計時寫 8/8、8/9 照跑) | **08:30 起(梯度開始;9:15 場只需 07:45 起,比 R1 的 8:45 場輕)** |
| 8/5 三 | taper:不碰新題、檢查表 + 時間預算 + **六題 30 秒英文定界唸一輪**(sol 檔頭 + spec-cards-answers) | 08:00 起、00:30 熄燈 |
| **8/6 四** | **coding #2(09:15)**,07:45 起 | 當天紀錄入庫 + 洞清單 |
| 8/7 五 | 修 #2 暴露的洞(targeted) | deep dive 口述 #3:**專案三(Rust/Tokio)** |
| 8/8 六 | 🔴 m:engine watchdog(HW-M 卡前導;吃 #2 暴露的方向) | culture fit 全串 |
| 8/9 日 | 🔴 n:priority scheduler + DAG 全場(已備;8/6 若指向他型,當天換皮或改 a–h 重打) | deep dive 全串(15m/專案) |
| 8/10 一 | taper | 早睡(8/6 後不回彈,整段維持 ≤08:30 起) |
| **8/11 二** | **coding #3(09:15)**,07:45 起 | 輕:deep dive 全串最後一遍(15m/專案,材料 8/9 前已備齊)|
| **8/12 三** | **deep dive + culture fit(09:15)**,07:45 起 | 收帳:全程紀錄入庫 |

culture fit 英文稿範圍:自我介紹 60–90s、why this company、conflict、failure、proudest project、想問他們的 3 個問題;底稿 = 7/26 的三條經驗故事。稿子檔案:`culture-fit-script.md`(寫完放本資料夾)。

**稿件狀態(7/29 晚,二輪定稿)**:deep dive 三專案 + culture fit 五題 ★ 已起草並按 Withers 回填的五個確認項修正(PG 14/收信不能停/內建不一致偵測計數 0/CPU-spin=前人 bug 只當 debugging 故事/logging daemon 誠實架構 = syslog-ng 尾端寫 DB);個人 failure 改用 **authz/authn 缺失 → CVSS 10.0** 故事。素材 = 履歷 + Holdwin 簡報轉軸,交易對照句已拿掉;唸稿鏡像頁 `html_p/r2-interview-scripts.html`(claude.ai index R2 備戰區有卡)。**仍待補**:conflict 故事的實際人物場景、專案三最難的 bug。
**7/31 三輪(逐字化)**:culture fit ★1/★2/★3/★6/★7 全數升級為**全英文逐字稿**(骨架版實測唸不出來——句間中文接縫卡嘴;中文降級為「唸法註」不出聲);★3 織入 Jon Gjengset 40-hours 論點(含使用守則:上場前重讀原文、被追問講論點不掰細節、拼命文化訊號強就把人名句縮成 "I optimize for sustained throughput, not heroics")。deep dive 專案二全段口述化(今晚口述 #1 用)、專案一 trade-off/演進段口述化、專案三問題/設計補口述句;唸稿鏡像頁同步重發。
**7/31 深夜四輪(討論後定稿)**:★1 金句後補 p99 句("'predictable' includes fast — tail latency is part of the contract",效能訊號不動金句節奏);★2 換「晶片定天花板、軟體定貼多近」句(產業通則取代對 Etched 內部現實的斷言)+ 織入 from-scratch/performance-critical(嗜好變正職句);★3 證據段換 **Moderation demo 衝刺**(原「面試準備」例退役:自我指涉+非工作產出)——⚠ 三細節待補真:時長、結局一句、是否點名 William。culture fit 剩餘討論(★6/★7 等)下次繼續。

閥門(時間不夠的砍序):認題卡 →(8/9)重打 → k;**i、j、l、m、兩段 taper、早起梯度不砍**。模擬題超時 = 挖到洞,記洞不記違規。
