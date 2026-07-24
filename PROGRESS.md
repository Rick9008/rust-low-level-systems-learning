# 學習進度(手動勾選,git 追蹤)

面試日:**2026-07-28**。逐日排程與砍序:[SCHEDULE.md](SCHEDULE.md)(2026-07-16 定)。
三欄語意:讀 = reference 讀懂能講;drill = 填完轉綠;
challenge = 空白手搓過(★ 才有)。日期一律寫絕對日期。
SCHEDULE 原則:**有彩排題覆蓋的 module,彩排就是它的 challenge**(ring→a、pool→b、framer→c)。

## 模組 × 三層

### TPS 優先(README 學習路徑 1–11)

| # | 模組 | 讀 | drill | challenge | 備註 |
|---|---|---|---|---|---|
| 1 | iter_mutate | ☐ | ☑ 2026-07-17 | — | 7 洞全綠(含 oracle) |
| 2 | bounded_queue | ☐ | ☑ 2026-07-16 | — | 凌晨填答已驗綠;〔v8.1〕讀排 7/17 白天、移 #[ignore] 7/17 晚開機第一件事 |
| 3 | thread_pool | ☐ | ☑ 2026-07-18 | — | 8 綠;親手抓 Drop 漏 join / 鎖圈住執行 / submit lost wakeup;challenge = 彩排 b。**骨架默寫 #1 2026-07-23**(22m+3 輪修:🔴退出條件 De Morgan ∧/∨ 三連翻→處方 loop+正面 break;⑤⑥④' 零提示寫對;帳在 `scratch/thread_pool.rs` 批改紀錄;7/24 開機重默 10m 驗秒殺) |
| 4 | ring_buffer | ☑ 2026-07-16 | ☑ 2026-07-16 | — | 7 tests 全開綠(含 oracle+白箱 guard);challenge = 彩排 a |
| 5 | spsc_ring | ☐ | ☑ 2026-07-18 | ☑ 2026-07-18 ★ | challenge 空白手搓綠(12 測試,含 DropSpy 驗 Drop);Miri 單執行緒 UB + loom 並發窮舉三重驗過;空白 20 分 ×3:7/19 ✗(35 編譯錯;Ordering 全對,傷在 use 塊/impl&lt;T&gt;/&amp;self+UnsafeCell,五類清單見 7/19 journal)、**7/22 ✓:10 分寫完(限 20)、首編 4 錯全手滑→達標**(二波 13 錯=4 根因:&self 簽名回歸、UnsafeCell 建構層、CachePadding 建構端、Drop idx+mask;Ordering/滿空算式零傷;smoke 單執行緒綠,並發由 challenge+loom 承擔)、7/26(#3 目標:20 分內含 smoke、首編 ≤2 錯)。7/20 開機默寫(讀卡→默寫→修綠,非冷測):35→12→3→1→0,三類肌肉傷全清,剩拼字/分號/turbofish 手滑(存底 commit 5387a04 scratch/skeleton.rs) |
| 6 | executor | ☐ | ☑ 2026-07-18 | ☑ 2026-07-23 ★ | drill 填綠(commit 538c624);**challenge 7/23 晚在公司完成(閥門日守住),oracle 5/5 綠**。戰報:🔴主洞=「poll 不准等」合約(Delay 曾在 poll 裡同步等+每圈 spawn,合約級提示一次才通);tier-2 三洞:Waker::from(Arc)/waker().clone()/as_mut().poll;clarify points 沒自答就動筆(pillar 1 又是)。寫對:park token 防永眠、loop 重 poll 免疫 spurious、wake_by_ref 覆寫、Delay 單欄位。遺留口述題:spawn-per-poll=「不存 waker、冗餘換正確」vs production 存 Mutex<Option<Waker>>(clarify #5,30 秒) |
| 7 | lru | ☐ | ☑ 2026-07-22 | ☐ ★ | 7/22 凌晨加時場:oracle 3/3 綠但 review 抓 2 洞(**連三場「零紅≠零洞」**):①put 淘汰路徑缺 map.insert(新 key)+promote——新 key 查不到/len 縮水/下次 put 反淘汰新 key ②unlink 頭/尾分支殘留鄰居髒指標(暫被 push_front 先 unlink 設計蓋住)。**7/22 白天修畢**(a77aa44):紅測先行×2 + 單獨 unlink 直測私有×2 + mutation 複驗(拔修行恰紅兩條) |
| 8 | fd_registry | ☑ 2026-07-19 | ☑ 2026-07-20(凌晨) | — | 6 測試全綠(stale/forged token 含);讀+概念 Q&A 全打通(epoll 三結構/generation/雙 waker);彩排 e2:7/21、7/25(7/24 晚滑帳,v9.2 移在家出聲場)|
| 9 | hw_bridge(protocol+framer) | ☐ | ☑ 2026-07-19 | ~~☐ ★~~ | 10 測試全開綠(含壓實 counterexample,red→green 驗過);standalone challenge 砍——彩排 c 即 challenge。**c#1 2026-07-23 晚:oracle 6/6 一次綠**(commit f8a5e26;dry run 自攔 2 錯;clarify 用 heartbeat 反推 len 不含 header)。✅**may_compact 雙洞 2026-07-24 修畢**(紅測先行:餵「累積消費 >4096 後繼續 feed」看 underflow → `drain(..self.ptr); self.ptr=0` 兩洞一起死;mutation 驗:主洞被咬,off-by-one 因**全 0 payload 漏網**→改 `[20]` payload 才咬住——「測試只驗想到的事」家族)。c#2 排 7/26(間隔 3 天 ✓) |
| 10 | dsu | ☐ | ☐ | ☐ ★ | **本輪砍**(doc 零訊號) |
| 11 | sharded_map | ☐ | ☐ | ☐ ★ | 降級:讀 + 口述(跨 shard 鎖序用講的) |
| 12 | signal_pipeline | ☑ 2026-07-23(深夜) | ☑ 3/3 綠 2026-07-24 | ☐ ★ | **讀 = 深夜追問串打穿**(五睡法/throttling≠鬧鈴/futex-epoll 分界=等記憶體位址 vs 等 fd/喚醒鏈終點=IRQ/acq-rel 是條件句→「最後一眼」原則/SB 兩 idiom+fence 四向牆/x86 映射 xchg-mov-mfence/shutdown 三語意;卡在 `scratch/hepta_20260724_fence_sleep_wake.md`)。drill **2026-07-24 收尾 3/3 綠、0 ignored**(Some 路徑摘牌:early return 別跳過 store(false)=每筆 send 白付 unpark;+ 拔 conservation `#[ignore]`)。litmus+扇入讀口述 → 併 7/25 晚口述錄音塊(v9.2);challenge post-TPS |
| 13 | endian_pack | — | ☐(排 7/25) | — | 7/23 凌晨新增:BE/LE 讀寫+手動 shift+i16 符號擴展+token pack/unpack(e2 mask 傷疤靶場)+混合 header,8 洞 6 測;c 題 framer 與 e2 token 的共用肌肉,drill-only。⚠ `gen` 是 edition 2024 保留字 |

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
| | c frame_parser_heartbeat | | | | | | | |
| | d tokio_frame_server | | | | | | | |
| 2026-07-21 | e2 fd_registry | ~2(僅自問 fd≤u32;無英文、無書面——pillar 1 仍最弱) | 未記錄 | 未記錄 | 部分(1 測試含手寫 trace,自認沒跑完) | 未報 Run 紀錄 | **oracle 5/5 首跑零紅**(a#1 四紅、b#1 三紅後三場首見),45 分內收工。review 抓 2 洞(oracle 漏網,皆已實跑證實):①`unregister` 的 `len -= 1` 逃出 `is_some` 守衛——偽「界內+gen 相符+空 slot」token 靜默腐化 len;len=0 時 usize underflow panic(oracle 的偽 token 恰好出界才沒炸)②`unpack` mask 寫 `(1<<31)-1` 少一 bit——fd ≥ 2³¹ alias 到低位(0x80000000 → 0x0)。**同晚修畢+repro 三案驗綠**(is_some 守衛圈住 gen+len、mask 改 32 bit、gen bump 自主採納 wrapping_add)。protocol 課:①提前收工沒把剩餘時間還給 boundary(兩洞恰在沒跑到的角落)②紅測未先行(直接修 code,規則 2 違例,補課補 assert)。**30 秒 trade-off 句已錄(7/21 晚,錄音尾段)**;兩洞卡入 Heptabase「Rust Low Level Notes」(`c84f43c4`)。e2#2(7/24)目標:clarify 出聲英文 ≥3 問、boundary 段跑滿、trade-off 收尾脫口「O(1) 世代 slot map 勝 O(n) 掃描+擋 stale」。**複核(7/22)**:mutation×2 全綠=**兩洞皆無網**(7/21 紅測未先行之債現形)→ 補紅測×2(forged 界內 gen token / 高位 fd roundtrip)先紅後綠;oracle `#[ignore]` 開燈常駐回歸 |

(e / f / g / h 預設 recognition:讀題 → 30 秒定界宣言 → 口述 arc,不計全程。)

## clarify 情境卡(每張 5 分鐘)

| 卡 | 做完日期 | 漏問了哪一類(掉不掉/速率/規模/SLA/偵測) |
|---|---|---|
| 1 telemetry hub | ☑(7/16–18 間,7/18 校正入帳;漏問欄當時未留紀錄) | — |
| 2 RPC gateway | 2026-07-19(重寫) | SLA(p99 vs p50)沒問——「SLA」標籤誤貼在 client timeout 上;另:英文把「關 client 連線的 EPOLLIN」講成「關 backend 的」 |
| 3 market data feed | **拿掉不寫**(2026-07-19 裁) | 模式 = per-key conflation,當日已深學;7/25 快打認題 30 秒帶過:"only latest matters → conflation slot, capacity = 1" |
| 4 log shipper | ☑ 2026-07-21(與 #6 連打) | 4/5:漏「斷線多久算常態」→ capacity=rate×outage 沒立式(題幹 seconds-to-minutes 沒消費);數字:1MB×1000 誤為 0.1GB;UDP 未辯護(隱形 drop) |
| 5 sensor bridge | 排 **2026-07-25 晚間出聲場開場**(v9.2:卡#5 需口述,移在家段;其餘五卡同日咖啡廳用寫的過完整流程+漏問模式表;7/20、7/22 兩滑,deadline 7/25 前仍成立) | |
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
| 2026-07-25 | `rehearsals/examples/tcp_skeleton_std.rs`(讀+默寫)+ `drills/src/io/endian_pack.rs` | 7/23 凌晨補位的兩個「上場怕要查」缺口:d 題 socket API 六行、c/e2 的 BE/LE+mask 肌肉 |
| 2026-07-27 | `rehearsals/recognition-scripts-en.md`(**先講出聲才准開**) | 九題型英文認題掃描的對分底稿——口述版 sol_*,含每題傷疤句 |
| 2026-07-25 | Heptabase 新六卡複讀(✅ 7/25 凌晨已壓縮上板 15→6,「Rust Low Level Notes」;ID 在兩份 `scratch/hepta_20260724_*` 檔頭) | 7/24 兩場深潛沉澱;口袋時間手機讀卡即可,scratch 源文件留全文底稿 |
| 2026-07-25 | `scratch/timer_queue2.rs` 檔頭批改 → 修 wheel 綠 | 回家繼續:第一版 11 error + len 沒記 + next_deadline 比較鍵;修完 `rustc --emit=metadata` 驗 |
