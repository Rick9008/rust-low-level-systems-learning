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
| 5 | spsc_ring | ☐ | ☑ 2026-07-18 | ☑ 2026-07-18 ★ | challenge 空白手搓綠(12 測試,含 DropSpy 驗 Drop);Miri 單執行緒 UB + loom 並發窮舉三重驗過;空白 20 分 ×3:7/19、7/22、7/26 |
| 6 | executor | ☐ | ☐ | ☐ ★ | 7/18 drill+challenge |
| 7 | lru | ☐ | ☐ | ☐ ★ | 降級:超前才寫 |
| 8 | fd_registry | ☐ | ☐ | — | JD sleeper;7/19 drill;彩排 e2:7/21、7/24 |
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
| event_loop | ☐ |
| tcp_echo | ☐ |
| file_io_offload | ☐ |
| hw_bridge 五 server 對照組(threaded / inline壞 / evented / sharded / spsc) | ☐ |
| mini_runtime(V0 scan → V1 epoll) | ☐ |
| async_sync(AsyncMutex / Notify;有 drill 四洞,選練) | ☐ |
| docs/async-runtime-anatomy.md | ☐ |
| docs/thread-safe-spectrum.md(7/25 口述底稿) | ☐ |
| docs/rust-five-axis.md(7/25 口述底稿;unsafe impl 辯護模板) | ☐ |

## rehearsal 計時紀錄

每跑一次加一列。時間欄填實際分鐘;protocol 目標:5 / 5 / 20 / 10 / 5。

| 日期 | 題 | clarify | skeleton | core | boundary | trade-offs | 一次編過? | 哪段爆 / 對照漏了什麼 |
|---|---|---|---|---|---|---|---|---|
| | a ring_drop_oldest | | | | | | | |
| | b pool_graceful_shutdown | | | | | | | |
| | c frame_parser_heartbeat | | | | | | | |
| | d tokio_frame_server | | | | | | | |
| | e2 fd_registry | | | | | | | |

(e / f / g / h 預設 recognition:讀題 → 30 秒定界宣言 → 口述 arc,不計全程。)

## clarify 情境卡(每張 5 分鐘)

| 卡 | 做完日期 | 漏問了哪一類(掉不掉/速率/規模/SLA/偵測) |
|---|---|---|
| 1 telemetry hub | | |
| 2 RPC gateway | | |
| 3 market data feed | | |
| 4 log shipper | | |
| 5 sensor bridge | | |
| 6 health prober | | |

## 下次複習

| 日期 | 要複習什麼 | 為什麼 |
|---|---|---|
| | | |
