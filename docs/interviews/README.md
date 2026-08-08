# 面試紀錄(跨機器 memory)

這個資料夾是面試進度的**唯一真相來源**——Claude 的本機 memory 不跨機器,所有面試紀錄、回饋、下一階段計畫一律寫這裡,commit + push 後家機才看得到。

## 目前狀態(更新於 2026-08-06)

| 輪次 | 日期 | 結果 |
|---|---|---|
| R1 coding(DMA dispatcher) | 2026-07-28 | ✅ 過,feedback 正向 → [紀錄](2026-07-28-tps-round1-dma.md) |
| coding #2(Jason Catlin) | 2026-08-06 10:15 | ✅ 考完,dma_copy over segments,題偏簡單 → [紀錄+洞清單](2026-08-06-r2-coding-jason-dma-copy.md) |
| technical deep dive(Ulysses Kao) | **8/10(一)10:00–10:45**(meet kbq-myia-tvk) | — |
| coding #3(Jan Lagarden,自 8/6 改期) | **8/11(二)09:15–10:00**(CoderPad 9FF772PP + meet off-xvwe-qek) | — |
| recruiter debrief(Molly Huang,隨 Jan 場移) | **8/11(二)10:00–10:15** | — |

**8/6 異動**:Jan 場臨時改期(面試官有事),官方信 2026-08-06 確認新行程如上;**待辦:Reply All 確認 + NDA 電子簽(另一封信,面試前簽)**。8/6 當天實際只考 Jason 一場。

Onsite 結構(2026-08-01 官方行程信確認;**⚠ 2026-08-06 再改:Jan 場沒在 8/6 跑成,移 8/11 09:15,debrief 隨移 8/11 10:00——以上方狀態表為準,本段保留歷史**;修正 7/31 記載的「deep dive 8/11」):兩場 coding 併到 **8/6(四)同天連跑**(09:15–10:00 Jan Lagarden、10:15–11:00 Jason Catlin,中間只休 15m——連打的體力與切換是新變數,8/4 背靠背模擬對此);**deep dive 在 8/10(一)10:00–10:45(Ulysses Kao),緊接 recruiter debrief 10:45–11:00(Molly Huang)**;**culture fit 沒有獨立場**——散在各場 behavioral 提問 + debrief,culture fit 稿的用途 = 每場開頭自介 + debrief。**兩個結論:六題攝取全部壓進 8/5 前;deep dive / culture fit 出聲練習壓進 8/7–8/9 三天(8/10 上午即上場)。**

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
| n | priority job scheduler + dependency DAG | **✅ lite 8/3 晚**(改制:harness 英文 spec+clarify 版;帳:cards 8/3 §九) | BinaryHeap+seq 破平手、indegree 入場閘、priority inversion |
| o | boot-order planner(**algo 系**,8/2 深夜新增) | lite 8/3 開機槽(取代 graph 一題) | Kahn 波次、**DAG 最長路徑=P 因無環**、環回報 pred-walk、blast radius;PDF「演算法穿硬體皮」對策首發 |

**廣度用認題卡補(15–20m/張,不寫完整 code)**:[spec-cards.md](spec-cards.md) 七張,每張=讀埋洞英文 spec → 寫 ≥3 clarify 問題 → 30 秒英文定界出聲 → state 表 → 開[答案鍵](spec-cards-answers.md)(含英文稿)。SP/FP/FR/TA/TQ 五張是複習卡(衝刺期已有 a–h 肌肉:signal_pipeline/c/e2/f/h);HW-L/HW-M 兩張當 l(lite)/m(全場)的 10 分鐘前導。

## 逐日計畫(7/30 → 8/10;練習時間比衝刺期少,量已壓)

原則:白天打字場 = 模擬題(clarify 打字來回不用出聲);**8/6 前晚上只留 coding 支援輕出聲(clarify 句庫/四形狀對白/30 秒定界),deep dive 口述 + culture fit 唸稿整段後移 8/7–8/9(8/10 10:00 上場)**,每晚至多一件出聲事,標「(選)」的累了就砍。signal pipeline 由 j 題 + 複讀覆蓋,不另開項目。時間資源(7/31 口報):週日 8/2 大塊,8/3–8/5 上班日也比平常多。

| 日期 | 白天(公司,~90m 上限) | 晚上(出聲) |
|---|---|---|
| 7/29 三 | **休息日(輕活)**:讀 `sol_sim_i_dma.rs`(三張表 + Dry-Run 紙上走) | deep dive / culture fit 英文稿起草(打字,輕;材料先貼給 Claude) |
| 7/30 四 | ✅ 實績:i-lite 40m(**drills 填空版** 6/6 綠——R1 兩洞在 drill 層關掉;空白重寫層的驗證留給 sim m(7/31 改制後前移 8/2),它是 R1 的直系延伸)+ 卡 SP(超時但值;批改入 spec-cards.md;兩課:freshness vs completeness 是 clarify 問題、API 層/primitive 層分層)+ 量級 reps v3 批改入檔(R1 掉零重寫/R2 補一行/R3 忘 timeout;荒謬檢查 0/3)+ 除膜手冊上線 `html_p/capacity-four-shapes.html`(四形狀+單位攜帶+名詞卡+錨點庫;7/30 晚應用戶要求補「四形狀英文對白」節——每形狀配面試官認題句+敘算句+數字唸法卡,晚上出聲場每晚挑一形狀唸 3 遍) | 唸稿:自介 60–90s + why-us + 補收 reps 殘帳(R1 數字鏈/R3 重算,5m)。**live spec-heavy 移 7/31 白天**(7/30 晚體力不足當晚拍板;它本來就是加開項,砍序第一位) |
| 7/31 五 | ✅ 實績:**reps 三題全結案**(R1/R2 v5 收、R3 v6 收;**五行頭結帳骨架定案** Given/Chain/Cross/Sanity/Verdict,入手冊頁分頁②+鏡像重發)+ **卡 SP 重做 19m 錶內**(五驗收 ①✓②✓④✓⑤⚠;③scope creep 換皮復發=無界 VecDeque;賽後補課:ISR=劫持核心的執行模型)+ 🔴 **sim j 全場 16:39–17:24 錶內完成 Phase 1+2**——wake 語意主動問到(卡 SP 漏的當天收斂)、自寫邊界測試抓到 shutdown 漏 drain 與測試汙染兩個真 bug 並自修、oracle 參考測試 2/2 綠;三課:shutdown=先 flag 後 wake、state 管道不混搭、黑箱測試配方要自發。**節奏洞第一驗證點:過**。live spec-heavy 砍→滑 8/1(晨間拍板)。心得 12 卡:`scratch/cards_2026-07-31.md` | 晚實績(收工 8/1 ~02:00):✅ **spin_lock drill 首寫填綠 3/3**(四洞+紙上五問;review 兩洞當場修——try_lock weak→strong〔單發假失敗=作偽證,None≠被佔用〕、AcqRel→Acquire〔上鎖時無物可發佈〕;commit bf98da9)+ 對話沉澱**兩教材頁已發**(mesi-rmw-atomics 🚌:MESI 四態/RMW 家族/TAS vs TTAS 帳單/釋放瞬間驚群;guard-design 🛡️:lifetime 綁定/Send 裁決/poisoning——index 深讀區 +2 卡)。(選)心得卡輕出聲**滑**,英文鐵律 7/31 記 ✗。同晚入庫 **conflation_slot 三層**(drill 排加時槽 8/2 尾或 8/3——**兩天未吃,8/4 凌晨裁決:drill 移 8/6 後與 challenge 白紙版同批固化**,概念層已深學+介面小=風險最低,8/5 定界輪帶一句認題;8/4 傍晚真空檔=第一順位選配)。deep dive 口述隨稿件線後移 8/7 起 |
| 8/1 六 | ✅ 實績(歌唱課+晚間舞台劇的夾縫日,量照 lite 排):**k 前導卡**(題幹當卡,10m 錶:clarify 六問命中五類——含 sim j 才學的 wake 語意;漏 shutdown 與公平性兩類 → **「lifecycle 三問」入固定 clarify 清單**)+ 追問課四發(conflation 換插槽不重開機/replace vs fold 判準/bitmask 回本線=N≫活躍數/aggregator=慢的集中化+丟摺批三路)→ 下午 **欠帳三題**:swap(0) ✓、清 bit 順序 ✓、**sizing 五行頭首答 ✗**(兩數字算對沒比較=「數字一出就收筆」原型;Cross 算成頻寬=假 Cross;重考走完批天花板 200k<320k → 摺+1% 抽樣,**三選一裁決權在門檻問題:sink 成本結構/資料語意/SLO**)→ **percpu_fanin drill 16m 填綠 4/4** + review 抓真洞:**先睡才查 stop**(sticky flag 合併只夠醒一次;mock 1s 保險絲把吊死降級成卡秒)→ 紅測先行 `shutdown_exits_promptly_without_pending_wake` 修至 5/5(03d0284)。**live spec-heavy 正式砍死**(8/4 背靠背全新題 ×2 = 替代品)。滑 8/2:sol/圖解對照 15m + 卡 FP。帳細節:`scratch/cards_2026-08-01.md` | 鐵律 code ✓ 英文 ✓(睡前 sim k 定界 30 秒出聲);對白一景砍(不欠) |
| 8/2 日(大塊) | ✅ 實績:sol_sim_k 對照 15m 暖身 → HW-M 前導卡(LL/SC 岔路課順收:weak CAS=單發 LL/SC 映射、假失敗是架構本性)→ 🔴 **sim m 全場 14:50–15:55(65m,+20;規:記洞不記違規)**——clarify ~22m 命中 hang 語意/zombie/3-strikes/「slow≠dead 無 fence 只能 bound risk」senior 句,但 **idempotency 靠釣兩竿**、timeout 走四步才落 3×;code 40m 搭出三段式骨架,**自寫測試 0**(sim j 第三課復發)→ review 八洞(三跑不起來級;最重:clarify 半場 non-idempotent、實作卻重派已完成塊=**討論→落地斷線**)→ **場後不開錶自主修帳全關(+修出又收掉 2 洞),參考測試 3/3 綠**(redispatch/retry_budget/zombie)。心理帳:「寫不完→是不是面不過」→ 數據校準:sol 191 行=六題之最(j=105 錶內收的那題),45m 框架隱藏前提=R1 底盤肌肉重打,**首打超時=預期結果**。晚:Alarm 具名 struct 重構(tuple>3 欄鐵律;Ord 四件套課)+ sol 對照三課(**deadline 與 owner 同居=stale 不可表示/先問 N 再選形狀/殭屍=生存證明即復活**)+ 挑出 sol 刺(tries 只進不出)。**卡 FP 砍**(閥門序);**l lite 深夜補打 ✅**(00:50 卡+clarify→drills 填空,中途開錯檔進 rehearsals 空白版造成 20 分鐘霧、SubmitHead/comp_head 混 ring 一洞自修 → **4/4 綠**含 backpressure/亂序/wrap;賽後 Q&A:barrier=Release/Acquire 的硬體版、SeqCst 不需要)。帳:`scratch/cards_2026-08-02.md` | 英文漏接三筆實錄→對策定型:**裁決抄紙、規則 read back、多問編號逐答**(入 8/4 評分表);(選)出聲隨緣不欠 |
| 8/3 一(大塊) | ✅ 實績:**sol_sim_m 刺結案**(8/2 挑出的 tries 只進不出——reference/sol 補「完成/放棄」兩刪除點+死單不進表,紅測先行 6/6 綠;設計課:計數搬進 ReqState 同居=洞不可表示;7bce2f0)→ **l 複讀 ✅ 升級深問課**(自推「MMIO 佇列=SPSC、消費者是矽」=sol 檔頭第一句;多 submitter → &mut 簽名編碼單寫者+NVMe per-core queue pair=sim k 形狀+「先問能不能不要 MPSC」;barrier 本體=排空 store buffer、≠刷 cache;lifo=把「順序非 contract」變可撥變因;watchdog「6 台不用 heap」自推=sol 同款,「先問 N 再選形狀」癒合)→ **午後認知飽和事故+數據校準**(詳 cards §三:LC 對映秒懂——sim o=207+1136+2050、extract_cycle=LC 142;三層診斷=檔頭倒著解碼/教錯對象/疲勞放大器;**制度修正:algo 首打槽 25m→45m 認錯、drill 檔讀法協定=測試→簽名→寫→檔頭最後**)→ 🔴 **sim o 填空 ✅ 8/8 綠**(不計時三輪拉鋸:v1 Dijkstra OR 閘=吊死/假環+filter().enumerate() 源頭認錯;v2 鬆弛關進閘=同圖換序 26→25 repro 實錘;v3 **紅測先行首次自主走完**,自我診斷「push 條件=Kahn 非 Dijkstra」收尾;賽後追問:parent 不能餵 extract_cycle=「找環用邊的存在性,不能用執行痕跡」)。✅ 晚場實績:**n lite 全鏈收**(改制:前導卡→harness 英文 spec+打字 clarify 三中三,deps 過去式=Phase 2 主雷自己問到;drill 5/5 綠;真洞「銷帳早於放行」+review 兩洞 dispatch 單發/dependents 只進不出=sim m 刺回鍋)+ **async 兩皮骨架首默 7 錯→兩輪修 0**(park/unpark 詞組、future=狀態機 drop 洞;`scratch/{tcp_server,executor}.rs`)+ 口說 30 秒英文 ✓;場前加映 AG-T 題意課+critical path 用途課(8/4 卡 AG-T 降半複驗)。滑帳:sim o stepper 複讀/i–k 複掃/卡 FP/TQ→8/4。鐵律 ✓✓✓;詳帳 cards §九 | 心理課入卡:躁=被動解碼無回饋、動手=有進度條;**8/6 逃生梯=定界句就是 LC 翻譯** |
| 8/4 二(大塊) | 開機:~~n lite 補打~~(**8/3 晚已收,槽釋放**)→ **骨架默寫抽查 15m 提前進開機槽,輪替制:async 兩皮第一位**(8/3 晚首默 7 錯,隔夜冷重默=驗收)**+ length-prefix 3 行(7/27 唯一 ✗)+ spsc use 塊(tokio use 塊全忘=衰退訊號);pool 兩條件/TCP std 六行移 8/5**→ **背靠背模擬**:Claude 出全新題 ×2(禁區同 live sim 規:不碰 j–n hidden spec 域;**題池納入 async 象限——tokio 或純 std 皮,盲測**),各 25–30m 只跑到 state 表+骨架,**中間休 15 分鐘照 8/6 真實節奏**——兩場連打的體力與切換先試一次;**每題埋一個 sizing 小題,用量級五行頭(Given/Chain/Cross/Sanity/Verdict)作答**(7/31 reps v5 定案的結帳骨架;評分表含 state 表開場 3 分鐘+英文抄紙/read back/編號逐答);下午空檔認題卡場:**FR → AG-R → TA → AG-T**(砍序照這個順序反過來砍;FR=e2 fd_registry=JD sleeper、AG-R=widest path 認題=algo 系(8/3 已提前吃掉骨,變複驗)、TA=f aggregator、AG-T=聚合樹修復;**卡 FP、TQ 自 8/3 滑入排最尾,砍序最先**;收完卡全結,8/5 翻閱只剩複讀)+ 擠空檔:i–k 洞複掃 + sim o stepper 複讀(8/3 晚滑入;骨架抽查已移開機槽) | **08:30 起(梯度開始;9:15 場只需 07:45 起,比 R1 的 8:45 場輕)** |
| 8/5 三 | taper:不碰新題、檢查表 + 時間預算 + **六題 30 秒英文定界唸一輪**(sol 檔頭 + spec-cards-answers)+ **async 兩皮定界**(d 題 tokio 三句 + 純 std block_on/poll 合約兩句)+ **tokio/Waker 骨架重默 5m**(骨架默寫=taper 豁免項;**加抽查殘項:pool 兩條件+TCP std 六行**,自 8/4 輪替滑入)+ ~~量級五行頭默寫 30 秒~~(**8/5 配比終裁砍**:sizing=配菜,被問才答「數字+比較+行動」一句;五行頭表格=Claude 直算/引導用,不叫用戶默寫手算)+ **a–n 認題翻閱 30m**(8/2 增:把六題定界輪往 a 端延伸成全題庫過一輪;材料限 spec-cards 複習卡 + sol 檔頭 + 心得卡,**不開實作碼**——一開函式本體就不是 30 分鐘的事;**記洞不修洞**:翻到不穩寫進檢查表帶進場,面試時多問一句 clarify 就能繞,真要補的提前到 8/3–8/4 處理) | 08:00 起、00:30 熄燈 |
| **8/6 四** | ✅ 實績:**只考 Jason 一場(10:15)**——dma_copy over segments,題偏簡單,follow-up(硬體壞掉怎麼辦)以 retry+exponential backoff+fail 上限+升級通知作答;洞清單三條(idempotency 沒點名/升級詞彙可更貼硬體語境/Jan 場不因今天簡單而降備)→ [紀錄](2026-08-06-r2-coding-jason-dma-copy.md)。**Jan 場改期 8/11 09:15**,debrief 隨移 8/11 10:00 | 收帳入庫 ✅;**Reply All 確認 ✅ + NDA 簽 ✅(當天辦完)**;晚上休整(遊戲夜,計畫內零產出)——稿件三洞(★3 Moderation demo 三細節/conflict 人物場景/專案三最難 bug+如果重來)全數照原排 8/7 起頭,素材倒給 Claude 收成口述稿 |
| 8/7 五 – 8/9 日 | coding 歸零,全轉 deep dive / culture fit(只有三天,8/10 上午即上場)。**✅ 8/7 白天實績**:三個稿件洞全清(★3 Moderation 三細節補真+訊息改寫〔通宵情節與原稿「睡眠紅線」對撞已解〕、★6 conflict 人物場景補真〔tech lead 提 mail flag+dsync 捷徑〕、專案三第 6/7 題以 repo 實證落地〔最難的 bug=`dbg!`→451 tempfail;如果重來=顯式入場控制〕)+ **砍掉一句 7/29 Claude 編的關鍵句**+ 吞吐數字出處結案(load test 已入 MR !65 可引用;3,500 定性為外推,照履歷講但不主動報精確數字)+ **四檔重構**(練習本只留問題/英文/中文,參考資料另立 notes,新增 practice-method)。**逐晚菜單改以 [practice-method.md](practice-method.md) 為準(8/7 晚拍板改制:8/7 整晚休,只留睡前 15m 專案二七題照唸一遍當保險;原 8/7 菜單整包併入 8/8)**:8/8 白天開場合併場 ~2–2.5h(專案二 Q1/Q6/Q7 三遍法+修復包+數字表 → 專案一+三 → culture fit ★1★2★3 → 打斷練習三專案隨機抽、專案二必抽 → 三十秒版;**會挖洞的活動〔第一次照唸、打斷練習、改稿〕8/8 當天結束——8/9 沒有備援日**);8/9 改制(8/9 拍板,稿件降級為單字句庫+紅線清單):taper=**live Q&A**(Claude 持履歷英文提問、先出聲再打字作答,取代照稿串講;紅線照舊、挖洞只記一句修法不重寫稿)+ 只唸首尾句 + **重讀 Gjengset 40-hours 原文**(culture fit 唯一殘項),早睡不動 | 整段維持 ≤08:30 起 |
| **8/10 一** | **deep dive 10:00–10:45(Ulysses Kao,meet kbq-myia-tvk)**;08:30 起即可;**手機上場包**(8/9 凌晨製,自介/首尾句/Q6 三故事/AA 四拍/修復包/數字唸法):[claude.ai artifact](https://claude.ai/code/artifact/86533cd0-759e-429d-81e1-43fe336b50a6) + Heptabase G&L 白板「【上場包】8/10 deep dive 速讀」 | 收帳:全程紀錄入庫。**20:00 mock with William**(他也面 Etched Supercomputing SWE;Withers 當面試官,考 thread pool + submit with JobHandle——當考官=主動回憶,順練 8/11;考官包:[mock-0810-william-threadpool.md](mock-0810-william-threadpool.md))。**mock 前後仍跑 Jan 場輕 taper**(量 ≤1h:`scratch/taper_0805.md` §E 掃讀 + §B 殘項默寫 + 兩皮串味複掃 + 8/6 洞清單三條;不碰新題不開 oracle),00:00 前熄燈(9:15 場 07:45 起) |
| **8/11 二** | **coding 09:15–10:00(Jan Lagarden,CoderPad 9FF772PP)→ recruiter debrief 10:00–10:15(Molly Huang,meet off-xvwe-qek)**;07:45 起,晨間動線照 `scratch/recall_checklist.md` §0、材料 taper §E(8/6 那套原封重跑);debrief 備妥想問的 3 個問題 + 結果時程問法 | 收帳:全程紀錄入庫 + onsite 全輪結案 |

culture fit 英文稿範圍:自我介紹 60–90s、why this company、conflict、failure、proudest project、想問他們的 3 個問題;底稿 = 7/26 的三條經驗故事。

**稿件四檔結構(2026-08-07 重構——原本練習材料與參考資料混在一起,唸稿時要跳過大量註解)**:

| 檔案 | 放什麼 | 什麼時候開 |
|---|---|---|
| [deep-dive-projects.md](deep-dive-projects.md) | **練習本**:三專案 × 七題,每題只有「面試官的問題 + 要唸的英文 + 那段英文的中文意思」 | 出聲練習時 |
| [culture-fit-script.md](culture-fit-script.md) | **練習本**:11 題同上格式 | 出聲練習時 |
| [deep-dive-notes.md](deep-dive-notes.md) | 參考資料:數字出處與能不能講、file:line 證據、三層成本模型、追問彈藥、備用 bug 故事、**保密紅線** | 被深追想加深度時、或想確認某個數字能不能講時 |
| [culture-fit-notes.md](culture-fit-notes.md) | 參考資料:各題沿革、為什麼這樣寫、追問防禦、Gjengset 引用守則 | 同上 |
| [practice-method.md](practice-method.md) | **怎麼練**:不背整段(只背首尾句)、三遍法、打斷練習、修復包七句、數字唸法表、三十秒版、8/7–8/9 逐晚菜單、找 Claude 練的四種指令 | **練習前先讀這個** |

**稿件狀態(7/29 晚,二輪定稿)**:deep dive 三專案 + culture fit 五題 ★ 已起草並按 Withers 回填的五個確認項修正(PG 14/收信不能停/內建不一致偵測計數 0/CPU-spin=前人 bug 只當 debugging 故事/logging daemon 誠實架構 = syslog-ng 尾端寫 DB);個人 failure 改用 **authz/authn 缺失 → CVSS 10.0** 故事。素材 = 履歷 + Holdwin 簡報轉軸,交易對照句已拿掉;唸稿鏡像頁 `html_p/r2-interview-scripts.html`(claude.ai index R2 備戰區有卡)。**仍待補**:conflict 故事的實際人物場景、專案三最難的 bug。
**7/31 三輪(逐字化)**:culture fit ★1/★2/★3/★6/★7 全數升級為**全英文逐字稿**(骨架版實測唸不出來——句間中文接縫卡嘴;中文降級為「唸法註」不出聲);★3 織入 Jon Gjengset 40-hours 論點(含使用守則:上場前重讀原文、被追問講論點不掰細節、拼命文化訊號強就把人名句縮成 "I optimize for sustained throughput, not heroics")。deep dive 專案二全段口述化(今晚口述 #1 用)、專案一 trade-off/演進段口述化、專案三問題/設計補口述句;唸稿鏡像頁同步重發。
**7/31 深夜四輪(討論後定稿)**:★1 金句後補 p99 句("'predictable' includes fast — tail latency is part of the contract",效能訊號不動金句節奏);★2 換「晶片定天花板、軟體定貼多近」句(產業通則取代對 Etched 內部現實的斷言)+ 織入 from-scratch/performance-critical(嗜好變正職句);★3 證據段換 **Moderation demo 衝刺**(原「面試準備」例退役:自我指涉+非工作產出)——⚠ 三細節待補真:時長、結局一句、是否點名 William。culture fit 剩餘討論(★6/★7 等)下次繼續。

閥門(時間不夠的砍序):live spec-heavy(7/31)→ 8/4 背靠背改單場 → 卡 FP → k/n lite 縮成認題(冷讀+clarify+state 表,不填空);**j 全場、m 全場、l lite、taper、早起梯度不砍**。模擬題超時 = 挖到洞,記洞不記違規。
