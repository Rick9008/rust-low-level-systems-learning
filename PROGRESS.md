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
| 3 | thread_pool | ☐ | ☑ 2026-07-18 | — | 8 綠;親手抓 Drop 漏 join / 鎖圈住執行 / submit lost wakeup;challenge = 彩排 b |
| 4 | ring_buffer | ☑ 2026-07-16 | ☑ 2026-07-16 | — | 7 tests 全開綠(含 oracle+白箱 guard);challenge = 彩排 a |
| 5 | spsc_ring | ☐ | ☑ 2026-07-18 | ☑ 2026-07-18 ★ | challenge 空白手搓綠(12 測試,含 DropSpy 驗 Drop);Miri 單執行緒 UB + loom 並發窮舉三重驗過;空白 20 分 ×3:7/19 ✗(35 編譯錯;Ordering 全對,傷在 use 塊/impl&lt;T&gt;/&amp;self+UnsafeCell,五類清單見 7/19 journal)、7/22(目標 ≤5 錯)、7/26。7/20 開機默寫(讀卡→默寫→修綠,非冷測):35→12→3→1→0,三類肌肉傷全清,剩拼字/分號/turbofish 手滑(存底 commit 5387a04 scratch/skeleton.rs) |
| 6 | executor | ☐ | ☑ 2026-07-18 | ☐ ★ | drill 填綠(commit 538c624;檔內 `todo!()` 是註解掉的規格提示);讀 + challenge 空白排 7/21 |
| 7 | lru | ☐ | ☐ | ☐ ★ | 降級:超前才寫 |
| 8 | fd_registry | ☑ 2026-07-19 | ☑ 2026-07-20(凌晨) | — | 6 測試全綠(stale/forged token 含);讀+概念 Q&A 全打通(epoll 三結構/generation/雙 waker);彩排 e2:7/21、7/24 |
| 9 | hw_bridge(protocol+framer) | ☐ | ☑ 2026-07-19 | ~~☐ ★~~ | 10 測試全開綠(含壓實 counterexample,red→green 驗過);standalone challenge 砍——彩排 c 即 challenge |
| 10 | dsu | ☐ | ☐ | ☐ ★ | **本輪砍**(doc 零訊號) |
| 11 | sharded_map | ☐ | ☐ | ☐ ★ | 降級:讀 + 口述(跨 shard 鎖序用講的) |
| 12 | signal_pipeline | ☐ | ☐(2 洞) | ☐ ★ | 7/23 drill + litmus 口述;扇入(fan_in)讀+口述排 7/25;challenge post-TPS |

### 次優先

SCHEDULE 裁決:inplace_leetcode 選配暖手;graph / trie / tree **本輪砍**。

| 模組 | 讀 | drill | challenge |
|---|---|---|---|
| inplace_leetcode | ☐ | — | — |
| graph | ☐ | ☐ | — |
| trie | ☐ | ☐ | — |
| tree | ☐ | ☐ | — |

### deep-dive(讀懂能講即可,不手搓)

SCHEDULE 裁決:全部 post-TPS。例外:event_loop / mini_runtime 略讀(7/20 通勤)、
五 server 對照組 p99.9(7/25,餵 trade-off 口述)、async-runtime-anatomy(同 7/25)。

| 模組 | 讀 |
|---|---|
| arena_lockfree | ☐ |
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
| 2026-07-21(凌晨,7/20 場)| b pool_graceful_shutdown | ~5(只問 1 題:graceful 語意——好問但獨苗;漏 queue 上限/job panic)| —(題檔附簽名)| **~33(溢時 +13)** | **1(只點名 1 條 case,沒 trace)** | 0(末段改抓 join 漏)| 整場沒按 Run | oracle 2 綠 3 紅,紅全 pillar-5:①worker 見 flag 即退不清 queue(0/16)②空佇列 shutdown hang(wait predicate 漏查 shutdown)③repeated_shutdown 連鎖。**亮點:boundary 唯一點名的 case 正是 hang 那條**(死因=core 溢時吃掉 trace 時間);**時限內自抓 shutdown 忘 join**(JD:finding your own bug = stronger signal)。三個 thread_pool drill 老 bug 全回歸。當場自行診斷②的兩條件(退出=shutdown∧空;睡=空∧¬shutdown)。**7/21 凌晨修至全綠 5/5(已驗,oracle 帶 `--include-ignored`)**。補課仍欠(今晚修 b#1 段):自寫紅測 ×3、shutdown 側 store+notify 的 lost-wakeup dry-run、英文 trade-off 收尾補錄(a#1+b#1 合場)|
| | c frame_parser_heartbeat | | | | | | | |
| | d tokio_frame_server | | | | | | | |
| | e2 fd_registry | | | | | | | |

(e / f / g / h 預設 recognition:讀題 → 30 秒定界宣言 → 口述 arc,不計全程。)

## clarify 情境卡(每張 5 分鐘)

| 卡 | 做完日期 | 漏問了哪一類(掉不掉/速率/規模/SLA/偵測) |
|---|---|---|
| 1 telemetry hub | ☑(7/16–18 間,7/18 校正入帳;漏問欄當時未留紀錄) | — |
| 2 RPC gateway | 2026-07-19(重寫) | SLA(p99 vs p50)沒問——「SLA」標籤誤貼在 client timeout 上;另:英文把「關 client 連線的 EPOLLIN」講成「關 backend 的」 |
| 3 market data feed | **拿掉不寫**(2026-07-19 裁) | 模式 = per-key conflation,當日已深學;7/25 快打認題 30 秒帶過:"only latest matters → conflation slot, capacity = 1" |
| 4 log shipper | 排 2026-07-21 開場(7/20 晚未動,與 #6 連打 15+15) | |
| 5 sensor bridge | 排 2026-07-22(**升級完整口述設計版** 25–30m:threads/tasks + 通訊協定,JD 複核 #4;7/20 晚未動) | |
| 6 health prober | 排 2026-07-21 開場(與 #4 連打) | |

## 下次複習

| 日期 | 要複習什麼 | 為什麼 |
|---|---|---|
| 2026-07-25 | `qa_eventfd_doorbell.html` 五化身表 + 面試句庫 | 口述底稿彈藥:「訊號帶狀態」五化身、executor×reactor 接縫、Poller 命名品味 |
| 2026-07-25 | `scratch/trade_off_map_ab.md`(a 四軸/b 五軸 + lock-free 應對階梯 + 英文範例句) | 7/21 凌晨對話沉澱;技術口述錄音的 a/b 段直接照地圖講,含「被逼 lock-free」兩套應對 |
| 2026-07-23 | `scratch/trade_off_map_ab.md` §fan-in(掃描聚合 + 留的那題) | signal_pipeline 讀到 SB stepper「帶著貨睡死」時回來對答案:consumer 睡誰叫醒 |
