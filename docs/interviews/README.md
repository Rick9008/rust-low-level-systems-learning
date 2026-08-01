# 面試紀錄(跨機器 memory)

這個資料夾是面試進度的**唯一真相來源**——Claude 的本機 memory 不跨機器,所有面試紀錄、回饋、下一階段計畫一律寫這裡,commit + push 後家機才看得到。

## 目前狀態(更新於 2026-07-31)

| 輪次 | 日期 | 結果 |
|---|---|---|
| R1 coding(DMA dispatcher) | 2026-07-28 | ✅ 過,feedback 正向 → [紀錄](2026-07-28-tps-round1-dma.md) |
| coding #2 + #3 | **8/6(四)09:15–10:00、10:15–11:00**(中間休 15 分鐘) | — |
| technical deep dive + culture fit 場 | **8/11(二)** | — |

Onsite 結構(2026-08-01 官方行程信確認;**修正 7/31 記載的「deep dive 8/11」**):兩場 coding 併到 **8/6(四)同天連跑**(09:15–10:00 Jan Lagarden、10:15–11:00 Jason Catlin,中間只休 15m——連打的體力與切換是新變數,8/4 背靠背模擬對此);**deep dive 在 8/10(一)10:00–10:45(Ulysses Kao),緊接 recruiter debrief 10:45–11:00(Molly Huang)**;**culture fit 沒有獨立場**——散在各場 behavioral 提問 + debrief,culture fit 稿的用途 = 每場開頭自介 + debrief。**兩個結論:六題攝取全部壓進 8/5 前;deep dive / culture fit 出聲練習壓進 8/7–8/9 三天(8/10 上午即上場)。**

R1 前的衝刺計畫在 `../../SCHEDULE.md`(7/16→7/28,已結案)。

## 下一階段練習方向:spec-heavy 新題型

R1 實測的題型:**長英文 spec(有洞)+ 一堆 provided API + 實作一個 fn,面試官是唯一 oracle**。
與 a–h 彩排的差別:重心從「手搓結構」移到「clarify 出隱藏 spec」+「多重 state 的 event loop 骨架」。R1 證明 clarify 已過線(面試官點名稱讚);真正的洞是**時間不夠 → code 有漏洞**,要練的是「40 分鐘內把 spec 轉成 state 表 + 骨架」的節奏。

練法:題本 [sim-problems.md](sim-problems.md)——每題 **Phase 1 開場給、Phase 2 面試官驗收後才放**(照 R1 的漸進節奏);面試官手冊 `sim-interviewer-guide.md` **跑題中不准開**。clarify 用英文打字來回(句庫:[clarify-phrasebook-en.md](clarify-phrasebook-en.md),含聽力修復句;每晚出聲場挑一類唸 3 遍)→ 計時 45m → review 20m。

**Harness 已全部入庫(跨機器直接執行)**:`rehearsals/src/sim_{i,j,k,l,m,n}_*.rs`——**介面與 requirement 一律英文**(面試全英文,讀英文就是練習;中文對照:[sim-problems-zh.md](sim-problems-zh.md))。上半「題目給的介面」可讀,**mock/SimBus 實作區與 `rehearsals/tests/sim_*_test.rs` 跑題前不准讀**(oracle 會在你違反協定時當場 panic 開燈)。執行:作答區填實作+檔尾寫自己的測試 → `cargo test -p rehearsals sim_<x>` → 跑完拔參考測試的 `#[ignore]` 對邊界 → 開 sol 對照。**六份 sol 全備**(`rehearsals/examples/sol_sim_*.rs`,寫完才開;19/19 參考測試已用解答驗過):i 的兼 7/29 閱讀材料(`cargo run -p rehearsals --example sol_sim_i_dma`);m/n 檔頭預讀取消(7/31 改制:兩題場次前移 8/2、8/3,場後直接全開)。
**轉製層已入庫(2026-07-30)**:六題各有 reference 教學版(5-pillar 檔頭 + mock + 手 trace 測試)
+ drills 填空版(挖核心轉移函式),對照表與複打流程在 `rehearsals/README.md` 進度狀態表下方;
各檔頂防雷 banner 標開放日期——**該題排定場次(全場或 lite)跑完才開**;lite 題(k/l/n)的 drills 填空版本身就是 lite 場材料,開跑即用,reference 答案本一律場後;全場題跑完再轉 drills 複打當固化練習。
**圖解鏡像(2026-07-31)**:`html_p/r2-sim-walkthroughs.html`(claude.ai 鏡像在 index 頂部 R2 備戰區,🔬)
——六題各一分頁:provided API 逐函式解剖表、全景示意圖(資料流 + 區域著色 + 怎麼讀)、互動 stepper、
code 走讀、陷阱清單;**每分頁掛同款防雷門**(i 已開,j–n 各自場次跑完才開;7/31 改制後 m/n 前移 8/2、8/3),賽前仍讀作戰本前導卡。

**補充教材(7/31 收錄,使用者自製)**:`html_p/conflation-slot-stepper.html`——per-key conflation slot 圖解 + 三 stepper(值層/通知層分離、lost update 確定性重現、無鎖通知順序「通知可多不可少」)。卡 SP「freshness vs completeness」課的下半場;wake 合併/pending signal 是它的同型結構。index R2 區有卡,claude.ai 鏡像已發。

**圖解作戰本**:`html_p/r2-onsite-visual-guide.html`(claude.ai 鏡像在 artifacts gallery,index 已收卡)——題型解剖(API 三堆分類法 + reactor 五步骨架)、sim i 三張表互動 stepper(pipeline 分水嶺劇本)、sim j–n 概念前導與 clarify 種子(刻意不含解法,跑題前讀是安全的)、英文對白道場五場景(雙語,R1 洞①的處方)。用法:各 sim 開跑前 10 分鐘讀對應前導卡;對白每晚出聲場挑一景唸 3 遍。

候選模擬題(JD 軸:telemetry / 硬體訊號 / event loop / lockless):

**7/31 改制(coding #2/#3 併到 8/6 同天)**:m、n 原排 8/8、8/9,會落在面試之後——全部攝取前移 8/5 前。全場只留 j(7/31,賽前唯一整場節奏校準)與 m(8/2,R1 直系延伸、命中率最高,「空白重寫層的驗證」由它扛);k、l、n 降 **lite 格式**(~55m/題):作戰本前導卡 10m(含手寫 state 表草稿)→ drills 填空 30m(照 i-lite,核心轉移函式親手寫)→ sol + 圖解分頁對照 15m。lite 少掉的是搭骨架的重複時間成本,親手寫核心邏輯與知識對照都保留。8/3 認題輪取消(被 lite 覆蓋)。

| # | 題 | 場次(7/31 改制) | 練什麼 |
|---|---|---|---|
| i | DMA dispatcher v2(R1 修洞重做) | ✅ 7/30 i-lite 完成 | per-request state、done 路由、cancel |
| j | ISR → bottom-half pipeline | **全場 7/31** | ISR 限制(不能 alloc/block)、SPSC 交棒、overflow policy |
| k | 多核 ISR / per-CPU queue fan-in | lite 8/1 | 多核多緒 race 避免、MPSC、聚合 |
| l | MMIO command queue(doorbell + completion ring) | lite 8/2 | descriptor→barrier→doorbell 鐵律、head/tail 對硬體、亂序 completion tag |
| m | engine watchdog / timeout(R1 直系延伸) | **全場 8/2**(自 8/8 前移) | event loop 的第三種 state:時間;idempotency 決定敢不敢 retry |
| n | priority job scheduler + dependency DAG | lite 8/3(自 8/9 前移;#2/#3 同天,原「8/6 後換皮」窗口不存在,直接吃) | BinaryHeap+seq 破平手、indegree 入場閘、priority inversion |

**廣度用認題卡補(15–20m/張,不寫完整 code)**:[spec-cards.md](spec-cards.md) 七張,每張=讀埋洞英文 spec → 寫 ≥3 clarify 問題 → 30 秒英文定界出聲 → state 表 → 開[答案鍵](spec-cards-answers.md)(含英文稿)。SP/FP/FR/TA/TQ 五張是複習卡(衝刺期已有 a–h 肌肉:signal_pipeline/c/e2/f/h);HW-L/HW-M 兩張當 l(lite)/m(全場)的 10 分鐘前導。

## 逐日計畫(7/30 → 8/11;練習時間比衝刺期少,量已壓)

原則:白天打字場 = 模擬題(clarify 打字來回不用出聲);**8/6 前晚上只留 coding 支援輕出聲(clarify 句庫/四形狀對白/30 秒定界),deep dive 口述 + culture fit 唸稿整段後移 8/7–8/9(8/10 10:00 上場)**,每晚至多一件出聲事,標「(選)」的累了就砍。signal pipeline 由 j 題 + 複讀覆蓋,不另開項目。時間資源(7/31 口報):週日 8/2 大塊,8/3–8/5 上班日也比平常多。

| 日期 | 白天(公司,~90m 上限) | 晚上(出聲) |
|---|---|---|
| 7/29 三 | **休息日(輕活)**:讀 `sol_sim_i_dma.rs`(三張表 + Dry-Run 紙上走) | deep dive / culture fit 英文稿起草(打字,輕;材料先貼給 Claude) |
| 7/30 四 | ✅ 實績:i-lite 40m(**drills 填空版** 6/6 綠——R1 兩洞在 drill 層關掉;空白重寫層的驗證留給 sim m(7/31 改制後前移 8/2),它是 R1 的直系延伸)+ 卡 SP(超時但值;批改入 spec-cards.md;兩課:freshness vs completeness 是 clarify 問題、API 層/primitive 層分層)+ 量級 reps v3 批改入檔(R1 掉零重寫/R2 補一行/R3 忘 timeout;荒謬檢查 0/3)+ 除膜手冊上線 `html_p/capacity-four-shapes.html`(四形狀+單位攜帶+名詞卡+錨點庫;7/30 晚應用戶要求補「四形狀英文對白」節——每形狀配面試官認題句+敘算句+數字唸法卡,晚上出聲場每晚挑一形狀唸 3 遍) | 唸稿:自介 60–90s + why-us + 補收 reps 殘帳(R1 數字鏈/R3 重算,5m)。**live spec-heavy 移 7/31 白天**(7/30 晚體力不足當晚拍板;它本來就是加開項,砍序第一位) |
| 7/31 五 | ✅ 實績:**reps 三題全結案**(R1/R2 v5 收、R3 v6 收;**五行頭結帳骨架定案** Given/Chain/Cross/Sanity/Verdict,入手冊頁分頁②+鏡像重發)+ **卡 SP 重做 19m 錶內**(五驗收 ①✓②✓④✓⑤⚠;③scope creep 換皮復發=無界 VecDeque;賽後補課:ISR=劫持核心的執行模型)+ 🔴 **sim j 全場 16:39–17:24 錶內完成 Phase 1+2**——wake 語意主動問到(卡 SP 漏的當天收斂)、自寫邊界測試抓到 shutdown 漏 drain 與測試汙染兩個真 bug 並自修、oracle 參考測試 2/2 綠;三課:shutdown=先 flag 後 wake、state 管道不混搭、黑箱測試配方要自發。**節奏洞第一驗證點:過**。live spec-heavy 砍→滑 8/1(晨間拍板)。心得 12 卡:`scratch/cards_2026-07-31.md` | 晚實績(收工 8/1 ~02:00):✅ **spin_lock drill 首寫填綠 3/3**(四洞+紙上五問;review 兩洞當場修——try_lock weak→strong〔單發假失敗=作偽證,None≠被佔用〕、AcqRel→Acquire〔上鎖時無物可發佈〕;commit bf98da9)+ 對話沉澱**兩教材頁已發**(mesi-rmw-atomics 🚌:MESI 四態/RMW 家族/TAS vs TTAS 帳單/釋放瞬間驚群;guard-design 🛡️:lifetime 綁定/Send 裁決/poisoning——index 深讀區 +2 卡)。(選)心得卡輕出聲**滑**,英文鐵律 7/31 記 ✗。同晚入庫 **conflation_slot 三層**(drill 排加時槽 8/2 尾或 8/3;spin_lock/conflation 兩 challenge 白紙版 8/6 後固化)。deep dive 口述隨稿件線後移 8/7 起 |
| 8/1 六 | ✅ 實績(歌唱課+晚間舞台劇的夾縫日,量照 lite 排):**k 前導卡**(題幹當卡,10m 錶:clarify 六問命中五類——含 sim j 才學的 wake 語意;漏 shutdown 與公平性兩類 → **「lifecycle 三問」入固定 clarify 清單**)+ 追問課四發(conflation 換插槽不重開機/replace vs fold 判準/bitmask 回本線=N≫活躍數/aggregator=慢的集中化+丟摺批三路)→ 下午 **欠帳三題**:swap(0) ✓、清 bit 順序 ✓、**sizing 五行頭首答 ✗**(兩數字算對沒比較=「數字一出就收筆」原型;Cross 算成頻寬=假 Cross;重考走完批天花板 200k<320k → 摺+1% 抽樣,**三選一裁決權在門檻問題:sink 成本結構/資料語意/SLO**)→ **percpu_fanin drill 16m 填綠 4/4** + review 抓真洞:**先睡才查 stop**(sticky flag 合併只夠醒一次;mock 1s 保險絲把吊死降級成卡秒)→ 紅測先行 `shutdown_exits_promptly_without_pending_wake` 修至 5/5(03d0284)。**live spec-heavy 正式砍死**(8/4 背靠背全新題 ×2 = 替代品)。滑 8/2:sol/圖解對照 15m + 卡 FP。帳細節:`scratch/cards_2026-08-01.md` | 鐵律 code ✓ 英文 ✓(睡前 sim k 定界 30 秒出聲);對白一景砍(不欠) |
| 8/2 日(大塊) | 🔴 **m 全場**(HW-M 卡當 10m 前導;R1 直系延伸、#2/#3 命中率最高;「空白重寫層的驗證」在此,自 8/8 前移)+ l **lite**(HW-L 卡當前導)+ 自 8/1 滑入:**sol_sim_k 對照 15m**(m 全場前當暖身)+ **卡 FP** | (選)輕出聲:已跑題(i–l)30 秒英文定界各唸一遍 |
| 8/3 一(大塊) | 開機:**drills graph 一題**(拓撲/indegree,20m——圖論興趣槽兼 n 的暖身)→ n **lite**(自 8/9 前移;認題輪取消,被 lite 覆蓋)+ i–k 洞複掃 + 骨架默寫抽查 15m | (選)clarify 句庫一類唸 3 遍 |
| 8/4 二(大塊) | **背靠背模擬**:Claude 出全新題 ×2(禁區同 live sim 規:不碰 j–n hidden spec 域),各 25–30m 只跑到 state 表+骨架,**中間休 15 分鐘照 8/6 真實節奏**——兩場連打的體力與切換先試一次;**每題埋一個 sizing 小題,用量級五行頭(Given/Chain/Cross/Sanity/Verdict)作答**(7/31 reps v5 定案的結帳骨架) | **08:30 起(梯度開始;9:15 場只需 07:45 起,比 R1 的 8:45 場輕)** |
| 8/5 三 | taper:不碰新題、檢查表 + 時間預算 + **六題 30 秒英文定界唸一輪**(sol 檔頭 + spec-cards-answers)+ **量級五行頭默寫 30 秒**(Given/Chain/Cross/Sanity/Verdict) | 08:00 起、00:30 熄燈 |
| **8/6 四** | **coding ×2(09:15–10:00 Jan Lagarden、10:15–11:00 Jason Catlin)**,07:45 起 | 當天紀錄入庫 + 兩場洞清單;收帳時確認 8/7–8/9 細排 |
| 8/7 五 – 8/9 日 | coding 歸零,全轉 deep dive / culture fit(只有三天,8/10 上午即上場):8/7 稿件補洞(★3 Moderation demo 三細節/conflict 人物場景/專案三最難 bug+如果重來)+ 晚口述 #1(專案二)→ 8/8 口述 #2+#3 + culture fit 全套唸(★1–★7)+ 模擬追問 → 8/9 雙全串 + taper、早睡;細排可在 8/6 收帳時再調 | 整段維持 ≤08:30 起 |
| **8/10 一** | **deep dive 10:00–10:45(Ulysses Kao)+ recruiter debrief 10:45–11:00(Molly Huang)**;08:30 起即可(10:00 場比 09:15 輕) | 收帳:全程紀錄入庫;debrief 備妥想問的 3 個問題 + 結果時程問法 |

culture fit 英文稿範圍:自我介紹 60–90s、why this company、conflict、failure、proudest project、想問他們的 3 個問題;底稿 = 7/26 的三條經驗故事。稿子檔案:`culture-fit-script.md`(寫完放本資料夾)。

**稿件狀態(7/29 晚,二輪定稿)**:deep dive 三專案 + culture fit 五題 ★ 已起草並按 Withers 回填的五個確認項修正(PG 14/收信不能停/內建不一致偵測計數 0/CPU-spin=前人 bug 只當 debugging 故事/logging daemon 誠實架構 = syslog-ng 尾端寫 DB);個人 failure 改用 **authz/authn 缺失 → CVSS 10.0** 故事。素材 = 履歷 + Holdwin 簡報轉軸,交易對照句已拿掉;唸稿鏡像頁 `html_p/r2-interview-scripts.html`(claude.ai index R2 備戰區有卡)。**仍待補**:conflict 故事的實際人物場景、專案三最難的 bug。
**7/31 三輪(逐字化)**:culture fit ★1/★2/★3/★6/★7 全數升級為**全英文逐字稿**(骨架版實測唸不出來——句間中文接縫卡嘴;中文降級為「唸法註」不出聲);★3 織入 Jon Gjengset 40-hours 論點(含使用守則:上場前重讀原文、被追問講論點不掰細節、拼命文化訊號強就把人名句縮成 "I optimize for sustained throughput, not heroics")。deep dive 專案二全段口述化(今晚口述 #1 用)、專案一 trade-off/演進段口述化、專案三問題/設計補口述句;唸稿鏡像頁同步重發。
**7/31 深夜四輪(討論後定稿)**:★1 金句後補 p99 句("'predictable' includes fast — tail latency is part of the contract",效能訊號不動金句節奏);★2 換「晶片定天花板、軟體定貼多近」句(產業通則取代對 Etched 內部現實的斷言)+ 織入 from-scratch/performance-critical(嗜好變正職句);★3 證據段換 **Moderation demo 衝刺**(原「面試準備」例退役:自我指涉+非工作產出)——⚠ 三細節待補真:時長、結局一句、是否點名 William。culture fit 剩餘討論(★6/★7 等)下次繼續。

閥門(時間不夠的砍序):live spec-heavy(7/31)→ 8/4 背靠背改單場 → 卡 FP → k/n lite 縮成認題(冷讀+clarify+state 表,不填空);**j 全場、m 全場、l lite、taper、早起梯度不砍**。模擬題超時 = 挖到洞,記洞不記違規。
