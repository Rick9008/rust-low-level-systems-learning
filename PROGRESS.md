# 學習進度(手動勾選,git 追蹤)

~~面試日:**2026-07-28**~~ → **R1 ✅ 過(7/28)**;R2 onsite(**8/6 改期版**):**8/6 Jason 場 ✅ 考完(dma_copy over segments)**;**8/10(一)10:00 deep dive(Ulysses Kao)**;**8/11(二)09:15 Jan coding + 10:00 debrief(Molly Huang)**——Jan 場自 8/6 改期,culture fit 無獨立場(散在各場 behavioral + debrief)。
R2 逐日計畫與材料:[docs/interviews/README.md](docs/interviews/README.md);R1 衝刺排程(已結案):[SCHEDULE.md](SCHEDULE.md)。
三欄語意:讀 = reference 讀懂能講;drill = 填完轉綠;
challenge = 空白手搓過(★ 才有)。日期一律寫絕對日期。
SCHEDULE 原則:**有彩排題覆蓋的 module,彩排就是它的 challenge**(ring→a、pool→b、framer→c)。

## R2 onsite(7/29 → 8/11)

R1 紀錄:[docs/interviews/2026-07-28-tps-round1-dma.md](docs/interviews/2026-07-28-tps-round1-dma.md)。

| 項 | 狀態 | 備註 |
|---|---|---|
| 讀 sol_sim_i_dma(三張表) | ☐ 排 7/29 | R1 洞的正解;Dry-Run 段紙上走一遍 |
| sim i-lite(30m 修洞重做) | ✅ 7/30 | drills 填空版 6/6 綠——R1 兩洞在 drill 層關掉;空白重寫層驗證由 m 全場扛 |
| sim j(ISR pipeline)全場 | ✅ 7/31 | 16:39–17:24 錶內完成 P1+P2,oracle 2/2 綠,自抓自修 2 bug;signal_pipeline 頁複讀 ✓ |
| sim k(per-core fan-in)**lite** | ✅ 8/1 | 前導卡(題幹當卡,clarify 六問中五類、漏 shutdown+公平性兩類→lifecycle 三問入固定清單)+ drill 16m 填綠 4/4 + review 抓「先睡才查 stop」真洞(sticky flag 合併只夠醒一次)紅測先行修至 5/5(03d0284);欠帳三題:swap(0) ✓/清 bit 順序 ✓/sizing 首答 ✗ 重考過(兩數字沒比較+假 Cross;丟/摺/批門檻問題化);sol 對照 ✓ 8/2(m 全場前暖身) |
| sim l(MMIO doorbell)**lite** | ✅ 8/2 深夜(00:50–01:45) | HW-L 卡 + clarify(暫存器所有權/滿判定/cmd 內容全問到)→ drills 填空 **4/4 綠**(backpressure/亂序 tag 路由/wrap);真洞一枚自修:滿判定誤用 comp_head(兩 ring 混帳)→ 改讀 SubmitHead;中途開錯檔(rehearsals 空白版)霧 20m → 積木圖鑑+入門頁的催生案;賽後 Q&A:barrier=store-release 硬體版 |
| sim m(watchdog)**全場** | ✅ 8/2 | 14:50–15:55(65m,**+20**;規:記洞不記違規)——clarify 命中 hang/zombie/3-strikes+「slow≠dead 無 fence 只能 bound risk」senior 句,但 idempotency 靠釣兩竿、自寫測試 0(sim j 第三課復發);review 八洞(最重:stale 鬧鐘不查帳就重派=討論→落地斷線)→ **場後不開錶自主修帳全關(+修出又收掉 2 洞),參考測試 3/3 綠**;晚間 Alarm 具名 struct 重構(Ord 四件套課)+ sol 對照三課(deadline 與 owner 同居/先問 N 再選形狀/殭屍=生存證明)+ 挑出 sol 刺(tries 只進不出)。帳:`scratch/cards_2026-08-02.md` |
| sim n(priority scheduler + DAG)**lite** | ✅ 8/3 晚(22:42–00:15,含 async 兩皮默寫段) | **改制**(用戶自點「要像真面試」):前導卡 → harness 上半英文 spec + 打字 clarify **三中三**(id 唯一但不保證單調→自發 seq/wait_event=epoll 形狀/**deps 過去式=Phase 2 主雷自己問出來**;漏問同權平手→probe 補);state 表漏 worker 側→「walk me through worker 2 done」補 owner 表;drill 5/5 綠;真洞「**銷帳早於放行**」(`waiting.remove` 綁事件不綁條件——與 sim o 鬆弛/入隊分家同族同日兩現)+ review 兩洞(**dispatch 單發**=submitted 序 oracle 對平行度全盲;`dependents` 只進不出=sim m 刺當日回鍋);附帶:**async 兩皮首默 7 錯→兩輪修 0**(park/unpark 詞組、future=狀態機 drop 洞、tokio 鏡像 std 兩規則一例外);帳:`scratch/cards_2026-08-03.md` §九 |
| sim o(boot planner,**algo 系**)**lite** | ✅ 8/3 填空 8/8 綠(不計時三輪拉鋸;含自寫紅測 ×2) | 「演算法穿硬體皮」對策首發(PDF 三支柱):Kahn 波次/DAG 最長路徑/環回報/blast radius;主課=**Dijkstra OR 閘→Kahn AND 閘**(LC 慣性走火認出即關)+ 鬆弛/入隊分家 + 同圖換序不變量測試(帳:`scratch/cards_2026-08-03.md` §四);配套認題卡 AG-R(widest path)/AG-T(聚合樹修復)排 8/4——**兩卡已升級全鏈**(reference `ds/route_planner` 4/4、`ds/aggregation_tree` 4/4 + drills 填空版;卡=前導,drill 卡後接打 20m 選配;AG-R 的骨 8/3 已提前吃掉=複驗);教材:algo-hardware-skin 頁 🛤️(三互動 stepper) |
| **8/4 開機槽 + 背靠背 ×2** | 🔶 上半 ✅(**Part B 場二欠帳今晚**) | **默寫改制**(用戶質疑「跟背課文哪裡不同」→ 題法改 micro-spec 閉卷重建、批改不對照底稿只看合約/rustc≤1輪/時間、**名字洞=形狀對+回收句=零扣分**):概念洞 **0**——昨 7 錯裡 ②park/unpark ④poll(&mut cx) ⑤pin 同顆 future 冷過、③Wake assisted(8/5 重驗)、①tokio Ext use 塊**二連**→8/5 taper;7/27 length-prefix 真洞**結案**(`.ok()?` 一輪);spsc use 塊 0 錯;block_on smoke 2/2(ready+跨線 park 路)。**場一 Fanout Gateway(tokio 皮)**:clarify 主雷自拆(slow-sub 政策)、broadcast 選型+內臟課(per-slot seq=Vyukov 同族;rem→0 即 drop/覆寫即棄);洞:retry 反射(scope creep 換皮)、**Part B 基準偷換+五行頭缺席**(Given 行=不一致偵測器)、抄紙三催。**場二 Admission Gate(std 皮)**:ThreadPool 誤讀被打回(gate=doorman、blocked callers 就是 queue)、骨架兩輪達標;洞:**&self 三度回鍋**、Condvar 拆家=lost wakeup by construction、JobHandle 死機器(scope creep 第三件)、**AND/OR 閘反**(sim o 換皮)、`entry().or_default()` 讀路徑副作用=**只進不出第四現**。批改全在檔尾:`scratch/backtoback_0804_p{1,2}.rs`、`scratch/warmup_0804_*.rs` |
| 認題卡 ×7 | ☐ 進行中 | SP ✅ 7/30(7/31 重做 19m 錶內)|HW-M ✅ 8/2 當 m 前導|HW-L = l 前導|**復活輪(8/2 晚定,用戶點名收尾):FP+TQ 排 8/3、FR+TA 排 8/4**,白天上班空檔各 15–20m——8/1 的 FP 滑帳與 7/31 改制砍掉的 8/3 槽(FR/TA/TQ)就此全數收回,七卡 8/4 前結清|FR/TA/TQ 原 8/3 槽被 n lite 取代,未再排|**8/4 白天被默寫改制+背靠背吃滿,FR/AG-R/TA/AG-T 滑今晚隨體力(砍序:TQ→FP→AG-T→TA;FR=JD sleeper 最後砍)** |
| deep dive 英文稿(3 專案) | 稿 ✅(7/29 起草、7/31 口述化) | 口述練習壓 8/7–8/9(8/10 上午上場);`docs/interviews/deep-dive-projects.md` |
| culture fit 稿 | 稿 ✅(7/31 ★ 五題全英文逐字化;★3 三細節待補真) | 無獨立場——散在各場 behavioral + debrief;唸稿壓 8/7–8/9;`docs/interviews/culture-fit-script.md` |
| **coding #2(Jason Catlin)** | ✅ **8/6 10:15 考完** | dma_copy over segments,題偏簡單;follow-up 答 retry+backoff+上限+升級;洞清單三條(idempotency 沒點名為首)→ [紀錄](docs/interviews/2026-08-06-r2-coding-jason-dma-copy.md) |
| **coding #3(Jan Lagarden,自 8/6 改期)** | ☐ **8/11(二)09:15–10:00**(CoderPad 9FF772PP;07:45 起) | 前晚(8/10)輕 taper ≤1h:taper §E+§B 殘項+8/6 洞清單;當天晨間動線照 recall_checklist §0 |
| **deep dive** | ☐ **8/10(一)10:00–10:45 Ulysses Kao**(meet kbq-myia-tvk) | 口述練習 8/7–8/9;08:30 起即可 |
| **recruiter debrief(隨 Jan 場移)** | ☐ **8/11(二)10:00–10:15 Molly Huang** | 備妥想問的 3 個問題 + 結果時程問法 |

## 模組 × 三層

### TPS 優先(README 學習路徑 1–11)

| # | 模組 | 讀 | drill | challenge | 備註 |
|---|---|---|---|---|---|
| 1 | iter_mutate | ☐ | ☑ 2026-07-17 | — | 7 洞全綠(含 oracle) |
| 2 | bounded_queue | ☐ | ☑ 2026-07-16 | — | 凌晨填答已驗綠;〔v8.1〕讀排 7/17 白天、移 #[ignore] 7/17 晚開機第一件事 |
| 3 | thread_pool | ☐ | ☑ 2026-07-18 | — | 8 綠;親手抓 Drop 漏 join / 鎖圈住執行 / submit lost wakeup;challenge = 彩排 b。**骨架默寫 #1 2026-07-23**(22m+3 輪修:🔴退出條件 De Morgan ∧/∨ 三連翻→處方 loop+正面 break;⑤⑥④' 零提示寫對;帳在 `scratch/thread_pool.rs` 批改紀錄;7/24 開機重默 10m 驗秒殺) |
| 4 | ring_buffer | ☑ 2026-07-16 | ☑ 2026-07-16 | — | 7 tests 全開綠(含 oracle+白箱 guard);challenge = 彩排 a |
| 5 | spsc_ring | ☐ | ☑ 2026-07-18 | ☑ 2026-07-18 ★ | challenge 空白手搓綠(12 測試,含 DropSpy 驗 Drop);Miri 單執行緒 UB + loom 並發窮舉三重驗過;空白 20 分 ×3:7/19 ✗(35 編譯錯;Ordering 全對,傷在 use 塊/impl&lt;T&gt;/&amp;self+UnsafeCell,五類清單見 7/19 journal)、**7/22 ✓:10 分寫完(限 20)、首編 4 錯全手滑→達標**(二波 13 錯=4 根因:&self 簽名回歸、UnsafeCell 建構層、CachePadding 建構端、Drop idx+mask;Ordering/滿空算式零傷;smoke 單執行緒綠,並發由 challenge+loom 承擔)、**7/26 #3 ✓:首編 0 錯**(25m 超線 5m、smoke 補寫+wrap 雙向蓋到;2 個編譯前自抓 `self.ring.` 路徑滑;發現=head/tail 座標系整檔對調〔Vyukov list vs ring 家族干擾第 5 現身,自洽故無 bug;處方=ring 固定 tail-write 或 read_idx/write_idx〕;Drop dry-run 四空全對)。7/20 開機默寫(讀卡→默寫→修綠,非冷測):35→12→3→1→0,三類肌肉傷全清,剩拼字/分號/turbofish 手滑(存底 commit 5387a04 scratch/skeleton.rs) |
| 6 | executor | ☐ | ☑ 2026-07-18 | ☑ 2026-07-23 ★ | drill 填綠(commit 538c624);**challenge 7/23 晚在公司完成(閥門日守住),oracle 5/5 綠**。戰報:🔴主洞=「poll 不准等」合約(Delay 曾在 poll 裡同步等+每圈 spawn,合約級提示一次才通);tier-2 三洞:Waker::from(Arc)/waker().clone()/as_mut().poll;clarify points 沒自答就動筆(pillar 1 又是)。寫對:park token 防永眠、loop 重 poll 免疫 spurious、wake_by_ref 覆寫、Delay 單欄位。遺留口述題:spawn-per-poll=「不存 waker、冗餘換正確」vs production 存 Mutex<Option<Waker>>(clarify #5,30 秒) |
| 7 | lru | ☐ | ☑ 2026-07-22 | ☐ ★ | 7/22 凌晨加時場:oracle 3/3 綠但 review 抓 2 洞(**連三場「零紅≠零洞」**):①put 淘汰路徑缺 map.insert(新 key)+promote——新 key 查不到/len 縮水/下次 put 反淘汰新 key ②unlink 頭/尾分支殘留鄰居髒指標(暫被 push_front 先 unlink 設計蓋住)。**7/22 白天修畢**(a77aa44):紅測先行×2 + 單獨 unlink 直測私有×2 + mutation 複驗(拔修行恰紅兩條) |
| 8 | fd_registry | ☑ 2026-07-19 | ☑ 2026-07-20(凌晨) | — | 6 測試全綠(stale/forged token 含);讀+概念 Q&A 全打通(epoll 三結構/generation/雙 waker);彩排 e2:7/21、7/25(7/24 晚滑帳,v9.2 移在家出聲場)|
| 9 | hw_bridge(protocol+framer) | ☐ | ☑ 2026-07-19 | ~~☐ ★~~ | 10 測試全開綠(含壓實 counterexample,red→green 驗過);standalone challenge 砍——彩排 c 即 challenge。**c#1 2026-07-23 晚:oracle 6/6 一次綠**(commit f8a5e26;dry run 自攔 2 錯;clarify 用 heartbeat 反推 len 不含 header)。✅**may_compact 雙洞 2026-07-24 修畢**(紅測先行:餵「累積消費 >4096 後繼續 feed」看 underflow → `drain(..self.ptr); self.ptr=0` 兩洞一起死;mutation 驗:主洞被咬,off-by-one 因**全 0 payload 漏網**→改 `[20]` payload 才咬住——「測試只驗想到的事」家族)。**c#2 ✓ 7/26:oracle 6/6 一次綠、30m、零洞——c 題型結案**(詳計時表) |
| 10 | dsu | ☐ | ☐ | ☐ ★ | **本輪砍**(doc 零訊號) |
| 11 | sharded_map | ☐ | ☐ | ☐ ★ | 降級:讀 + 口述(跨 shard 鎖序用講的) |
| 12 | signal_pipeline | ☑ 2026-07-23(深夜) | ☑ 3/3 綠 2026-07-24 | ☐ ★ | **讀 = 深夜追問串打穿**(五睡法/throttling≠鬧鈴/futex-epoll 分界=等記憶體位址 vs 等 fd/喚醒鏈終點=IRQ/acq-rel 是條件句→「最後一眼」原則/SB 兩 idiom+fence 四向牆/x86 映射 xchg-mov-mfence/shutdown 三語意;卡在 `scratch/hepta_20260724_fence_sleep_wake.md`)。drill **2026-07-24 收尾 3/3 綠、0 ignored**(Some 路徑摘牌:early return 別跳過 store(false)=每筆 send 白付 unpark;+ 拔 conservation `#[ignore]`)。litmus+扇入讀口述 → 併 7/25 晚口述錄音塊(v9.2);challenge post-TPS |
| 13 | endian_pack | — | ☑ **2026-07-26**(6/6 綠、0 ignored、clippy 淨;實 ~35–40m 含岔題,照 v9 規則記實際) | — | 7/23 凌晨新增:BE/LE 讀寫+手動 shift+i16 符號擴展+token pack/unpack(e2 mask 傷疤靶場)+混合 header,8 洞 6 測;c 題 framer 與 e2 token 的共用肌肉,drill-only。⚠ `gen` 是 edition 2024 保留字 |
| 14 | spin_lock | ☐ | ☑ **2026-08-01(凌晨)** 3/3 綠 | ☐ ★(8/6 後固化) | 7/31 晚新增教材(TTAS+RAII guard)。drill 四洞(lock/try_lock/DerefMut/Drop)+紙上五問入檔;review 兩洞當場修(commit bf98da9):①try_lock 誤用 `compare_exchange_weak`——單發假失敗=作偽證(None≠被佔用),改 strong ②成功側 AcqRel 過度訂購(上鎖時無物可發佈),改 Acquire。對話沉澱兩教材頁 mesi-rmw-atomics(🚌 MESI/RMW 家族/TAS vs TTAS/驚群)+ guard-design(🛡️ lifetime 綁定/Send 裁決/poisoning),claude.ai 已發、index 深讀區 +2 卡 |
| 15 | conflation_slot | ☐ | ☐ 排 8/2 尾或 8/3(加時槽) | ☐ ★(8/6 後固化) | 7/31 晚新增三層教材(值層/通知層分離、invariant queued⟺∈ready);drill 挖 publish/recv/close;使用者自製 stepper 頁(🗜️)7/31 已發本帳號鏡像 |

### 次優先

SCHEDULE 裁決:inplace_leetcode 選配暖手;graph / trie / tree **本輪砍**。

| 模組 | 讀 | drill | challenge |
|---|---|---|---|
| inplace_leetcode | ☐ | — | — |
| graph | ☐ | ☐ | — |
| trie | ☐ | ☑ 2026-07-21(深夜) | — |
| tree | ☐ | ☐ | — |
| mpmc_ring(7/21 加產:MPMC 保險題,spsc 後續) | ☐ | ☐ | ☐ ★ |
| mpsc_list(7/21 加產:tokio remote-wake queue;縫顯式化) | ☐ | ☐ | — |

### deep-dive(讀懂能講即可,不手搓)

SCHEDULE 裁決:全部 post-TPS。例外:event_loop / mini_runtime 略讀(7/20 通勤)、
五 server 對照組 p99.9(7/25,餵 trade-off 口述)、async-runtime-anatomy(同 7/25)。

| 模組 | 讀 |
|---|---|
| arena_lockfree | ☐ |
| mpmc_list(Michael–Scott;7/21 加產:help vs 縫、reclamation 攤開講) | ☐ |
| ws_deque(Chase–Lev;7/21 加產:SB fence 第二實戰位、loom 抓洞實錄) | ☐ |
| mpsc_ring(7/21 加產:退化表實體——head 非原子;drill 由 mpmc_ring 第 5 問覆蓋) | ☐ |
| rcu_snapshot(7/21 加產:RCU/ArcSwap 模式 std 實體——免費寬限期、無 AtomicArc 的原因) | ☐ |
| epoll_sys | ☐ |
| event_loop | ☑ 2026-07-20(原「通勤略讀」升級為桌前深讀 Q&A:WAKE_TOKEN 旁路/woken 字條/Arc&lt;EventFd&gt; 兩張臉;沉澱 `qa_eventfd_doorbell.html`) |
| tcp_echo | ☐ |
| file_io_offload | ☐ |
| hw_bridge 五 server 對照組(threaded / inline壞 / evented / sharded / spsc) | ☐ |
| mini_runtime(V0 scan → V1 epoll) | ☑ 2026-07-20(block_on 三段迴圈/arm_io/FdRegistry&lt;Waker&gt; interest table 走讀;Events 死在 reactor 邊界、Waker 唯一介面) |
| async_sync(AsyncMutex / Notify;有 drill 四洞,選練) | ☐ |
| docs/async/async-runtime-anatomy.md | ☐ |
| docs/concurrency/thread-safe-spectrum.md(7/25 口述底稿) | ☐ |
| docs/rust-five-axis.md(7/25 口述底稿;unsafe impl 辯護模板) | ☐ |

## rehearsal 計時紀錄

每跑一次加一列。時間欄填實際分鐘;protocol 目標:5 / 5 / 20 / 10 / 5。

| 日期 | 題 | clarify | skeleton | core | boundary | trade-offs | 一次編過? | 哪段爆 / 對照漏了什麼 |
|---|---|---|---|---|---|---|---|---|
| 2026-07-19 | a ring_drop_oldest | ~5 | ~5 | ~25 | **0(自行跳過)** | ~5 | 未記錄 | oracle 4/5 紅,全數 pillar-5 miss:①pop 判空用 head==tail(滿=空二義,連鎖 len>cap+FIFO 毀)②drop_cnt 整條沒 ++ ③Part 2 擅改 contract 成阻塞 pop(clarify miss)。亮點:SPSC×drop-oldest 衝突當場談判降級。修洞 7/20 紅測先行 |
| 2026-07-21(凌晨,7/20 場)| b pool_graceful_shutdown | ~5(只問 1 題:graceful 語意——好問但獨苗;漏 queue 上限/job panic)| —(題檔附簽名)| **~33(溢時 +13)** | **1(只點名 1 條 case,沒 trace)** | 0(末段改抓 join 漏)| 整場沒按 Run | oracle 2 綠 3 紅,紅全 pillar-5:①worker 見 flag 即退不清 queue(0/16)②空佇列 shutdown hang(wait predicate 漏查 shutdown)③repeated_shutdown 連鎖。**亮點:boundary 唯一點名的 case 正是 hang 那條**(死因=core 溢時吃掉 trace 時間);**時限內自抓 shutdown 忘 join**(JD:finding your own bug = stronger signal)。三個 thread_pool drill 老 bug 全回歸。當場自行診斷②的兩條件(退出=shutdown∧空;睡=空∧¬shutdown)。**7/21 凌晨修至全綠 5/5(已驗,oracle 帶 `--include-ignored`)**。補課帳:**英文 trade-off 錄音 ✓ 7/21 晚(a#1+b#1 合場,兩天欠帳還清)**;三紅卡入 Heptabase「Rust Low Level Notes」(`e2eb0dfb`,含 lost-wakeup 預測題);自寫紅測 ×3 + dry-run + 回放 → 7/22 與 e2#1 複核合段。**補課完結(7/22,d3b4a44)**:自寫紅測×3、mutation 逐洞驗咬人(①drain 紅 ②hang exit=124 ③冪等綠);**新抓三洞全修**:④空佇列喚醒 pop unwrap panic+毒鎖、全被 `let _=join()` 吞掉(--nocapture 揭發;修 match+continue、join().expect 常駐)⑤lost-wakeup:store+notify 不拿 jobs 鎖(dry-run 先手走:結果對、修法答錯→修正為 mutex 序;修 notify 進鎖,**loom_lost_wakeup 三變體裁決**:不拿鎖必死/store 進鎖/notify 進鎖皆綠)⑥鎖圈 job 全池串行(40×10ms/4w 實測 0.40s→放鎖跑 **0.10s**)|
| 2026-07-23(晚)| c frame_parser_heartbeat(**#1**)| ~2(heartbeat 反推 len 不含 header)| — | 未分段記 | 部分 | 有 | **oracle 6/6 一次綠** | dry run 自攔 2 錯;遺留 may_compact 雙洞(累積消費 >4096 後 underflow)→ 7/24 紅測修畢+mutation 驗(全 0 payload 漏網→改 [20] 才咬住) |
| 2026-07-26(15:08–15:38)| c frame_parser_heartbeat(**#2,impl 挖空真空白**)| ~5(**3 問英文全載重**:max len+Err 合約/len 語意 heartbeat 反推/compact 記憶體策略——本輪最佳)| — | **22** | 多輪(multi-frame 測試**點名三次才寫**)| ~5(三拍磨兩輪才乾淨)| **oracle 6/6 一次綠,總 30m** | **零洞、c 題型結案**;may_compact 傷疤肌肉重寫一次對;parse 改回傳 Option<Frame> 比 #1 乾淨;P1 4/5|P4 3/5(cache locality 錯置→正解=攤銷拷貝)|P5 3/5(結果零 miss、流程「boundary 不自燃」連三場);評級 H 摸 SH 邊 |
| 2026-07-26(16:03–16:49)| g bounded_channel(**首跑全場**)| ~3(真問題僅 1:close 時機;blocking/Drop 語意用宣告不是問;**drain 語意答了還掉**→處方:答案複誦回去)| — | ~28(檔記 4:11–4:31)| 自寫 2 測但 boundary_test **沒 join = 空測試** | ~4(「Ordering 等 buf flush」錯誤心智模型)| 未跑先喊(46m 超 2m)| **oracle 首跑 4 紅同根**:recv 兩個 `cnt==0 → None` early return 蓋過「佇列還有貨」(合約講過兩次仍掉,d#1 idle_timeout 家族)→修畢 6/6×3。review 再抓 2 洞:①**雙 Drop store-不-notify+缺 mutex 括號**(b#1 lost-wakeup 本尊回鍋,loom 親證的窗;修=store→拿鎖放鎖→notify_all,Sender 側 fetch_sub==1 idiom)②「**沒 join 的斷言不是斷言**」(孤兒執行緒 panic 被吞,它想驗的場景真死鎖);P4 2.5/P5 2(自燃 ✓ 但火濕) |
| 2026-07-26(18:20–19:05 含修)| f telemetry_aggregator(**#1,30+10**)| ~4(**複誦式開場**=把理解講回來讓面試官修——本輪最佳一手;「now 從哪來」是送的)| — | **21** | **首次全自發 ✓**(dry run+boundary 自己轉場)| ~4(停放題 O(1) 未主動回收)| 自寫 2 綠 | oracle 4/5:far_future_jump 鬼資料(**「案名點過仍漏」**——三格 boundary 唸過案名仍漏第一格);修=**lazy validation**(桶掛 epoch 牌,讀寫各自驗;勝 eager 清掃:record 嚴格 O(1))→5/5×3;**回放驗證**:131 行改壞→紅(鬼=epoch 2 的 sum:50 同餘撞 slot 0)→還原綠;同餘鬼資料家族第 3 現身 |
| 2026-07-25(晚 23:15–00:00)| d tokio_frame_server(**首寫**)| ~5(4 問英文:JoinHandle 處理/parser 重用/accept 錯誤傳播/flow count scope)| —(題檔附簽名)| **15** | **0 自寫(review 點名後補 2 條)** | 未跑(00:00 壓線)| 修後 oracle **6/6 一次綠** | 三大洞全 review 抓(非 dry-run 自攔):①**idle_timeout 整條蒸發**——clarify 沒問到的需求恰是掉的需求 → 處方:動筆前 clarify 對讀需求清單 30 秒 ②echo 只回 payload **掉 wire format** ③自測零條(boundary 又跳,e2#1 死因重演);自測寫死 port `AddrInUse`(`:0` 肌肉當天教當天沒用)。亮點:`break 'parsing` 跨巢狀一次寫對;「喊綠沒驗」×1 |
| 2026-07-21 | e2 fd_registry | ~2(僅自問 fd≤u32;無英文、無書面——pillar 1 仍最弱) | 未記錄 | 未記錄 | 部分(1 測試含手寫 trace,自認沒跑完) | 未報 Run 紀錄 | **oracle 5/5 首跑零紅**(a#1 四紅、b#1 三紅後三場首見),45 分內收工。review 抓 2 洞(oracle 漏網,皆已實跑證實):①`unregister` 的 `len -= 1` 逃出 `is_some` 守衛——偽「界內+gen 相符+空 slot」token 靜默腐化 len;len=0 時 usize underflow panic(oracle 的偽 token 恰好出界才沒炸)②`unpack` mask 寫 `(1<<31)-1` 少一 bit——fd ≥ 2³¹ alias 到低位(0x80000000 → 0x0)。**同晚修畢+repro 三案驗綠**(is_some 守衛圈住 gen+len、mask 改 32 bit、gen bump 自主採納 wrapping_add)。protocol 課:①提前收工沒把剩餘時間還給 boundary(兩洞恰在沒跑到的角落)②紅測未先行(直接修 code,規則 2 違例,補課補 assert)。**30 秒 trade-off 句已錄(7/21 晚,錄音尾段)**;兩洞卡入 Heptabase「Rust Low Level Notes」(`c84f43c4`)。e2#2(7/24)目標:clarify 出聲英文 ≥3 問、boundary 段跑滿、trade-off 收尾脫口「O(1) 世代 slot map 勝 O(n) 掃描+擋 stale」。**複核(7/22)**:mutation×2 全綠=**兩洞皆無網**(7/21 紅測未先行之債現形)→ 補紅測×2(forged 界內 gen token / 高位 fd roundtrip)先紅後綠;oracle `#[ignore]` 開燈常駐回歸 |

| 2026-07-25(晚 22:40–23:13)| e2 fd_registry(**#2**)| **3 問英文出聲 ✓**(sparse/dense 帶結構後果/token↔epoll 用途/**fd 回收重用 = 錢問題**);定界宣言太薄(一句就開寫)| 未分段記 | ~30 | 繼承 e2#1 三測(未新寫)| 未跑 | 修後自測 3 + oracle 5/5 綠 | **部分重寫**(Token+測試繼承 e2#1,diff 35+/55−,收斂訊號打折)。初版:len 全程沒記帳、unregister **沒驗 generation**(`_generation` 自首)——過期 token 拔現任;修版 `len -= 1` 又逃出佔用確認 → **繼承的 forged 紅測抓到 e2#1 洞① 回鍋**(oracle 5/5 綠抓不到,自寫測試 4 天後自動放哨 = 規則 2 複利首例)。傷疤「狀態變更押在 take()==Some 之後」記**未癒合**;「喊綠沒驗」×1 |

(e / h 預設 recognition。**f/g 已升計時並完賽**(7/26,表列);e 快寫 ✓ 7/26(34m,非計時:真洞=handler_count 吃 id 回全域〔參數沒用=漏讀警報〕;retain_mut 進肌肉;type erasure 預期洞未現=7/24 pool 課有效);h 7/24 親手寫過。)

**7/27 taper 收帳**:骨架默寫抽查 1✓/5⚠/1✗(✗=length-prefix `usize::from_be_bytes` 真洞;Sender Drop `==1` 方向默反;批改在 `scratch/recall_20260727.rs` 檔頭)|h 口述 ⚠(掉 drift-free+wait_timeout 兩招牌句)|三舊解 dry-run ✓(a wrap/d idle×半 frame/g recv drain)|卡#5+executor #5 雙結案|**情報 #4**(coffee chat)觸發 **toposort 快寫綠**(CoderPad 實機,3 案+validity checker;新洞=iter 借用鏈+String/&str 生疏)+ DS 升級階梯靶場二發(LRU vs registry 對照)|全日蒸餾 → **晨讀本 `scratch/recall_checklist.md`**(7/28 早唯一讀物)。

## clarify 情境卡(每張 5 分鐘)

| 卡 | 做完日期 | 漏問了哪一類(掉不掉/速率/規模/SLA/偵測) |
|---|---|---|
| 1 telemetry hub | ☑(7/16–18 間,7/18 校正入帳;漏問欄當時未留紀錄) | — |
| 2 RPC gateway | 2026-07-19(重寫) | SLA(p99 vs p50)沒問——「SLA」標籤誤貼在 client timeout 上;另:英文把「關 client 連線的 EPOLLIN」講成「關 backend 的」 |
| 3 market data feed | **拿掉不寫**(2026-07-19 裁) | 模式 = per-key conflation,當日已深學;7/25 快打認題 30 秒帶過:"only latest matters → conflation slot, capacity = 1" |
| 4 log shipper | ☑ 2026-07-21(與 #6 連打) | 4/5:漏「斷線多久算常態」→ capacity=rate×outage 沒立式(題幹 seconds-to-minutes 沒消費);數字:1MB×1000 誤為 0.1GB;UDP 未辯護(隱形 drop) |
| 5 sensor bridge | ☑ **2026-07-27(口述,第五滑結案)** | 2/5:容量立式沒立(burst rate×duration——登記傷疤第 2 類再現)+ conflation 沒上桌("only the latest matters?" 值一個量級)。亮點:四問結構好、三定理自己回收(drop-oldest→SPMC/aggregate→window/heartbeat=liveness 要主動訊號);皇冠句=push 側在 IRQ context → wait-free 是正確性需求 |
| 6 health prober | ☑ 2026-07-21(與 #4 連打) | 3/5:漏併發上限(題幹 must not hammer 沒消費)+ 判死無去抖(連續 N 次);window=interval+N×timeout 沒立式。亮點:沒掏 lock-free ✓、push/pull 主動決策 ✓ |

## 下次複習

| 日期 | 要複習什麼 | 為什麼 |
|---|---|---|
| 2026-07-25 | `qa_eventfd_doorbell.html` 五化身表 + 面試句庫 | 口述底稿彈藥:「訊號帶狀態」五化身、executor×reactor 接縫、Poller 命名品味 |
| 2026-07-25 | `scratch/trade_off_map_ab.md`(a 四軸/b 五軸 + lock-free 應對階梯 + 英文範例句) | 7/21 凌晨對話沉澱;技術口述錄音的 a/b 段直接照地圖講,含「被逼 lock-free」兩套應對 |
| 2026-07-23 | `scratch/trade_off_map_ab.md` §fan-in(掃描聚合 + 留的那題) | signal_pipeline 讀到 SB stepper「帶著貨睡死」時回來對答案:consumer 睡誰叫醒 |
| 2026-07-24 | `qa_lockfree_followups.html`(九題)+ 頁尾 stepper 對照表 | 7/24 lockfree 家族段的文字版前置;讀完照表逐台走 stepper |
| 2026-07-24 | `scratch/thread_pool.rs` 批改紀錄 → 白紙全骨架重默 10m | 默寫 rep#1 主傷疤:退出條件否定式 ∧/∨ 三連翻;秒殺線=首編 ≤3 錯、兩條件一次對 |
| 2026-07-25 | `html_p/runtime-lockfree-upgrade-map.html`(§1 三問 + §8 追問鏈)+ ~~SPSC→MPMC 30 秒稿~~(✅ 7/25 凌晨錄畢;同場 Q1/Q4 複測:Q2 過、**Q1 why 層半洞**〔unconditional vs conditional claim〕→ 晚間口述複測) | 錄音「被逼 lock-free」段現在有 repo 實體(mpmc_ring/mpsc_list/ws_deque),照地圖講;ws_deque 的 loom 抓洞實錄是「窮舉>直覺」的第一手證據 |
| 2026-07-25 | ~~`rehearsals/examples/tcp_skeleton_std.rs`(讀+默寫)~~(✅ 7/25 咖啡廳 6 輪 7 洞→0,傷疤在 `scratch/tcp_skelton2.rs` 檔頭;7/26 d-std 前 5m 重默)+ `drills/src/io/endian_pack.rs`(→ **7/26 08:20**) | 7/23 凌晨補位的兩個「上場怕要查」缺口:d 題 socket API 六行、c/e2 的 BE/LE+mask 肌肉 |
| 2026-07-27 | `rehearsals/recognition-scripts-en.md`(**先講出聲才准開**) | 九題型英文認題掃描的對分底稿——口述版 sol_*,含每題傷疤句 |
| **2026-07-28 早** | **`scratch/recall_checklist.md` 晨讀本(8:00 起床唯一讀物)** | §0 分鐘動線 → §1 默寫暖手(寫完才翻 §2)→ §3 九題+情報 → §4 金句出聲;§8 漏洞卡全集 wrong→right、§9 低機率認題卡。7/27 全日+coffee chat 情報蒸餾 |
| 2026-07-25 | Heptabase 新六卡複讀(✅ 7/25 凌晨已壓縮上板 15→6,「Rust Low Level Notes」;ID 在兩份 `scratch/hepta_20260724_*` 檔頭) | 7/24 兩場深潛沉澱;口袋時間手機讀卡即可,scratch 源文件留全文底稿 |
| ~~2026-07-25~~ **post-TPS** | `scratch/timer_queue2.rs` 檔頭批改 → 修 wheel 綠 | 7/25 咖啡廳擠照閥門陣亡;wheel 概念已由口述覆蓋(O(1) 攤還記帳法/退化條件,`hepta_20260725_cafe_qa.md` 卡4),修綠不再是 TPS 前置 |
