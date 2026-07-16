# 學習進度(手動勾選,git 追蹤)

面試日:**2026-07-28**。三欄語意:讀 = reference 讀懂能講;drill = 填完轉綠;
challenge = 空白手搓過(★ 才有)。日期一律寫絕對日期。

## 模組 × 三層

### TPS 優先(README 學習路徑 1–11)

| # | 模組 | 讀 | drill | challenge | 備註 |
|---|---|---|---|---|---|
| 1 | iter_mutate | ☐ | ☐(7 洞) | — | 手感基礎,先做 |
| 2 | bounded_queue | ☑ 2026-07-16 | ☑ 2026-07-16 | — | push/pop/close 已填,MPMC stress 過 |
| 3 | thread_pool | ☐ | ☐(+submit/join 2 洞新增) | — | |
| 4 | ring_buffer | ☐ | ☐ | — | |
| 5 | spsc_ring | ☐ | ☐ | ☐ ★ | 目標:空白 20 分鐘、一次編過、寫三遍 |
| 6 | executor | ☐ | ☐ | ☐ ★ | |
| 7 | lru | ☐ | ☐ | ☐ ★ | |
| 8 | fd_registry | ☐ | ☐ | — | JD sleeper;彩排 e2 |
| 9 | hw_bridge(protocol+framer) | ☐ | ☐ | ☐ ★ | |
| 10 | dsu | ☐ | ☐ | ☐ ★ | |
| 11 | sharded_map | ☐ | ☐ | ☐ ★ | |
| 12 | signal_pipeline | ☐ | ☐(2 洞) | ☐ ★ | JD 本尊圖;掛牌握手 = SeqCst 實戰位 |

### 次優先

| 模組 | 讀 | drill | challenge |
|---|---|---|---|
| inplace_leetcode | ☐ | — | — |
| graph | ☐ | ☐ | — |
| trie | ☐ | ☐ | — |
| tree | ☐ | ☐ | — |

### deep-dive(讀懂能講即可,不手搓)

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
