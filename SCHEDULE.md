# SCHEDULE.md — Etched TPS 衝刺(7/16 → 7/28)

> **2026-07-29 結案**:R1(7/28 TPS coding)✅ 過,feedback 正向(clarify 被點名稱讚;洞 = 時間不夠 code 有漏洞)。
> **R2 onsite**:coding 8/6(四)+ 8/11(二)、deep dive + culture fit 8/12(三),皆 09:15 開場。
> 新階段的逐日計畫、題本(sim i–m)、認題卡、句庫、稿件全在 **[docs/interviews/README.md](docs/interviews/README.md)**——本檔保留衝刺歷史,逐日表不再更新。
> 7/29 = 休息日(輕活):讀 `rehearsals/examples/sol_sim_i_dma.rs` + deep dive / culture fit 英文稿起草。

容量(7/22 v9 修正):平日 = 白天公司「打字場」淨 4h+(實排 3h 留正職突發)+ 晚上在家「出聲場」23:30–02:00(實排 2h);週末在家 8h。總量與舊「平日 5h」近似,但**場地決定內容**:出聲類(彩排 narrate/口述/錄音/卡片口述版)只能晚上或週末;打字類(drill/challenge/空白/修洞/dry-run/日讀/卡片筆寫)白天優先。

**每日鐵律**:收工前兩問——今天有打 code 嗎?有張嘴講英文嗎?兩個 yes 才算數。
**每場彩排 review 的打分順序**:pillar 1(clarify)永遠第一個打——那是你最弱的,每場彩排都是它的練習。
**原則:有彩排題覆蓋的 module,彩排就是它的 challenge,不重做。**(ring→a、pool→b、framer→c)

---

## 時間模型 v9(7/22 定,取代 v8 的時段假設)

1. **白天(公司,到 19:30)= 主力工作塊**:淨 4h+,打字/閱讀自由、**出聲不行**——彩排(narrate 是硬動作)排不進白天。
2. **晚上真實可用 23:30–02:00**:出聲場,每晚**至多一場彩排**(45+30)+ 一段口述。**02:00 熄燈**——舊 23:00 規則自始不可行(23:30 才到家),作廢;先前「破戒 +2h」等帳按此重估,多數不是紀律失敗是規則虛構。調 8:45 面試時差(7/22 修正:本人常態 02:00–03:00 睡 / 09:00+ 起,一步跳 23:00 做不到,改 30 分/天梯度,**槓桿是起床+晨光,不是硬躺**):熄燈/起床 = 7/22 02:00/08:45 → 7/23 01:30/08:30 → 7/24 01:00/08:15 → 7/25 00:30/08:00 → 7/26 00:00/07:45 → 7/27 23:30/**07:15–07:30 上場日**。平日晚彩排壓 23:30–01:00 收工;週末彩排改白天。真正紅線 = **睡滿 ≥6.5h**(R1 死因是疲勞)。
3. **卡片估時一律 30–45m**(已做過的重打卡 15m)。超時主因是挖到不會的東西——那是卡片的**產出**,結帳記「挖到 N 個洞」,不記「超時違規」。5m/15m 幻想估值作廢(7/21 卡#4+#6 實績為證)。

## 逐日

| 日期 | 內容(依序做) | 時數 |
|---|---|---|
| **7/16 四** | 〔v8.1〕卡#1(15m)→ **手寫 wrap trace(拍照)** → **`iter_mutate` drill 7 洞(盡力,硬停損)**。ring_buffer drill ✅ 已完成;aggregator 延伸**移 7/24** | ~3.5h |
| **7/17 五** | 〔v8.1〕開機第一件事:**移 bounded_queue `#[ignore]` 跑全綠(5m)** → 卡#2 → `thread_pool` drill 4 洞含 JobHandle(90m)→ `spsc_ring` drill + 逐 op 英文講 Ordering 理由(75m)→ **unsafe impl 三段式對 spsc_ring 首唸**。日讀:**bounded_queue reference(先)**→ spsc artifact | ~3.5h |
| **7/18 六** | 〔v8.1〕~~上午開場:iter_mutate 殘洞 ≤40m~~(**7/17 已補完清空**;framer drill 不需讓位,7/20 那條保險解除)→ 卡#3 → ★`spsc` challenge 空白手搓 + diff + 跑 loom(90m)→ ★`executor` drill+challenge 含 park-token 口述 + Delay(120m)→ Q7 timer 接尾(20m)→ `hw_bridge` framer **drill**(45m;standalone challenge 砍掉,c 就是它的 challenge) | ~5h |
| **7/19 日** | 卡#4 → 🔴**a#1 ring_drop_oldest**(45m+review 30m,pillar1 先打分)→ 漏洞清單 → `fd_registry` artifact 讀 + drill 3 洞(90m,弱點提前)→ **spsc 空白 #1**(20m) | ~4.5h,晚上休 |
| **7/20 一** | 卡#5 → **修 a#1 的洞**(targeted,60–90m)→ 🔴**b#1 pool_graceful**(45+30m)。通勤:event_loop / mini_runtime 略讀(餵 executor×reactor 那句) | ~3.5h |
| **7/21 二** | **卡#4+卡#6 開場連打**(15+15;#4 自 7/20 滑入)→ 🔴**e2#1 fd_registry**(45+30m)→ **b#1 補課**(45–60m:紅測 ×3 + lost-wakeup dry-run + 回放 + 英文錄音 a#1+b#1 合場——code 凌晨已全綠,不用再修)→ **executor 讀 + challenge 空白手搓**(60–90m,7/19 裁決落位)+ Q7 timer 口述接尾(10m,qa_timer_queue 頁)。睡眠債下預設:executor challenge 第一個滑 7/22 晚尾、Q7 口述併 7/26 recognition | ~4h |
| **7/22 三** | **白天實績 ✅**:lru 兩洞修畢(紅測×2+單獨 unlink×2+mutation 複驗,a77aa44)|qa_lockfree_followups 沉澱(晨間 MPSC/MPMC 九題+stepper 導覽)|SCHEDULE v9 時間模型|spsc 空白 #2(10 分/首編 4 錯全手滑,達標)|e2#1 複核(兩洞皆無網→補紅測×2 先紅後綠,oracle 開燈)|b#1 補課完結(紅測×3 mutation 驗+新抓④⑤⑥三洞全修 0.40→0.10s;loom 三變體佐證)|signal_pipeline 頁大改(fence 全套)|telemetry_aggregator drill 新增|hepta 卡沉澱|作息梯度定案。**晚(在家)**:aggregator 填綠(45m)→ h timer 快寫(30m)→ 30 秒口述出聲(鐵律英文)→ **02:00 熄燈/明早 08:45 起**。滑帳:🔴c#1 → 7/23 晚|卡#5 → 7/25 開場|executor → 7/23 白天連打|p8 → 7/24 | 白 ~5h + 晚 ~1.5h |
| **7/23 四** | **白天實績(至午後)**:pool 骨架默寫 rep#1 ✅(22m+3 輪修到 0 error;🔴主傷疤=退出條件 De Morgan ∧/∨ 三連翻,處方 loop+正面 break;⑤⑥④' 零提示寫對;批改紀錄留 `scratch/thread_pool.rs`,**7/24 開機重默 10m 驗秒殺**)|順帶入卡:lock-free MPMC 換得掉排隊換不掉睡覺;std::mpmc(nightly #126840)= 退出條件的 API 封裝|調序:先讀 SPSC/MPSC/MPMC(帶凌晨快考 Q1/Q4 兩洞)再 executor。**晚間追加實績 ✅**:executor challenge 完成(晚 7:30 留在公司寫,oracle 5/5;主洞=「poll 不准等」合約,詳 PROGRESS #6)。**晚場實績 ✅(22:10–02:00)**:mpsc_ring dif 表終版入檔(「交換位子」單日第 4 現身→編碼層處方:**seq 永遠單獨站等號左邊,名牌−票**;−cap 縫可達性+dif 地板定理自推)|mpsc stepper 7 幀 seq bug **用戶自抓**(發布後 seq 沒跳、還格 off-by-one),修正+重發 artifact;signal_pipeline 7/22 大改版補發(線上原是舊三節版)|🔴**c#1 oracle 6/6 一次綠**(commit f8a5e26;dry run 自攔 2 錯;遺留 may_compact 雙洞→7/24 紅測修)|**Q5 英文 30 秒 ✓ 結清 7/22+7/23 鐵律**(發現:L2 負載下 CAS 對象講反=最新學習最先掉,骨架重念)|signal_pipeline **深讀取代快走**(追問串:五睡法/futex-epoll 分界/喚醒鏈終點=IRQ/acq-rel 條件句+「最後一眼」原則/SB 兩 idiom/fence 四向牆/ordering 兩個讀者/shutdown 三語意;FAQ Q5 補單向牆視覺圖上線,light/dark 皆驗;6 卡直上 Heptabase——Notes×3+Memory Order×3,源 `scratch/hepta_20260724_fence_sleep_wake.md`)|drill 2 洞填、2/3 綠(**殘:Some 路徑摘牌+conservation ignore→7/24 開機 5m**)。滑帳:重打卡#2→7/24 開機塊;litmus 口述→7/24 晚口述段;h/aggregator 未動照舊。就寢 ~02:00(超線 30m,7/24 梯度照 01:00 歸隊) | 白 ~3h + 晚 ~3.5h |
| **7/24 五** | 白天(公司):**開機 pool 骨架重默**(10m 白紙,驗 7/23 兩條件傷疤;秒殺線=首編 ≤3 錯、兩條件一次對)→ **signal_pipeline drill 收尾**(5m:Some 路徑摘牌 2 行+拔 conservation ignore+跑綠)→ 重打卡#1+#2(各 5–15m;#2 自 7/23 滑入——7/23 深夜裁決:23:30 打符號提取題=假陰性,移開機)→ **修 c#1 的洞**(30m,紅測先行:先寫累積消費 >4096 後繼續 feed 的測試看 may_compact 炸,再修 drain/rebase+off-by-one)→ **日讀 p8 + 45 分鐘劇本六步**(c#1 已回放,合法;讀完當修洞底稿)→ 日讀 p6 + **lockfree 家族段**(qa_lockfree_followups 複讀 + upgrade-map §2/§7 + 頁尾表逐台 stepper,~60m)→(若欠)h 快寫/aggregator 補完。晚:🔴**e2#2**(45+20)→ 30 秒口述(SPSC→MPSC→MPMC)出聲 → **01:00 熄燈/08:15 起**。**開機暖手(用戶 7/23 拍板破例,寫 code 爽點)**:全空白重寫 spsc / pool / executor(各 ≤15m,scratch/,只求編過+smoke,不開 oracle)。**輕量語法熟悉(有上限,非全跑)**:mpsc_ring / mpmc_ring core 空白重寫(各 ≤20m,scratch/,熟語法用,卡住翻答案不深挖);hw_bridge challenge **只當語法暖身**(≤25m,不計時、不追全綠)——⚠ 這三項全排在 c#1 修洞 / e2#2 / lockfree 閱讀【之後】,是餘裕項,擠掉核心就砍 | 白 ~3.5h + 晚 ~1.5h |
| **7/25 六(上午 PT+剪髮;15:30 咖啡廳;晚在家出聲場——**v9.2 實況版**)** | **早上口袋件(候診/等位,手機,零壓力,做多少算多少)**:Heptabase 新六卡複讀(7/25 凌晨上板「Rust Low Level Notes」)+ PROMPTS_EN 卡題幹預讀(GitHub)+ ds_sync §8 先答再翻(claude.ai 鏡像,**手機瀏覽器開、不是 app**;讀不到 → 咖啡廳筆電補)→ **咖啡廳 15:30–19:00(打字+安靜件)**:**TCP 骨架讀+默寫**(10m 暖手,`rehearsals/examples/tcp_skeleton_std.rs`:讀一遍→白紙默 std 六行→對答案;d#1 前置)→ **aggregator 填綠**(45m,`drills/src/ds/telemetry_aggregator.rs`——**f 覆蓋帳最後一格關帳**;必含「未來 ts 清 window」case)→ **endian_pack drill**(40m,`drills/src/io/endian_pack.rs`;c#2 前鎖手感)→ **五卡完整流程用寫的**(卡1–4、6,各 ~10m:英文寫五問×2答案×2後果+30 秒定界,寫完開 clarify-answers 記漏類;+ **漏問模式表** 10m → `scratch/clarify_miss_pattern.md`,7/27 掃描+7/28 早上讀)→ **wheel 修綠**(30m,`scratch/timer_queue2.rs` 照檔頭批改,`rustc --emit=metadata` 驗)→ 晚餐+回家 → **在家 20:00–00:30(出聲場)**:**卡#5 口述設計版**(40m:sensor bridge threads/tasks+協定+五問,JD 複核 #4,**首做**)→ 🔴**e2#2**(45+20;三目標:clarify 英文出聲 ≥3 問|boundary 段跑滿〔e2#1 兩洞恰在沒跑到的角落〕|trade-off 招牌句;⚠ 參數名 `generation` 非 `gen`)→ 🔴**d#1 tokio_frame_server**(45+20,只跑一遍;d 題型首寫)→ **口述錄音 ~55m**(ordering / Waker 鏈 / 選型 + executor×reactor + 五 server p99.9 + unsafe impl 三段式 + litmus + signal_pipeline 扇入 + **Q1 why 層 30 秒英文複測**〔unconditional vs conditional claim,先講再對〕;30 秒光譜已 7/25 凌晨錄畢 ✓)→ 收帳 commit。**00:30 熄燈/07:30 起(v9.3:晨間動線彩排,詳 7/26)**。閥門:咖啡廳擠 → 先砍 wheel;晚場崩 → 口述縮 30m → 卡#5 縮 20m;**e2#2/d#1 不動** | 口袋 ~50m + 咖啡廳 ~3h + 晚 ~3.5h |
| **7/26 日(在家,彩排移白天;v9.3 晨間動線彩排 #1「寫」)** | **07:30 起 → 08:00 暖手:spsc 空白 #3**(20m 含 smoke,首編 ≤2 錯;自日中移入,暖手兼結帳)→ **08:45–09:30 🔴c#2 計時**(45m 釘上場時刻 + review 20m;c#1 7/23 → 間隔 3 天 ✓)→ 🔴**g#1 bounded_channel**(45+20,**取代 b#2**——Sender/Receiver 雙端+Clone 計數+Drop 協定+`SendError(T)` 是 bounded_queue drill 沒有的層;send/recv block+notify 順帶複驗 b 的 condvar/lost-wakeup 肌肉)→ 🔴**a#2**(45+20,自 7/25 移入,驗收斂)→ **d-std**(45m,寫或口述視狀態;動筆前 TCP 骨架重默 5m)→ recognition **e/f/h**(45m;g 免——剛全場)+ Q7 timer 口述 → **e 快寫 30m**(非計時,`rehearsals/src/event_registry.rs`:HashMap+`Box<dyn FnMut>`+retain_mut+After 寫到綠+自寫 smoke——**type erasure 新洞靶場**)→ 經驗故事 3 條(40m)→ 英文句庫唸出聲(30m)→ 讀自己的 challenge code(30m,縮)→ **00:00 熄燈/07:30 起**。閥門:累了砍序 = 讀 code → a#2(a 已 a#1+修洞兩驗) | ~7.3h |
| **7/27 一(請假,整天在家;v9.3 晨間動線彩排 #2「說」)** | 〔**⚠ 日程已被 v9.5 取代,見「進度校正 2026-07-27」節;本欄僅晨間動線時刻+taper 鐵規仍有效**〕**07:30 起 → 08:00 骨架默寫抽查 15m**(原 taper ② 釘進晨間格,白紙:spsc use 塊+impl、pool 兩條件、framer 簽名、**TCP accept-loop 六行、length-prefix 解析 3 行(checked_add→get→from_be_bytes+try_into)、token pack/unpack(mask 足 32 bit)**;骨架默寫 = 鐵規豁免項)→ **08:45 口述模擬一題 15m**(PROMPTS_EN 冷讀,挑 7/26 掃出的 ⚠ 題:30 秒英文定界 → 解法 arc → trade-off 收尾;**不計時寫題、不開 oracle——同時刻練「開口」不練「開 oracle」**)→ 之後 **Taper 升級版:全線回憶掃描**(7/22 定:時間多 → 從「空」升級,但鐵規不變:**不碰新題、不開 oracle、不計時跑題、不寫新 code**〔骨架默寫除外〕;卡住 → 記下、翻答案讀懂就走,**不深挖**)。①**九題型掃描** a/b/c/d/e2/f/g/h(每題 12–15m:讀 PROMPTS_EN 題幹 → **全程英文出聲**:30 秒定界 → 解法 arc+選型 → trade-off 收尾 ≥2 沒選解法+Big-O → 對分:`rehearsals/recognition-scripts-en.md`(**先講才准開**,口述版 sol_*)+ sol_*/漏洞卡 → 記 ✓/⚠/✗;請假在家 → 全程出聲為主,「在公司筆寫兩句」fallback 作廢)③**Heptabase 漏洞卡全翻**(每張 1 分鐘:當時錯什麼、修了什麼)④原 taper 收尾:背時間預算(0-3/3-5/5-10/10-35/35-40/40-45)+ 五 pillar + 開場三句 + 檢查 CoderPad/Meet/耳機/水。**產出:「認題→開場」檢查表(題型\|定界句\|選型\|trade-off 兩句\|我的傷疤),7/28 早上暖手就讀它**。⚠/✗ 超過 3 題不是加班訊號,是「靠已會的 80% 打」的提醒;**請假多出的時間預設 = 休息與睡眠存款,不是加練——taper 總量 ~3.5h 不變**。**23:00 熄燈不動** | ~3.5h |
| **7/28 二** | 〔7/27 晚改版〕**8:00 起床 → 晨讀本 `scratch/recall_checklist.md` 45 分鐘動線**(§0 分鐘表:默寫暖手 12m → 九題+金句出聲 10m → 鐵律 3m → 裝備)→ **8:45–9:30 TPS**。7:45 起床才解鎖加碼默寫區 | — |

彩排間隔(同題 ≥3 天,近了是背答案):a 7/19→7/26|b 7/20 一遍+補課完結(b#2 砍,condvar 肌肉由 g#1 側驗)|e2 7/21→7/25|c 7/23→7/26|d 7/25 一遍|g 7/26 首跑。
SPSC 空白 20 分鐘一次編過 ×3:**7/19 / 7/22 / 7/26**。

**彩排覆蓋帳(7/21 裁——「每個題型至少親手寫過一次」)**:a=a#1✓|b=b#1✓|c=framer drill✓+c#1|d=d#1+d-std|e=**e2 即其進階版**✓|f=**7/24 aggregator 延伸即 f contract**|g=**bounded_queue drill 即 g**✓|h=**7/24 快寫 30m 補上**(唯一沒寫過的)。e/g 不升全程——那是練已經最強的地方;g 的 lock-free 版不寫:block-on-full 是等待問題,condvar 繞不掉,try_push 版 = spsc_ring 本人(會講即可)。**⚠ 7/25 凌晨修正**:「e2=e 進階版」只在 JD/token 軸成立——e 的 `Box<dyn FnMut>`+After+retain_mut(**type erasure 軸**)e2 沒有 → e 補快寫 30m(7/26);g 彩排版多 Sender/Receiver+Drop 協定層(drill 未覆蓋)→ **g#1 升全場**(7/26,取代 b#2)。

## 每日輸入(當天要開的檔案)

v8.1 規則 5 的操作版:動手到哪開到哪,這張表就是「當天該開哪幾份」。
彩排/卡片題幹一律 `rehearsals/PROMPTS_EN.md`(規則 6);`sol_*` 與 `clarify-answers.md` 寫完才開。
已 publish 到 claude.ai 的鏡像(通勤讀)在 artifacts gallery:claude.ai/code/artifacts。

| 日期 | 要開的檔案(依當天順序) |
|---|---|
| **7/17** | 掀牌:`drills/src/concurrency/bounded_queue.rs`(移 `#[ignore]`)→ 卡#2:`rehearsals/PROMPTS_EN.md`(Card 2)+ `rehearsals/clarify-cards.md`(規則 5 自我批改)→ `drills/src/concurrency/thread_pool.rs` → `drills/src/concurrency/spsc_ring.rs` + 三段式 `docs/rust-five-axis.md`。日讀:`docs/artifacts/bounded_queue.html` → `docs/artifacts/ring_buffer.html`(§5 len 之死,spsc 前置)→ `docs/artifacts/spsc_ring.html` → `html_p/p1-rust-atomic-spsc-ordering.html`(追問鏈 + quiz) |
| **7/18** | 卡#3(同兩檔)→ `challenges/src/concurrency/spsc_ring.rs` + `reference/tests/loom_spsc.rs` → `drills/src/runtime/executor.rs` + `challenges/src/runtime/executor.rs` + `docs/artifacts/executor.html` → Q7:PROMPTS_EN(h 題)→ `drills/src/io/hw_bridge/`。日讀:`html_p/p2-async-executor-handbook.html` |
| **7/19** | 卡#4 → a#1:PROMPTS_EN(a 題;寫完才開 `rehearsals/examples/sol_ring_drop_oldest.rs`)→ `docs/io/fd_registry.md` + `drills/src/io/fd_registry.rs` → spsc 空白(白紙) |
| **7/20** | 卡#5 → 修 a#1(照漏洞清單)→ b#1:PROMPTS_EN(b 題)。通勤:`docs/io/event_loop.md` + `docs/async/mini_runtime.md` + `docs/artifacts/event_loop.html` + `html_p/p3-epoll-readiness-event-loop.html` |
| **7/21** | 卡#6 → e2#1:PROMPTS_EN(e2 題)→ 修 b#1。晚加碼:`challenges/src/runtime/executor.rs`(空白手搓,tests 在 `challenges/tests/executor.rs`)→ `docs/cost-model.md` →(選)`docs/concurrency/thread-safe-spectrum.md` / `scratch/capacity_reps.md` |
| **7/22** | 實績:`drills/src/ds/{lru,telemetry_aggregator}.rs`、`rehearsals/src/{fd_registry,pool_graceful_shutdown}.rs`、`reference/tests/loom_lost_wakeup.rs`、`docs/artifacts/{qa_lockfree_followups,signal_pipeline}.html`、`scratch/hepta_20260722_lockfree_day.md`;晚:`drills/src/ds/telemetry_aggregator.rs` + `rehearsals/src/timer_queue.rs` |
| **7/23** | signal_pipeline:`docs/concurrency/signal_pipeline.md` → `docs/artifacts/signal_pipeline.html`(7/22 大改版:fence 六題 FAQ + SB stepper)→ `reference/src/concurrency/signal_pipeline.rs`(`start_fan_in` 6 tests)→ `drills/src/concurrency/signal_pipeline.rs` → `challenges/src/runtime/executor.rs`(連打)→ 晚:c#1:PROMPTS_EN(c 題) |
| **7/24** | 修 c#1(照昨晚清單)→ 日讀:`html_p/p8-hw_bridge_teaching.html`(c#1 已回放)+ 45 分鐘劇本六步 + `html_p/p6-telemetry-spsc-ring-reference.html` → lockfree 家族段配 `qa_lockfree_followups.html` + `html_p/runtime-lockfree-upgrade-map.html`(§2/§7)→ 晚:e2#2:PROMPTS_EN |
| **7/25** | 卡#5 + 五卡全過:`rehearsals/PROMPTS_EN.md`(卡題幹)+ `rehearsals/clarify-cards.md`(規則)+ `docs/clarify-playbook.md`(五問方法論;寫完才開 `clarify-answers.md`)→ e2#2:PROMPTS_EN(e2 題)→ TCP 默寫:`rehearsals/examples/tcp_skeleton_std.rs` → d#1:PROMPTS_EN(d 題,tokio)→ aggregator:`drills/src/ds/telemetry_aggregator.rs` → endian:`drills/src/io/endian_pack.rs` → wheel:`scratch/timer_queue2.rs` → 口述底稿:`docs/concurrency/thread-safe-spectrum.md` + `docs/rust-five-axis.md` + `docs/io/hw_bridge.md`(五 server 對照組)+ `docs/async/async-runtime-anatomy.md` + `docs/cost-model.md` → `docs/artifacts/ds_sync.html` §8 先答再翻 |
| **7/26** | spsc 空白(白紙,08:00 暖手)→ c#2:PROMPTS_EN(c 題,08:45 計時)→ g#1:PROMPTS_EN(g 題)+ `rehearsals/src/bounded_channel.rs` → a#2:PROMPTS_EN(a 題)→ d-std 前重默:`rehearsals/examples/tcp_skeleton_std.rs` → e/f/h 題幹:PROMPTS_EN + 認題掃描表 `rehearsals/README.md` → e 快寫:`rehearsals/src/event_registry.rs` → 英文 I/O 唸出聲:PROMPTS_EN + `docs/clarify-playbook.md` → 讀自己的 `challenges/src/` |
| **7/27** | 掃描:`rehearsals/PROMPTS_EN.md`(九題題幹)+ `rehearsals/recognition-scripts-en.md`(**先講才開**)+ `rehearsals/examples/sol_*.rs`(**對分才開**)+ Heptabase 漏洞卡(e2 兩洞/b 三紅/lru 兩洞/c 的洞)+ `rehearsals/README.md`(protocol+時間預算)+ `docs/coderpad-constraints.md`;產出檢查表寫 `scratch/recall_checklist.md` |
| **7/28** | 暖手小 drill 一題 + pillar-5 dry-run 清單(`rehearsals/README.md`)→ 上場 |

---

## 進度校正(2026-07-18 實況)

**實際完成**:ring_buffer(讀+drill)、iter_mutate(drill)、bounded_queue(drill,`#[ignore]` 已移)、**thread_pool(drill 8 綠——親手抓 3 個 bug:Drop 漏 join / 鎖圈住 job 執行 / submit lost wakeup)**、**spsc_ring(drill 3 綠,`#[ignore]` 已移)**、卡#1、ring_buffer artifact 補記憶體排版圖。

**落後**:spsc challenge、executor、卡#2–#3、P1/P2 日讀 尚未動 → 約落後 1 天;主因時間偏向「讀 + 補教材」(規則 #5 提醒:動手到哪開到哪)。

**吸收決策**:動用砍序 ① —— **砍 d(tokio 彩排)**,7/19 起騰出來補 spsc(脊椎:JD ②封包流 + ③lockless,第一優先);executor 順延;fd_registry(e2)兩遍不動。

**明日(7/19)待讀**(spsc drill 今日已補完、騰出時間):P1(atomic ordering)、P2(async executor),以及新增的 Q&A 圖解頁(sync/async oneshot、lock-free P/C、tail latency;atomic ordering 頁製作中)。

---

## 進度校正(2026-07-19 實況)

**完成**:hw_bridge framer drill 10 綠(含壓實 counterexample,commit 2cf0288)、卡#2 重寫+批改(漏 SLA 已補;容量帳特訓 3h)、🔴a#1 首跑(oracle 4 紅→當晚修綠 5/5;boundary 段跳過=全數 pillar-5 miss,漏洞卡上白板)、spsc 空白 #1(20m 超時,當晚磨到 0 錯 smoke pass;7/22 #2 目標 20m ≤5 錯)、fd_registry 讀+drill 晚間補上(回歸原排程)。

**裁決**:卡#3 拿掉(conflation 已深學,7/25 快打認題帶過)|卡#4 → 7/20 與 #5 併日|executor challenge + Q7 timer 口述 → 7/21|deep-dive 情報(主管面,非技術深挖)→ 7/25 已拆 75m+45m。

**7/20 修訂**:卡#4+#5 → a#1 補課(自寫紅測 ×3 + 英文 trade-off 口述,60m)→ 🔴b#1(45+30m)(fd_registry 已於 7/19-20 凌晨全綠,commit 16c7ab1)。通勤:event_loop / mini_runtime。開機默寫:use 塊 + impl<T>。

## 進度校正(2026-07-20 白天實況)

**白天完成**:開機默寫 spsc 骨架 35→12→3→0(use 塊/impl&lt;T&gt;/&amp;self+UnsafeCell 三類肌肉傷全清,剩拼字手滑;存底 `scratch/skeleton.rs`,commit 5387a04)|drills 排雷:本機 1.91 vs pad 1.92 版本坑,hw_bridge `as_array`→`try_into`(5ee4978),四道閘回綠|a#1 補課:自寫紅測 ×3 全綠 + condvar 屍體清完(fb52b5c;**回放三輪 + 英文口述移晚上**)|event_loop/mini_runtime 原「通勤略讀」升級為桌前深讀 Q&A(eventfd/token/signal/timer/offload 全線打穿),沉澱 `qa_eventfd_doorbell.html`(Q&A 區 12→13)+ `scratch/hepta_eventfd_doorbell.md`。

**系統性修正**:**通勤讀槽全部作廢**(騎機車,已入 memory)——今後「通勤讀」一律改排桌前或砍;7/22 p8 只剩「c#1 回放後晚尾」一個選項。

**晚上(回家)待辦**:卡#4(15m)→ **卡#5 sensor bridge 升級為完整口述設計**(25–30m:分 threads/tasks + 定通訊協定 + 五問——JD 複核 #4「HW-adjacent system design」的落地,不只答 clarify)→ a#1 回放三輪(10m,改壞→紅→還原)→ 🔴b#1(45+30m)→ 英文口述一場錄完(a#1 trade-off + b#1 收尾合併)。b#1 今晚必跑——7/21 的「修 b#1」依賴它;真不行則修 b#1 滑 7/22(仍有 5 天間隔,安全)。

## 進度校正(2026-07-21 凌晨實況,7/20 場收尾)

**完成**:a#1 全收工(含回放三輪)|🔴**b#1 已跑**(2 綠 3 紅,帳在 PROGRESS 計時表:core 溢時 +13 吃掉 boundary/收尾;時限內自抓 join 漏;當場診斷清 queue + wait predicate 兩洞)。**未動**:卡#4、卡#5、英文錄音(半夜錄音價值低,裁掉併後)。

**7/21 修訂**:卡#4+卡#6 開場連打(15+15)→ 🔴e2#1(45+30)→ **修 b#1 補課**(每洞紅測先行 → 綠;含 shutdown 側 store+notify 有無 mutex 同步的 lost-wakeup dry-run;回放;英文 trade-off 收尾補錄——b#1 沒講到的那 5 分鐘)→ executor challenge **滑 7/22 晚尾**(p8 前)。卡#5 口述設計版 → 7/25 開場(7/22 再滑,deadline 內)(deadline 7/25 前不變)。

**7/21 晚加碼(e2#1 零紅收工、有餘力,晚間定)**:executor challenge **拉回今晚**(60m,7/22 晚尾釋放給 p8+緩衝)→ cost-model.md 讀 15m(數字錨,直餵錄音)→ 錄音照舊。22:30 檢查點後選配:thread-safe-spectrum 讀 30m(7/25 底稿提前)或容量速算 3 題(scratch/capacity_reps.md,結帳表紀律)。**23:00 硬熄燈,明天 4.5h 不准借。**

**收尾補記(7/22 ~01:00,加時場帳)**:23:00 熄燈規則**破戒 +2h**——四輪加時談判(12:00 → 還能寫誰 → trie 盒 → lru)是 7/20 模式重播,記錄在案。實收:**trie drill 填綠**(2/2 + clippy/fmt;`?` 對 Option 自 1.22、get_or_insert_with 二次借用地雷 → 卡 `208b2fde`)+ **lru drill 3/3 綠但 review 抓 2 洞**(連三場「零紅≠零洞」):①put 淘汰路徑缺 `map.insert(新 key)` + promote(repro:cap=2 put a/b/c → get c = None;len 縮水;下次 put 反淘汰新 key)②unlink 頭/尾分支殘留鄰居髒指標(暫被「push_front 先 unlink 且立即重接」蓋住,單獨用 unlink 即炸)。**7/22 白天公司修,紅測先行×2**——規則 2 這次從紅開始走,TODO 已標檔內。meta 教訓三連:「我的測試只驗我想到的事」→ 併 7/25 口述。**今晚(7/22 晚)23:00 規則無談判**——c#1 的品質直接由睡眠付帳。

**收尾補記(7/21 晚 23:10 定帳)**:今晚實收:**鐵律錄音還清**——a#1 四軸 + b#1 五軸合場(照 trade_off_map_ab.md)+ e2#1 30 秒 trade-off 句(兩天欠帳結清;鐵律:code ✓ e2#1+修洞、英文 ✓)|cost-model 速讀 ✓|**兩張漏洞卡入 Heptabase「Rust Low Level Notes」**:e2#1 兩洞卡(`c84f43c4`)+ b#1 三紅卡(`e2eb0dfb`,內含 lost-wakeup **預測題**——shutdown 側 store+notify 沒拿 jobs mutex,7/22 dry-run 先手走再翻答案)|錄音中場概念收穫:**drop-oldest 併發化 = producer 兼職 consumer → 退化表 SPMC 格**(policy 決定同步結構;a 段第 4 軸升級句已錄);seqlock 覆寫未讀 → 誠實邊界句處理,材料併 7/24 lock-free 升級地圖日讀|**滑走**:b#1 紅測×3 + dry-run + 回放 → 7/22 與 e2#1 複核合段(~50m);executor challenge → 回 7/22 晚尾原槽|⚠ **7/22 負載超標**(卡#5 30 + c#1 75 + 合併段 50 + spsc 20 + executor 60 + p8 ≈ 4h+ vs ~3h):預設砍序 **p8 → 7/23 通勤**;executor 仍最後一位,再塞不下 → 7/23 開機默寫後連打(默寫同肌群;留存測試 7/23 是底線,再晚意義衰減)|journal 已同步:e2#1 錄音+漏洞清單 ✓、b#1 每條洞怎麼修 ✓(卡片即帳)、Q7 timer due 改 7/26。

**收尾補記(7/21 ~2:30am 定帳)**:🔴b#1 凌晨已修至 **oracle 5/5 全綠(驗過)**——明晚「修 b#1」只剩補課(紅測×3/dry-run/回放/錄音),不用再修 code|主管面 45m 塊移 post-TPS(7/28 只有 TPS)|**平行大產出入帳(user 自寫)**:`ds_sync` 對照組(reference 4 模組 + loom_dsu + 教學頁,已 publish ⚖️)+ 全 repo 重構(ds/ 子目錄、docs 四分類)——**此後開檔按新路徑**|凌晨概念課沉澱:`scratch/trade_off_map_ab.md`(a 四軸/b 五軸/lock-free 應對/fan-in),**7/25 錄音直接照它講**|**鐵律檢查:今天打 code ✓(b#1+修洞),張嘴英文 ✗**——錄音兩天欠帳,明晚必還|明晚睡眠債下的預設砍序:executor challenge 第一個滑(閥門已設)。

## 進度校正(2026-07-23 凌晨)

**7/22 晚場沒跑**:aggregator 填綠 / h timer 快寫 / 30 秒口述全數未動——疲勞優先,00:40 就寢裁決(早於 02:00 線;R1 死因是疲勞,今晚睡飽就是 c#1 的品質投資)。滑帳照既有條款:**h/aggregator → 7/23 白天餘裕、7/24「(若欠)補完」槽;h 優先於 aggregator**(h 是覆蓋帳唯一沒親手寫過的題型)。英文鐵律 7/22 ✗ 記帳,7/23 晚 c#1 narrate(本就英文)償還。

**流暢度複核(7/23 凌晨,對「彩排題全部寫順不用查」目標)**:a/b/e2/f/g 走完既排 reps 可達;c 靠 c#1(7/23)→修(7/24)→c#2(7/26)成環;**h、d 到不了順寫、也不需要**(情報 #2/#3:題目規模 = a/b/c/e2 量級,h/d 是保險層,taper 原則=靠已會的 80% 打)。點名的缺口補位四項:

1. **TCP 骨架默寫**:`rehearsals/examples/tcp_skeleton_std.rs` 新增(std 六行 + tokio 對照,gates 編譯防爛)——7/25 讀+默寫 10m(d#1 前置)、7/26 d-std 前重默 5m、進 7/27 抽查。
2. **endian_pack drill**:`drills/src/io/endian_pack.rs` 新增(BE/LE 讀寫、手動 shift、i16 符號擴展、token pack/unpack 直打 e2 mask 傷疤、混合 header;8 洞 6 測)——7/25 40m;與 c 成環:c#1 診斷 → drill 治療 → c#2 驗證。附贈 pad 坑:**`gen` 是 edition 2024 保留字**,e2 場上用 `generation`。
3. **英文認題稿**:`rehearsals/recognition-scripts-en.md` 新增(a–h 八題型:定界宣言+clarify 問+做法枚舉+trade-off 收尾+傷疤句;a/b/c/e2 詳、d/f/g/h 簡)——7/27 掃描的對分底稿,**先講出聲才准開**,規矩同 sol_*。
4. **Heptabase**:scratch 兩份沉澱(eventfd 門鈴、lockfree day 卡1–卡8)拆 8+1 張推板(Notes / Memory Order notes / Low level learning);journal 7/22–7/28 推新版,舊條目 Withers 自刪。

**凌晨快考帳(01:50,SPSC/MPSC/MPMC 五題 10m)**:Q2 全對|Q3 缺「seq 歸零 → dif=−cap 永遠滿」半題|**Q1/Q4 同源洞:list-swap vs ring-CAS 原語混淆** + slot seq 誤記 SeqCst(正解 Acq/Rel)。7/23 待還:白天翻卡4 一分鐘 → 晚上 Claude 複測 2–3 題(先答再批)→ **Q5 英文 30 秒**(SPSC→MPMC 升級)併 c#1 場出聲,連 7/22 英文鐵律一起結。就寢裁決:02:30 熄燈 / 09:00 起(保 6.5h 紅線,梯度慢半階,7/24 歸隊)。

## 進度校正(2026-07-24 白天+傍晚,實況)

**白天打字場實績(全在「有打 code」格)**:
- ✅ **c#1 may_compact 雙洞修畢**(紅測先行:餵「累積消費 >4096 後繼續 feed」看 underflow → `drain(..self.ptr); self.ptr=0` 兩洞一起死)。mutation 驗:主洞被咬、off-by-one 因**全 0 payload 漏網** → 改 `[20]` payload 才咬住(「測試只驗想到的事」家族)。
- ✅ **signal_pipeline drill 收尾 3/3 綠、0 ignored**(Some 路徑摘牌:early return 別跳過 store(false) + 拔 conservation `#[ignore]`)。
- ✅ **pool 骨架重默 → 升級完整版**(execute + submit 回傳 + JobHandle + oneshot + panic 隔離 + graceful Drop;編過+複核正確,scratch/thread_pool2.rs)。傷疤(worker 三分支 De Morgan)**癒合**;新洞 = **type erasure**(pool 對 job 型別是瞎的,泛型放方法非 struct)。沉澱 3 條通用規則(型別擦除 / condvar 鐵律 / 跨邊界側信箱)。**Claude 給錯碼表教訓**:把 drill 進階版當 rep#1 spec、壓 10m;rep#1 真 scope = 射後不理版(~40 行)。
- ✅ **timer_queue(h)寫成——彩排覆蓋帳最後一格補上**(h 是唯一沒親手寫過的题型)。min-heap 版 `Reverse<(deadline,id,interval)>`;tie-break 傷疤(原次鍵誤放 interval → 排序錯,紅測先行修);**加碼 lazy-delete `del_id`**(HashSet 墓碑 = heap-cancel 標準做法)。
- ✅ **timer_queue2 wheel 第一版**(scratch,單層 hashed wheel + rounds)——11 error + len 沒記 + next_deadline 比較鍵,**批改寫進檔頭,7/25 回家修**;方向全對(絕對 slot / extract_if 單趟 / drift-free 重排)。
- ✅ clarify 材料補:submit-after-shutdown 三分法進 `recognition-scripts-en.md` b 段(7/27 掃描料)。
- ✅ hepta 2 卡備:`hepta_20260724_threadpool_full.md`(7 卡 + catch_unwind 4b)、`hepta_20260724_timer_wheel_qa.md`(7 卡)。

**Q&A 深潛(回家複讀,已入卡)**:type erasure|lost-wakeup ≠ 可見性(acq/rel 救不了,loom 親證)|oneshot promise|panic 隔離三件套|`catch_unwind(AssertUnwindSafe)` 語法|`thread::Result` vs `Result`|timing wheel(tick/SLOTS 選型、hierarchical)|Vec `retain`/`extract_if` 的 O(n) 壓實|heap 刪除(pop/rebuild/懶刪除/indexed/sift)|`sort_by_key` vs `impl Ord`。

**未跑 / 待辦**:晚上出聲場(litmus 口述 + 🔴e2#2 + 30 秒口述)——騎車回家未跑|wheel 修綠(7/25)|`scratch/skeleton.rs` 出現刪除(`D`,非 Claude 動作)+ `.claude/` 未追蹤 → 本次 commit **未納入**,待用戶裁。

## 裁決(2026-07-25 01:50,用戶拍板——7/25 升 v9.1「不砍全排」)

- **7/24 晚出聲場整場滑帳**(騎車回家未跑):e2#2 / litmus 口述 / 30 秒光譜 / lockfree Q1/Q4 複測 → 全數併入 7/25。Claude 提的砍法(a#2 移 7/26 換 b#2、口述縮 45m)**用戶否決**:9h 全排,7/26 表不動(b#2 保留原「累了先砍」閥)。
- **卡片線升級**:六卡「快打」→ **全副認真過**(卡#5 口述設計版首做 40m + 其餘五卡各 ~10m 完整流程),主題=「學怎麼問、問哪些」;新產出**漏問模式表**(五類統計,`scratch/clarify_miss_pattern.md`)餵 7/27 掃描 + 7/28 早上暖手。
- **覆蓋帳實況(7/25 凌晨盤點)**:aggregator 仍 2 `todo!` / 5 測紅——f 是最後一格,7/25 關帳;d 題型仍零覆蓋,d#1 今日首寫。
- 三場計時同日(e2#2/d#1/a#2)是本輪首見,間隔各 ~3h+;閥門寫在日表尾。
- 就寢實況 ~02:00;起床 08:00(原表)vs 08:30(保 6.5h 紅線)用戶自裁。

**第二輪(~03:00,連環拍板後 v9.2 定案)**:

- **實況修正**:7/25 上午物理治療+剪頭髮,15:30–16:00 才到咖啡廳 → 9h 單場改 **v9.2 三段式**(早上口袋件/咖啡廳打字場/在家出聲場),日表已改。三場計時剩 e2#2+d#1(a#2 移 7/26)。
- **b#2 砍定案,位子給 g#1 bounded_channel 全場**(用戶要作;題面驗過:雙端+Drop 協定層 drill 沒有,且順帶側驗 b 肌肉)。
- **e vs e2 軸修正**(用戶質疑觸發,見覆蓋帳 ⚠):e 補快寫 30m(7/26)。
- **凌晨實績**:30 秒 SPSC→MPSC→MPMC 英文稿批改後**錄畢**(7/24 板欠帳清)|lockfree Q1/Q4 複測:**Q2 過**(seq Acq/Rel + pos Relaxed + 無 SB pattern 故無需 SeqCst)、**Q1 半洞換形態**(原語層癒合;why 講成 atomicity/contention,正解 = **unconditional vs conditional claim**)→ 7/25 晚口述段複測 why 層|hepta 兩份沉澱**壓縮 15→6 卡上板**「Rust Low Level Notes」,ID 回填源文件|TCP 默寫質疑 → 確認 `tcp_skeleton_std.rs` 已在(7/23 建,101 行),讀→默→對流程不變。
- `.claude/`(settings.local.json)入 .gitignore——本地設定不進 repo;skeleton.rs 刪除已不在 working tree,7/24 待裁項雙雙結案。
- **復盤協議(用戶立)**:每個 block 收尾報「實際 vs 排定」,超了當場商量。

## 裁決(2026-07-25 午後,用戶拍板——v9.3 晨間動線彩排 ×2)

- **7/26、7/27 連兩天照 7/28 上場動線跑早晨**:07:30 起 → 08:00 暖手 → 08:45 開跑。身體先走兩遍同一條晨間動線,7/28 是第三遍。
- **7/26 = 寫的模擬**:08:45 釘 🔴c#2 計時 45m(本來就是當日第一場,**不加量,只釘時刻**);08:00 暖手 = spsc 空白 #3(自日中移入)。起床 07:45 → 07:30(7/25 00:30 熄燈不動,睡 7h ≥ 6.5h 紅線 ✓)。
- **7/27 = 說的模擬**(taper 鐵規不變):08:00 骨架默寫抽查(taper ② 釘進晨間格,鐵規豁免項)→ 08:45 口述模擬一題(PROMPTS_EN 冷讀 7/26 的 ⚠ 題)。**前一天跑計時模擬考被否**:考差沒時間修、只傷信心——同時刻練「開口」,不練「開 oracle」。
- **7/27 已請假,整天在家**:九題型掃描全程出聲為主(「在公司筆寫」fallback 作廢);多出來的時間**預設是休息與睡眠存款,不是加練**——taper 總量 ~3.5h 不變。

## 進度校正(2026-07-26 凌晨,7/25 收帳)

**咖啡廳段(排 15:30–19:00,實 ~15:40–19:20)**:早上口袋件未做 → 開場補課:§8 冷診斷 **1✓/2✗/3半/4半**(Q2/Q4 = 軸認錯不是不會;題幹預讀砍——併五卡流程)+ 六卡複讀 ✓。**TCP 骨架默寫 rep#1:6 輪 7 洞 → 0**(五條傷疤 + 逐輪帳在 `scratch/tcp_skelton2.rs` 檔頭;7/26 d-std 前 5m 重默驗收)。**aggregator drill 6/6 綠**——自寫紅測抓「同餘撞桶」鬼資料(提供測試漏網;`Bucket::empty` min/max 哨兵修正)→ **f 覆蓋帳關帳,九題型全部親手寫過**。咖啡廳 Q&A 七卡沉澱 `scratch/hepta_20260725_cafe_qa.md`(a479539,脊椎=「這個寫入承不承載不變量」)。滑帳:endian → 7/26 08:20|wheel 修綠**陣亡**(post-TPS)|五卡 → 口述化,見 7/26 修訂。

**晚場(排 20:00,實開 22:33)**:剩 2h,照閥門精神手術——卡#5 / 錄音段 / signal_pipeline 翻讀全滑 7/26;**e2#2 / d#1 保住**:

- 🔴**e2#2**(22:40–23:13):**部分重寫**(Token impl 與測試繼承 e2#1,diff 35+/55−,收斂訊號打折記帳)。oracle 5/5 綠,但**繼承的自寫紅測抓到 e2#1 洞① 回鍋**——len/gen bump 未押在 `take()==Some` 之後(初版 unregister 連 generation 都沒驗,`_generation` 自首)。「修洞必寫 counterexample」規則 4 天後自動放哨抓洞 = 規則 2 的複利首例;傷疤「**狀態變更押在確認移除之後**」記**未癒合**。clarify 三問英文 ✓(fd 回收 = 錢問題有問到);定界宣言太薄(一句就開寫)。
- 🔴**d#1**(23:15–00:00):core 15m,review 抓三大洞——**idle_timeout 整條蒸發**(clarify 沒問到的需求恰是掉的需求 → 處方:動筆前 clarify 清單對讀需求清單 30 秒)/ **echo 掉 wire format**(只回 payload 沒回 header)/ **自測零條**(boundary 又跳,e2#1 死因重演);另自測寫死 port `AddrInUse`(`:0` 肌肉當天教、當天沒用上)。修後自測 2 綠 + **oracle 6/6 一次綠**,d 題型首寫入帳。亮點:`break 'parsing` 帶標籤跨巢狀迴圈一次寫對;parser 重用裁決正確。
- 🔴**流程頭號傷疤:「喊綠沒驗」×2**(e2#2、d#1 各一)→ 新鐵律:**說「綠」之前,終端機裡要有那行 `test result: ok`**。
- 鐵律結帳:code ✓(aggregator/TCP/e2/d)、英文 ✓(兩場 clarify + narrate 全英)。Q1 why 層複測:00:00 起休息,結果 7/26 晨補記;executor clarify #5 → 7/26 口述塊。

**7/26 修訂彙整**(v9.3 晨間動線不動,微調):08:00 spsc 空白 #3 → **08:20 endian_pack 壓縮 25m** → 08:45 🔴c#2 → 🔴g#1 → **a#2 降級 overflow**(改 5m 口述快掃;白天跑得快才撈回)→ TCP 重默 5m + signal_pipeline 翻 10m → d-std → **f#1 計時 30+10 新增**(`rehearsals/src/telemetry_aggregator.rs`;⚠ 間隔 1 天 = 形狀鞏固非收斂訊號,review 只對 reference 載重差異)→ recognition e/f/h(f 份額 = 10m drill vs reference diff-read)+ **ds_sync 補洞環**(讀 code+html 30m 硬上限插 c#2 後或午後;下午閉卷重烤 15m + transfer 變體〔Vyukov seq/pos、e2 generation〕+ CLOCK 最壞掃描題)→ e 快寫 30m → 經驗故事 → 英文句庫 → **卡#5 口述設計版(佔原「讀 code」槽)** + 卡1/卡2 口述重打 15m + 錄音殘項(litmus/扇入/五 server/光譜/unsafe 三段式/executor clarify #5)→ 漏問模式表 10m → 00:00 熄燈。超載砍序:**ds_sync 補洞環 → f#1 → 卡1/2 重打 → 錄音殘項壓縮**;c#2/g#1/d-std/e 快寫/卡#5 不動。

**00:52 起床時刻改判**:實際就寢 ~01:00(收帳+上板拖長),07:30 起僅 6h20m **破 6.5h 紅線** → 起床改 **09:00**(用戶要八小時,10:00 被否——週一 07:30 定錨日會變一步跳 2.5h,違反 30 分/天梯度;09:00→07:30 = 1.5h 勉強可守)。晨間動線整段平移、offset 不變(起+75m 開跑,同構 7/28 的 07:30→08:45):**09:00 起 → 09:30 暖手(spsc #3 + endian 25m)→ 10:15 🔴c#2 計時**。全天後移 ~1.5h,晚段擠壓照既定砍序;**7/27 07:30/23:00 不動**(上場前唯一定錨日)。

**00:35 用戶點名複核**:e 從頭寫 = e 快寫 30m ✓ 已在表|f 從頭寫 = f#1 ✓ 已在表|g = g#1 全場 ✓ 已在表(PROMPTS_EN 舊注「recognition 級」已更正)。**signal_pipeline 動手版 → overflow 池第 2 位**(ds_sync 之後):20m fan-in 骨架快默(scratch、非計時,只默簽名+fence 擺位),白天超前才碰;口述層(litmus/扇入)照原排,challenge 仍 post-TPS。

## 進度校正(2026-07-27 凌晨,7/26 收帳 + 7/27 改版 v9.5)

**7/26 實況**:起床 ~11:30(排 09:00)——晨間動線彩排 #1 的時刻錨失效,內容照序全跑,「不動」名單 **5/6**(唯一沒中=卡#5 第五滑):

- 暖手三件 ✓:spsc 空白 #3(**首編 0 錯**,曲線 35→4→0;25m 超線+smoke 補寫;座標系對調發現)|endian 6/6|TCP 重默 #3(0 錯,實質 1.5 洞)
- 🔴**c#2 ✓ 結案**(oracle 6/6 一次綠、30m、零洞;may_compact 傷疤癒合;評級 H 摸 SH 邊——差距=boundary 不自燃+trade-off 兩輪才乾淨)
- 🔴**g#1 ✓**(46m;oracle 4 紅同根=recv 不 drain→修畢 6/6×3;review 兩洞:雙 Drop「store 不開燈」+缺 mutex 括號〔b#1 lost-wakeup 本尊〕、「沒 join 的斷言不是斷言」)
- **f#1 ✓**(4/5→lazy validation 修畢 5/5×3+回放咬洞;**boundary 首次全自發**——self-ignition n=1)
- e 快寫 ✓(retain_mut 進肌肉;真洞=per-id 漏讀)|教材:signal_pipeline **IRQ 喚醒鏈**節 md+html+鏡像補發|`scratch/sprint_summary_for_chat_20260727.md` 產出
- 滑帳:卡#5→7/27 綁 executor #5|recognition e/f/h→併快版掃描|a#2 口述/signal 翻讀/漏問表/hepta 上板→7/27|經驗故事→post-TPS|ds_sync 環→砍
- 鐵律:code ✓×4、英文 ✓×3 場|trade-off 三拍公式定版(價格→沒走的路≥2 條用軸開頭→有效範圍)|就寢 ~00:40(超線 40m:收帳+summary+排程改版)

**7/27 改版 v9.5(chat-Claude 提案 + 兩修正,取代 v9.3 欄的日程;晨間動線與鐵規不變)**。核心論點:「35 分那格自己站起來」的 self-ignition **n=1**(僅 f#1),是規則不是習慣——要推到 n=4;缺的不是題,是那個轉場動作的第 2/3/4 次:

1. **07:30 起 → 08:00 骨架默寫抽查 15m**(原清單 + **bounded_channel 雙 Drop 六行**:store/sub→拿鎖放鎖→notify)
2. (選配)**d-std 非計時暖手,25m 硬上限**——7/24「寫 code 爽點」條款;不想寫直接砍,是獎勵不是義務
3. **三份首跑最爛舊解冷讀口述 dry-run 45m**(a#1/d#1/g#1 各挑一個**非原點邊界**;不開 oracle、不改 code、不計時;**轉場句 "now let me dry-run the boundaries" 要自己說**——那個動作就是在練的 rep,Claude 不提醒)
4. **九題型快版掃描 60–75m**(每題 ~7m:英文定界 30 秒 + arc;對 `recognition-scripts-en.md` **先講才開**;記 ✓/⚠/✗ 餵檢查表)
5. **卡#5 sensor bridge 完整口述設計 + executor clarify #5,綁一場 45m**
6. **漏洞卡全翻 + 漏問模式表合併 20m** → **認題檢查表 15m**(`scratch/recall_checklist.md`,7/28 早上暖手就讀它)
7. **日讀:`html_p/rust-static-lifetime.html`**(~20m,7/27 凌晨新入庫的 'static/lifetime 深讀——user 點名排入;taper 合法閱讀件)
8. 總量 ~3.5–4h(選配 d-std 才到 4h+);**23:00 熄燈不動**;taper 鐵規不變(不寫新題/不開 oracle/不計時跑題)

## 進度校正(2026-07-27 晚,taper 收帳 + 情報 #4)

**實況**:晨間動線 #2 沒跑(14:35 開工);taper 核心全數完成,部分改形:

- **骨架默寫抽查 ✓**(1✓/5⚠/1✗:✗=length-prefix `usize::from_be_bytes` 真洞〔正解=u32 解再 `as`,wire 型別決定陣列長〕;Sender Drop `==1` 方向默反一次〔==1 的人要關燈〕;⚠ 餘為 compile 級——全數當場修對,批改在 `scratch/recall_20260727.rs` 檔頭)
- **h 口述模擬 ⚠**(heap/Big-O/升級路全對;掉兩句招牌:drift-free reschedule + wait_timeout re-checked predicate → 晨讀本複誦)
- **三舊解 dry-run ✓**(a#1 wrap+滿載傷疤路|d#1 idle_timeout×半 frame——磨出「idle 答 peer 活沒活、frame-age 答 frame 拖多久,不同問題不同 timer」|g#1 recv drain——「醒來後佇列自己就是答案」)
- **卡#5 口述 ✓ 第五滑結案**(漏 2:容量立式〔登記傷疤第 2 類再現〕+ conflation 沒上桌;皇冠句=push 側在 IRQ context → wait-free 是正確性需求不是效能選擇)|**executor #5 ✓ 結案**(spawn-per-poll 冗餘無害 vs 存最新 waker,30 秒定版入 taper notes;lockfree 快考記憶體帳全清)
- 九題快掃 / static-lifetime 日讀 / 漏洞卡 app 翻 → 砍或改形:**全數蒸餾進晨讀本 `scratch/recall_checklist.md`**(§0 8:00 分鐘級動線|§1-2 默寫暖手+對答案|§2.5 String/&str/HashMap 教學|§3 九題速查+情報加權|§4 金句|§8 漏洞卡全集 wrong→right 含 code|§9 低機率認題卡)——**7/28 早唯一讀物**,user 另丟 chat 做 HTML stepper
- 加碼概念課(user 追問觸發):ordering 四層鏈(clone=Relaxed 所有權論證 → drop=Release 遺言 → 為何不全 AcqRel=拆帳 → winner fence(Acquire)=drop 是最大且不拿鎖的讀取)+「mutex 洗白 Relaxed」條件句分析 + CLOCK/second-chance
- hepta 六卡上板 ✓ ID 回記|**7/28 改 8:00 起床**(45 分動線見晨讀本 §0;7:45 起才解鎖加碼默寫區)|23:00 熄燈

## 內線情報 #4(2026-07-27 17:30 coffee chat,software head 本人)

1. **Role 實況**:近期主力=支援工廠測試程式+平台(出貨壓力),之後「回歸 SW」——時間表列 deep-dive 追問點。user 聽完**更想去**(run-sheet 實況欄有帳)。
2. **考題訊號 ①「很多 test 要決定執行順序」= toposort/依賴排程**——「graph 砍」舊裁決失效(當時 doc 零訊號,現在本人親口)。**當晚 40m 快寫補洞**(CoderPad 實機):Kahn 兩表一佇列、幽靈依賴 skip(user dry-run 自抓)、validity checker「**斷言合約不斷言實例**」(先驗長度再驗每條邊),3 案全綠;追問三層備妥(環內容=沿 stuck dep 走到撞鬼|同波歸零=可平行一批→接 thread pool|增量重跑=正向鄰接 BFS)。唯一新洞:iter 借用鏈(`&&str` 押表)+ String/&str 轉換生疏 → 晨讀本 §2.5 教學節。
3. **考題訊號 ②「給一個 DS 改 multi-thread/concurrency-safe/lock-free」= 升級階梯主場**——當晚口述靶場二發:LRU(**get 會寫** → RwLock 陷阱 → shard 整台=近似 LRU / CLOCK 一 bit 把 get 變回 read)vs config registry(真 read → RwLock 正解 → 讀壓再升=snapshot publication/rcu_snapshot,stale-but-consistent 要先問)。判準句:**看 read path 動不動結構,不看讀寫比。**

## 考後實錄 #1(2026-07-28 TPS 本尊,當天口述回憶)

**形式**:45m 英文、CoderPad、不可用 AI、只能與面試官對談。**考法 = 一大堆 spec + `todo!()` 函式骨架 + 題目敘述,指定要用哪些 todo 函式** ——與本 repo drills 格式同構(練習媒介押中);「丟英文 description 要認真讀」(情報 #2-2)獲本尊證實,且閱讀量比預期大——**文本分析本身是考點**。

**題目**:DMA 訊號處理。接收 DMA request(一個 request 含多個 blocks),有 **0–5 台 DMA engine** 可處理 blocks;本體 = 寫 event loop 派工。**雙狀態機**:request 側(哪些 blocks 完成/剩多少)+ engine 側(忙/閒、正在跑哪個 block),engine 完成要**輪詢**——同時追兩邊 state 並設計兩介面怎麼交互,是題目的真difficulty(「不像一般 leetcode,要分析文本+雙狀態」)。模組對映:fd_registry(engine slot 表)× thread_pool 派工迴圈(佇列→空閒 worker)× aggregator(per-request 完成計數)× event_loop 輪詢——**練過的四塊的合體,但以組合題形式出現**。

**實績**:只完成第一題;code + 想法說服考官(narrate-while-coding 有效,情報 #2-4 的押注兌現)。clarify/spec 對讀吃掉大量時間(此題型下屬**必要成本**,非流程洞)。**寫完即壓線 45m——boundary test / dry-run 零時間**:題目複雜度把整個時段填滿(規模明顯大於情報 #2-3 的「a/b/c/e2 量級」預期);此情境下 boundary 缺席是**題目尺寸問題,不是 pillar-5 流程洞**——10 場彩排練出的 dry-run 肌肉這場沒得展示,narrate 中帶出的不變量講解是唯一替代載體(有做到,考官被說服)。

**暴露的洞**(餵下一輪):①英文對談雙向 repair——自己的構句對方有時聽不懂需重講;對方的話有時要請 repeat(處方:rephrase 句庫 + f#1 驗證過的「複誦式確認」升為預設動作)②長 spec 快速定位「哪些 todo 是骨幹、哪些是配菜」的閱讀策略沒有練過——本輪練習全是「短題幹+自己設計」,沒有「長 spec+指定介面」形態。

**debrief 補記(同日)**:
- **todo API 面**:等待各種 event 的阻塞原語(等 DMA request 到達 / engine 完成通知)|`send_dma_engine`(派 block 給 engine)|`get_dma_engine_done() -> Option<engine_id>`(輪詢哪台完成)|`notify_dma_request_complete`(回報 request 完成)。**要寫的主體 = `run_dma_loop`(名字類似)** ——標準 reactor 形狀:wait_event → drain(新 request 拆 blocks 入 waiting_block_queue / engine 完成回收)→ 派工到 engine 滿或佇列空 → per-request 計數歸零則 notify。
- **本人設計**:request 進來逐 block 入 `waiting_block_queue`;完成判定 = 該 request 剩餘 block 數歸零 + 佇列/engine 回到全閒。per-request countdown 是正確不變量(多 request in-flight 下仍成立);「全閒」是 global quiescence 的輔助檢查。
- Q2 題面**沒看到**(未翻頁即壓線)。考官反應段:考後疲勞記不得——正常,不追。
- **主管關情報(coffee chat 時問到的)**:主管面**其實是 culture fit talk**,不是技術 deep-dive → 準備降級:經驗故事 3 條 + why-Etched + WLB 問答即可,情報 #2-5(履歷/WLB/最難問題)口徑吻合。

**Post-TPS 排程草案(2026-07-28 定,等結果期間)**:
1. **今天**:休息 + debrief 補細節 + 收帳 commit。鐵律已雙 ✓(考場本身 code+英文)。
2. **等消息期(預估 3–7 天)低量維持**:(a)**主管面準備 = culture fit 級**(debrief 補記證實非技術 deep-dive):經驗故事 3 條(一直滑帳的那格,一場 40m 做完)+ why-Etched + WLB 問答,run-sheet 情報 #1/#4 現成——仍是第一優先但總量 ~1.5h 就夠;(b)英文 repair 句庫 + 複誦式確認(今天洞①);(c)每 2 天一次 20m 暖手默寫防手感涼(spsc/pool/TCP 任一)。
3. **若下一輪仍是 coding**(overflow 池規則早有預告「四題的家在 7/29 後的 coding rounds」):加練「**長 spec + todo! 骨架**」模擬 1–2 場(雙狀態機 event loop 題,DMA 調度變體)+ 情報 #4 兩訊號(toposort/DS 改併發)保溫。
4. **下一輪時間**:若可自選,通知後抓 **5–7 天**(至少含一個週末做 2–3 場針對練習);不拖超過兩週。8 月找 HR 主線不變。

## 內線情報 #2(2026-07-20,Etched 在職網友;#1 = 7/19 deep-dive 情報,已入 7/25)

對口是 firmware 面試官 → 網友(firmware 入職)判斷**考題同款**。五條增量,前四條全是「既有裁決的確認」:

1. **epoll 確認出局**:網友根本不知道 epoll 是什麼、照樣過關 → 「epoll = deep-dive 讀物、場上 3 行 Poller stub」的裁決從推測升級為證據。
2. **題目形式 = 丟英文 description,要認真讀** → PROMPTS_EN 英文題幹練法正確;讀題本身就是 pillar 1 的一半,彩排時不准跳讀。
3. **題目規模「不算特別大」**:一個需求/scenario,實作一個結構(≈ a/b/c/e2 這個量級),不是 build-a-runtime 大題——escalation ladder 是保險,不是預設。
4. **風格 = 直接寫,不是問答**:「寫的過程中帶出知識點」。⇒ **唯一排程調整:narrate-while-coding 升為每場彩排硬動作**——邊寫邊講不變量/選型理由(protocol 10–30 分那格本來就有「邊寫邊講」,現在它就是考試本體,不是加分項);trade-off 收尾照舊。
5. **Deep dive 補證**:45 分鐘、台灣主管(+1)、聊履歷/WLB/最難問題、唯一可用中文的關;但網友全程面試只講了 ~10 分鐘中文 → **7/25 兩塊口述照舊全英文準備**,中文只當提問逃生門。

Timeline:8 月找 HR 的目標不變。

## 內線情報 #3(2026-07-21,BurgerDragon @ Discord)

1. **MPMC/MPSC 未考**(本人上場沒遇到、但有練過)→「g lock-free 版不寫、MPMC 口述應對」的裁決獲第二實證。7/21 午休已加產保險材料入庫(mpmc_ring 三層、mpsc_list、M-S、Chase-Lev + `html_p/runtime-lockfree-upgrade-map.html`,帳在 PROGRESS)——**主線場次一場不動**,材料走 7/24 日讀配套與 overflow 池。
2. **「這難度至少練 5 題到 strong hire」**→ 帳已超標:計時彩排 5 題型 10 場(a/b/c/e2 各兩遍 + d + 浮動#3),親手寫過 ≥9 題型(見彩排覆蓋帳)。

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
  7/18→p2(executor)|7/20→p3(epoll)+epoll-eventloop(封包→callback)|7/23→p8(hw_bridge,7/22 定帳自 7/22 移入)|
  7/24→p6(telemetry,已排)+lockfree 佇列家族+mpsc 交錯 stepper|**7/25 口述底稿→`docs/concurrency/thread-safe-spectrum.md` +
  `docs/rust-five-axis.md`(已從 p5/five-axis 濃縮成 repo docs,含 repo 模組對映;
  互動深挖版仍在 html_p)**。
- **7/16 產出欄**:drills/ring_buffer 綠 ✓|卡#1|drills/iter_mutate 綠 ✓(7/17 補完)|
  手寫 wrap trace 拍照|aggregator 延伸綠(**含「未來 ts 清 window」case**,
  規格照 rehearsals 題目 f 的 contract,寫在 ring_buffer 同檔)。
- **Overflow 池規則(每天適用)**:dsu / graph / trie / tree 只在三條件**全**成立時碰:
  ①當天產出欄全勾(含卡、含錄音)②明天沒欠債 ③還有力氣。
  優先序固定:spsc 空白加跑(20m)> 沒修完的彩排洞 > **fd_registry 空白加跑
  (e2 題幹再手搓一遍,25m;7/20 裁——JD sleeper 多一遍不虧)** >
  **mpmc_ring drill(30m;7/21 加產的 MPMC 保險,情報 #3 說沒考——超前才碰)** > lru challenge >
  才輪到 dsu → graph → trie/tree,每個 timebox 25m。
  (這四題的家在 7/29 後的 coding rounds;graph 是 comfort-zone 陷阱。)
- **Google 舊 block**(277/158/588、LRU/LFU、segtree):無日期 → 每週最多 1–2 題
  維持手感;本週只挑 588 當暖手,其餘全停。

## v8.1 常駐規則(2026-07-16 晚補,適用每一天)

1. **「綠」的定義**:全 suite 全開含 oracle,`#[ignore]` 移光——才算那格勾得動。
2. **修 code 洞的驗收**:先寫一個會紅的 counterexample 測試 → 轉綠;oracle 只當
   回歸網。流程洞(protocol/時間分配)下一場彩排驗。
3. 彩排中 **oracle 先抓到的 bug = pillar-5 miss**(你的 dry-run 漏了它),進漏洞清單。
4. **寫下 `unsafe impl` 的當下必唸三段式辯護**(模板:docs/rust-five-axis.md)。
5. **20 頁 artifacts 不通讀**——動手到哪開到哪。例外:7/23 signal_pipeline 頁、
   每日 clarify 決策室。
6. **彩排與卡片一律英文 I/O**:題幹讀 `rehearsals/PROMPTS_EN.md`,clarify 五問、
   定界宣言、trade-off 收尾用英文寫/講(中文版只當對照)。

## JD 對齊複核(2026-07-18,對照根目錄 `interview_prep.md`)

排程結構**經複核仍全中 JD 三技術支柱,不改逐日表**。只補「講法」四條:

1. **fd_registry = JD 白紙黑字的「event registry」**:e2 兩遍(永不砍)正確;彩排必脫口答
   「O(1) 世代 slot map 為何勝 O(n) 掃描、又擋 stale token」——③支柱的招牌題。
2. **Big-O 出聲**:JD 明言「講出 Big-O implication = massive green flag」。升為每場彩排
   trade-off 收尾的硬動作,不是心裡想。
3. **定界用 JD 場景詞**:a=網路封包流|b=HW health check|e2=event registry|
   c=wire protocol framing|signal_pipeline=telemetry 聚合。
4. **HW-adjacent system design(唯一薄弱點)**:JD 列為核心 topic,目前只有 signal_pipeline
   (JD 本尊圖)+ clarify 卡覆蓋。7/25 前把一張卡(sensor bridge / health prober)跑成
   **完整口述設計**(分執行緒/任務 + 定通訊協定),不只答五問。

**7/20 全詞掃描增補**(JD 逐詞比對,殘項全數落位;彩排場次一場不動):

- RPCs → 卡#2 已練(RPC gateway)+ 7/25 口述「hw_bridge = RPC over TCP 骨架」一句對映
- "without relying on external libraries" → **d-std(std-only TCP frame server)= 7/25 浮動#3 預設題**;d#1(7/24 tokio)照排不動——std 保險用寫的、tokio 保險照原計畫跑
- "strict hardware memory boundaries"(Low-Level IO)→ 7/25 口述 3 分鐘:volatile vs atomic / `repr(C)`/packed(epoll_sys 已有)/ endianness(hw_bridge 已有)
- HW-adjacent system design → 7/20 晚卡#5 升級完整口述設計(複核 #4 落地)
- executor challenge → 7/21 落位(7/19 裁決一直沒進逐日表)|fd_registry 加跑 → overflow 池第三位

其餘 JD 名詞本就全中:ring/packets=a、pool/health checks=b、lockless queue=spsc、
event registry=e2、telemetry 記憶體帳=f+aggregator(7/24)、runtime asynchrony=executor+7/25 口述。

不理會(JD 有提但非 coding round):Zero-Touch OS 佈署 / provisioning / stress-testing
framework = 角色風味;588 以外 Google block 全停(JD 本身反 LeetCode)。

## Clarify 配方(最弱項的處方:高頻小塊,不開大 block)

- 每天 session 開場一張卡(15m 含對答案);7/25 六張快打重來。**實況(7/21 晚)**:#1 ✓|#2 ✓ 7/19|#3 裁掉(快打時認題帶過)|#4+#6 ✓ 7/21(詳批+修課完;沉澱:playbook 五問結帳表+SOP)|#5 口述設計版 → 7/25 開場(7/22 再滑,deadline 內)
- **重打排程(7/21 定,用結帳表紀律)**:7/23 開場重打 #2(RPC gateway,5m)、7/24 開場重打 #1(telemetry hub,5m)——兩張都是結帳表誕生前做的,重測「數字/式子/裁決」三結帳;7/25 快打 = 總驗收
- **五問決策表背到能默寫**(掉不掉→full policy→容量算式→shard→SLA→怎麼知道死了)——它就是你的 clarify 演算法,每張卡、每場彩排都跑它
- 每場彩排 review 第一個打分 = pillar 1

---

進度勾選:[PROGRESS.md](PROGRESS.md)(彩排計時表、clarify 卡紀錄都在那)。
