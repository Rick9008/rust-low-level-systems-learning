# SCHEDULE.md — Etched TPS 衝刺(7/16 → 7/28)

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
| **7/25 六(在家,彩排移白天)** | 開場:**卡#5 口述設計版**(40m 實估:sensor bridge 分 threads/tasks + 定通訊協定 + 五問,JD 複核 #4——deadline 內落地)→ 六卡快打(60–75m;卡3 認題 30 秒帶過)→ **TCP 骨架讀+默寫**(10m,`rehearsals/examples/tcp_skeleton_std.rs`:讀一遍 → 白紙默寫 std 六行 → 對答案;d#1 前置)→ **endian_pack drill**(40m,`drills/src/io/endian_pack.rs`:BE/LE 讀寫+手動 shift+token pack/unpack——e2 mask 傷疤靶場;c#2 前鎖手感)→ 🔴**a#2**(45+20,驗收斂;自原 7/23 移入)→ 🔴**d#1 tokio_frame_server**(45+20,只跑一遍)→ **口述錄音 ~75m**(ordering / Waker 鏈 / 光譜 / 選型 + executor×reactor + 五 server p99.9 + unsafe impl 三段式 + 硬體記憶體邊界 3 分鐘 + hw_bridge=RPC over TCP 一句 + TPS 尾聲反問 2 題)→ ds_sync.html §8 先答再翻(20m)。**00:30 熄燈/08:00 起** | ~7.5h |
| **7/26 日(在家,彩排移白天)** | 🔴**c#2**(45+20;c#1 7/23 → 間隔 3 天 ✓)→ 🔴**b#2**(45+20,累了這場先砍)→ **d-std**(45m,寫或口述視狀態;動筆前 TCP 骨架重默 5m)→ recognition e/f/g/h(60m)+ Q7 timer 口述 → 經驗故事 3 條(40m)→ 英文句庫唸出聲(30m)→ **spsc 空白 #3**(20m 含 smoke,首編 ≤2 錯)→ 讀自己的 challenge code(60m)→ **00:00 熄燈/07:45 起** | ~6.5h |
| **7/27 一** | **Taper 升級版:全線回憶掃描**(7/22 定:時間多 → 從「空」升級,但鐵規不變:**不碰新題、不開 oracle、不計時跑題、不寫新 code**〔骨架默寫除外〕;卡住 → 記下、翻答案讀懂就走,**不深挖**)。①**九題型掃描** a/b/c/d/e2/f/g/h(每題 12–15m:讀 PROMPTS_EN 題幹 → **全程英文出聲**:30 秒定界 → 解法 arc+選型 → trade-off 收尾 ≥2 沒選解法+Big-O → 對分:`rehearsals/recognition-scripts-en.md`(**先講才准開**,口述版 sol_*)+ sol_*/漏洞卡 → 記 ✓/⚠/✗;在公司 → 定界/trade-off 筆寫兩句,回家 23:00 前補 30m 出聲快掃)②**核心骨架默寫抽查**(15m 白紙:spsc use 塊+impl、pool 兩條件、framer 簽名、**TCP accept-loop 六行、length-prefix 解析 3 行(checked_add→get→from_be_bytes+try_into)、token pack/unpack(mask 足 32 bit)**)③**Heptabase 漏洞卡全翻**(每張 1 分鐘:當時錯什麼、修了什麼)④原 taper 收尾:背時間預算(0-3/3-5/5-10/10-35/35-40/40-45)+ 五 pillar + 開場三句 + 檢查 CoderPad/Meet/耳機/水。**產出:「認題→開場」檢查表(題型\|定界句\|選型\|trade-off 兩句\|我的傷疤),7/28 早上暖手就讀它**。⚠/✗ 超過 3 題不是加班訊號,是「靠已會的 80% 打」的提醒。**23:00 熄燈不動** | ~3.5h |
| **7/28 二** | 7:30 起床 → 8:00 暖手(小 drill 10m + pillar-5 清單 + 時間預算)→ **8:45–9:30 TPS** | — |

彩排間隔(同題 ≥3 天,近了是背答案):a 7/19→7/25|b 7/20→7/26|e2 7/21→7/24|c 7/23→7/26|d 7/25 一遍。
SPSC 空白 20 分鐘一次編過 ×3:**7/19 / 7/22 / 7/26**。

**彩排覆蓋帳(7/21 裁——「每個題型至少親手寫過一次」)**:a=a#1✓|b=b#1✓|c=framer drill✓+c#1|d=d#1+d-std|e=**e2 即其進階版**✓|f=**7/24 aggregator 延伸即 f contract**|g=**bounded_queue drill 即 g**✓|h=**7/24 快寫 30m 補上**(唯一沒寫過的)。e/g 不升全程——那是練已經最強的地方;g 的 lock-free 版不寫:block-on-full 是等待問題,condvar 繞不掉,try_push 版 = spsc_ring 本人(會講即可)。

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
| **7/25** | 卡#5 + 六卡快打:`rehearsals/clarify-cards.md` → TCP 默寫:`rehearsals/examples/tcp_skeleton_std.rs` → endian:`drills/src/io/endian_pack.rs` → a#2:PROMPTS_EN(a 題)→ d#1:PROMPTS_EN(d 題,tokio)→ 口述底稿:`docs/concurrency/thread-safe-spectrum.md` + `docs/rust-five-axis.md` + `docs/io/hw_bridge.md`(五 server 對照組)+ `docs/async/async-runtime-anatomy.md` + `docs/cost-model.md` → `docs/artifacts/ds_sync.html` §8 先答再翻 |
| **7/26** | c#2 → b#2 → d-std 前重默:`rehearsals/examples/tcp_skeleton_std.rs` → e/f/g/h 題幹:PROMPTS_EN + 認題掃描表 `rehearsals/README.md` → 英文 I/O 唸出聲:PROMPTS_EN + `docs/clarify-playbook.md` → spsc 空白 → 讀自己的 `challenges/src/` |
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
