# SCHEDULE.md — Etched TPS 衝刺(7/16 → 7/28)

容量:平日 5h / 週末 8h。排入約 45–50h,刻意留 slack——**寧可每天做完有餘,不要每天欠債**(R1 死因是疲勞)。

**每日鐵律**:收工前兩問——今天有打 code 嗎?有張嘴講英文嗎?兩個 yes 才算數。
**每場彩排 review 的打分順序**:pillar 1(clarify)永遠第一個打——那是你最弱的,每場彩排都是它的練習。
**原則:有彩排題覆蓋的 module,彩排就是它的 challenge,不重做。**(ring→a、pool→b、framer→c)

---

## 逐日

| 日期 | 內容(依序做) | 時數 |
|---|---|---|
| **7/16 四** | 〔v8.1〕卡#1(15m)→ **手寫 wrap trace(拍照)** → **`iter_mutate` drill 7 洞(盡力,硬停損)**。ring_buffer drill ✅ 已完成;aggregator 延伸**移 7/24** | ~3.5h |
| **7/17 五** | 〔v8.1〕開機第一件事:**移 bounded_queue `#[ignore]` 跑全綠(5m)** → 卡#2 → `thread_pool` drill 4 洞含 JobHandle(90m)→ `spsc_ring` drill + 逐 op 英文講 Ordering 理由(75m)→ **unsafe impl 三段式對 spsc_ring 首唸**。日讀:**bounded_queue reference(先)**→ spsc artifact | ~3.5h |
| **7/18 六** | 〔v8.1〕~~上午開場:iter_mutate 殘洞 ≤40m~~(**7/17 已補完清空**;framer drill 不需讓位,7/20 那條保險解除)→ 卡#3 → ★`spsc` challenge 空白手搓 + diff + 跑 loom(90m)→ ★`executor` drill+challenge 含 park-token 口述 + Delay(120m)→ Q7 timer 接尾(20m)→ `hw_bridge` framer **drill**(45m;standalone challenge 砍掉,c 就是它的 challenge) | ~5h |
| **7/19 日** | 卡#4 → 🔴**a#1 ring_drop_oldest**(45m+review 30m,pillar1 先打分)→ 漏洞清單 → `fd_registry` artifact 讀 + drill 3 洞(90m,弱點提前)→ **spsc 空白 #1**(20m) | ~4.5h,晚上休 |
| **7/20 一** | 卡#5 → **修 a#1 的洞**(targeted,60–90m)→ 🔴**b#1 pool_graceful**(45+30m)。通勤:event_loop / mini_runtime 略讀(餵 executor×reactor 那句) | ~3.5h |
| **7/21 二** | 卡#6(最後一張新卡)→ 🔴**e2#1 fd_registry**(45+30m)→ 修 b#1 的洞(45m) | ~3h |
| **7/22 三** | 🔴**c#1 frame_parser_heartbeat**(45+30m)→ 修 e2#1 的洞(45m)→ **spsc 空白 #2**(20m)。〔v8.1〕日讀 p8 排 **c#1 回放之後**(晚尾或 7/23 通勤) | ~3h |
| **7/23 四** | 〔v8.1〕`signal_pipeline` **110m**:**扇入先於 litmus**——讀 `start_fan_in` 6 tests + SB stepper 走四組合 + 3 trade-off 口述(不寫碼)→ drill 2 洞 → litmus 口述(**最後一份新材料**)→ 🔴**a#2**(45+20m,驗收斂)→ 修 c#1 的洞(30m) | ~3.5h |
| **7/24 五** | 🔴**e2#2**(45+20m)→ 🔴**d#1 tokio_frame_server**(45+20m,**只跑一遍**——「面試官說可用 crate」那條分支的保險;預設仍 std-only + 陳述假設)。日讀:p6 →〔v8.1〕**配套動手:aggregator 延伸**(含「未來 ts 清 window」case) | ~3h |
| **7/25 六** | 六張 clarify 卡**快打重來一輪**(40m;卡3 認題 30 秒帶過即可)→ 🔴**c#2**(45+20m)→ 🔴**浮動 #3**:給兩遍都爆的那題(45+30m)→ 口述錄音 120m **拆兩塊**(7/19 內線情報:deep-dive 面 = 主管聊履歷/WLB/最難問題,非技術深挖):①技術口述 ~75m:ordering / Waker 鏈 / 光譜 / 選型 + executor×reactor + 五 server p99.9,內含「**unsafe impl 三段式脫口(spsc_ring 實例)**」(仍餵 coding round trade-off 收尾)②主管面 ~45m:3 分鐘履歷 walk-through、最難問題故事、WLB 答案、反問清單(英文錄音) | ~5h |
| **7/26 日** | 🔴**b#2**(45+20m,累了這場先砍)→ recognition 級 e/f/g/h:讀題→30 秒定界→口述 arc(60m)→ 經驗故事 3 條寫成 bullet(40m)→ 英文句庫整份唸出聲(30m)→ **spsc 空白 #3**(20m,最後手熱檢查)→ 讀自己的 challenge code(60m) | ~4.5h,早睡 |
| **7/27 一** | **Taper。不碰新題(命令)。** 10 分鐘暖手 drill → 背時間預算(0-3/3-5/5-10/10-35/35-40/40-45)+ 五 pillar + 開場三句 → 檢查 CoderPad link / Meet / 耳機 / 水 → **早睡** | ≤1.5h |
| **7/28 二** | 8:00 暖手(小 drill 10m + pillar-5 清單 + 時間預算)→ **8:45–9:30 TPS** | — |

彩排間隔(同題 ≥3 天,近了是背答案):a 7/19→7/23|b 7/20→7/26|e2 7/21→7/24|c 7/22→7/25|d 7/24 一遍。
SPSC 空白 20 分鐘一次編過 ×3:**7/19 / 7/22 / 7/26**。

## 每日輸入(當天要開的檔案)

v8.1 規則 5 的操作版:動手到哪開到哪,這張表就是「當天該開哪幾份」。
彩排/卡片題幹一律 `rehearsals/PROMPTS_EN.md`(規則 6);`sol_*` 與 `clarify-answers.md` 寫完才開。
已 publish 到 claude.ai 的鏡像(通勤讀)在 artifacts gallery:claude.ai/code/artifacts。

| 日期 | 要開的檔案(依當天順序) |
|---|---|
| **7/17** | 掀牌:`drills/src/bounded_queue.rs`(移 `#[ignore]`)→ 卡#2:`rehearsals/PROMPTS_EN.md`(Card 2)+ `rehearsals/clarify-cards.md`(規則 5 自我批改)→ `drills/src/thread_pool.rs` → `drills/src/spsc_ring.rs` + 三段式 `docs/rust-five-axis.md`。日讀:`docs/artifacts/bounded_queue.html` → `docs/artifacts/ring_buffer.html`(§5 len 之死,spsc 前置)→ `docs/artifacts/spsc_ring.html` → `html_p/p1-rust-atomic-spsc-ordering.html`(追問鏈 + quiz) |
| **7/18** | 卡#3(同兩檔)→ `challenges/src/spsc_ring.rs` + `reference/tests/loom_spsc.rs` → `drills/src/executor.rs` + `challenges/src/executor.rs` + `docs/artifacts/executor.html` → Q7:PROMPTS_EN(h 題)→ `drills/src/hw_bridge/`。日讀:`html_p/p2-async-executor-handbook.html` |
| **7/19** | 卡#4 → a#1:PROMPTS_EN(a 題;寫完才開 `rehearsals/examples/sol_ring_drop_oldest.rs`)→ `docs/fd_registry.md` + `drills/src/fd_registry.rs` → spsc 空白(白紙) |
| **7/20** | 卡#5 → 修 a#1(照漏洞清單)→ b#1:PROMPTS_EN(b 題)。通勤:`docs/event_loop.md` + `docs/mini_runtime.md` + `docs/artifacts/event_loop.html` + `html_p/p3-epoll-readiness-event-loop.html` |
| **7/21** | 卡#6 → e2#1:PROMPTS_EN(e2 題)→ 修 b#1 |
| **7/22** | c#1:PROMPTS_EN(c 題)→ 修 e2#1 → spsc 空白。晚尾日讀:`html_p/p8-hw_bridge_teaching.html`(c#1 回放後才開) |
| **7/23** | signal_pipeline 110m:`docs/signal_pipeline.md` → `docs/artifacts/signal_pipeline.html`(SB stepper)→ `reference/src/signal_pipeline.rs`(`start_fan_in` 6 tests)→ `drills/src/signal_pipeline.rs` → a#2 → 修 c#1 |
| **7/24** | e2#2 → d#1:PROMPTS_EN(d 題,tokio)。日讀:`html_p/p6-telemetry-spsc-ring-reference.html` → aggregator 延伸動手(f 題 contract,寫在 `drills/src/ring_buffer.rs` 同檔) |
| **7/25** | 六卡快打:`rehearsals/clarify-cards.md` 全六張 → c#2 → 浮動#3 → 口述底稿:`docs/thread-safe-spectrum.md` + `docs/rust-five-axis.md` + `docs/hw_bridge.md`(五 server 對照組)+ `docs/async-runtime-anatomy.md` + `docs/cost-model.md`(數字與「再快呢」三句) |
| **7/26** | b#2 → e/f/g/h 題幹:PROMPTS_EN + 認題掃描表 `rehearsals/README.md` → 英文 I/O 唸出聲:PROMPTS_EN + `docs/clarify-playbook.md`(五問英文問法)→ spsc 空白 → 讀自己的 `challenges/src/` |
| **7/27** | 只開兩份:`rehearsals/README.md`(45 分鐘 protocol + 時間預算)+ `docs/coderpad-constraints.md`(環境確認) |
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

**晚上(回家)待辦**:卡#4+#5(30m)→ a#1 回放三輪(10m,改壞→紅→還原)→ 🔴b#1(45+30m)→ 英文口述一場錄完(a#1 trade-off + b#1 收尾合併)。b#1 今晚必跑——7/21 的「修 b#1」依賴它;真不行則修 b#1 滑 7/22(仍有 5 天間隔,安全)。

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
  7/18→p2(executor)|7/20→p3(epoll)+epoll-eventloop(封包→callback)|7/22→p8(hw_bridge)|
  7/24→p6(telemetry,已排)+lockfree 佇列家族+mpsc 交錯 stepper|**7/25 口述底稿→`docs/thread-safe-spectrum.md` +
  `docs/rust-five-axis.md`(已從 p5/five-axis 濃縮成 repo docs,含 repo 模組對映;
  互動深挖版仍在 html_p)**。
- **7/16 產出欄**:drills/ring_buffer 綠 ✓|卡#1|drills/iter_mutate 綠 ✓(7/17 補完)|
  手寫 wrap trace 拍照|aggregator 延伸綠(**含「未來 ts 清 window」case**,
  規格照 rehearsals 題目 f 的 contract,寫在 ring_buffer 同檔)。
- **Overflow 池規則(每天適用)**:dsu / graph / trie / tree 只在三條件**全**成立時碰:
  ①當天產出欄全勾(含卡、含錄音)②明天沒欠債 ③還有力氣。
  優先序固定:spsc 空白加跑(20m)> 沒修完的彩排洞 > lru challenge >
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

不理會(JD 有提但非 coding round):Zero-Touch OS 佈署 / provisioning / stress-testing
framework = 角色風味;588 以外 Google block 全停(JD 本身反 LeetCode)。

## Clarify 配方(最弱項的處方:高頻小塊,不開大 block)

- 每天 session 開場一張卡(15m 含對答案),7/17–7/22 六張走完;7/25 六張快打重來
- **五問決策表背到能默寫**(掉不掉→full policy→容量算式→shard→SLA→怎麼知道死了)——它就是你的 clarify 演算法,每張卡、每場彩排都跑它
- 每場彩排 review 第一個打分 = pillar 1

---

進度勾選:[PROGRESS.md](PROGRESS.md)(彩排計時表、clarify 卡紀錄都在那)。
