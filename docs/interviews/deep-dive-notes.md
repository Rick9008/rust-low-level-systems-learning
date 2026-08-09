# Deep dive 參考資料(證據、出處、追問彈藥)

**這個檔不是拿來練的。** 練習用 [deep-dive-projects.md](deep-dive-projects.md);這裡放的是:
為什麼那樣寫、數字的出處、被深追時的彈藥、以及不准講的東西。

場次:**8/10(一)10:00–10:45(Ulysses Kao)——這天只有這一場。**
recruiter debrief(Molly Huang)因 Jan 場改期,已移到 **8/11 10:00–10:15**,接在 Jan 的 coding 場之後。
形式:挖履歷上的專案,一路追問到底。

---

## 面試官必問五題(每個專案講完自己跑一輪)

- Why X over Y?(每個技術選型都要有一個被否決的對手)
- What breaks at 10× scale?
- What was YOUR contribution vs the team's?
- Hardest bug — walk me through the debugging.
- What would you change if you rebuilt it?

## 為什麼是「二 → 一 → 三」這個順序

Etched 的軸是 event loop / telemetry / 低階診斷 / Rust,**不是分散式共識**。所以
**專案二(logging daemon)命中率最高**——它就是一條 telemetry pipeline:多 producer → 路由層 → 一個結構化 sink。
Holdwin 簡報裡的交易對照句(fill/cancel、開盤時間)**全部拿掉**,不要帶進 Etched 場。

## 事實確認項(2026-07-29 Withers 回填,稿已按此修正)

1. PostgreSQL = **14** → 「雙向邏輯複製當時不可用」一句結案;根本理由 = 複製決定不了先後順序、也不知道誰的操作不可挽回(刪除/寄出)。
2. 「客戶端自運轉」真義 = **收信必須自動、不能暫停**——凌晨斷線也不能停收信,不能等人。
3. 觀測手段 = **內建不一致偵測:偵測到就自動重新同步 + 記次數**;半年以上全客戶機器計數 0,且零 support 回報。**這比 hedge 句強,直接當證據講。**
4. CPU-spin 是**前人程式碼的陳年 bug** → 只當 debugging 故事,不認領 failure;個人 failure 用 authz/authn 缺失(culture fit ★7)。
5. logging daemon 實際架構 = **dovecot → syslog → syslog-ng → 自寫 C++ daemon 解析後寫 DB**。
   **別再講「多後端 sink 扇出」那個膨脹版**——sink 就是 DB;多後端 logging library 是**另一件事**
   (各服務發送端的統一介面,−30% 重複碼),兩者分開講。吞吐數字不確定 → **不講數字、不掰**。
6. (2026-08-08 照唸兩連抓後結案,履歷為準)專案二 Q6 的兩個 bug 故事(FD 耗盡、CPU 空轉 0-byte read):
   Withers 先記成「moderation 的」、再記成「同一個事件(EOF 空轉→fd 累積→中斷)」,兩次都自承不確定
   → **裁決:全部照履歷講**(`withers_resume.tex:109-110`,規則同 3,500):**兩條獨立 bullet、
   兩個獨立事件,分開講、不合併、不斷言因果鏈**;「部署尖峰+DDoS」是履歷原文,非 7/29 起草黏合劑。
   履歷未指名服務——①是 on-call 處理的**平台級中斷**(mail 收發全斷),②只說 "code I inherited"。
   歸屬句已改成 on-call/平台框架,**不講 moderation daemon**(記憶不確定、履歷也沒寫)。
   分開講每一半各自為真;被追問細節記不清就守層級、不編。8/8 內若回憶起確切 component 可再補,8/9 起不改稿。
   **(8/9 晚追補,記憶源頭結案)**:Withers 的「moderation+狂刷 log+fd 累積」記憶查證後屬於**第三個故事**
   ——G&L 白板 strace 卡 = `dbg!` 灌爆 /var/log/systemd 400MB → pipe 壞 → panic,**即專案三 Q6 的 451 story**。
   三個事件三個家:履歷兩條(專案二 Q6,分開講)+ dbg!/451(專案三 Q6)。「EOF→fd 累積→炸」的因果鏈
   無任何當時紀錄支持,是三事件縫合,**上場不講**;被追 fd 根因 → 照履歷答尖峰+DDoS,更深根因誠實說沒做。
   **(8/9 晚終版:誠實 hedge 句)**Withers 堅持記得機制(busy-wait 連線不放 fd+洪峰開新連線→撞上限,
   工程上成立)→ 折衷:可講但標成懷疑,一句上場:*"I've suspected the two were related — a connection
   stuck in that busy-wait loop never releases its descriptor. But I never verified the link, so I won't
   claim it. What I verified is: the spike exhausted the descriptors, and the read loop was missing its
   EOF check."* 任何追問的答案都是 "I didn't — that's why they're two findings",與「不編數字」同品味。

## 8/7 進度

✅ ★6 conflict 人物與場景補真;✅ ★3 Moderation 三細節補真;✅ 專案三第 6/7 題以 repo 實證落地;
✅ 兩個練習檔重構成「問題 + 英文 + 中文」。
~~殘項:8/9 taper 重讀 Gjengset 40-hours 原文確認轉述正確~~ **✅ 8/9 已驗(Claude 抓原文對表)**:轉述成立;原句 "Crunch time is fine as long as there is a recovery period afterwards" + area-under-the-curve 論證,兩錨點已入上場包 ★3。demo 故事=論點的教科書案例。

---

# 專案一:預期追問

- **Redis 在關鍵路徑是不是單點?** → 誠實講 persistence 設定 + AOF fsync 視窗。
- **consumer group / PEL / XAUTOCLAIM,訊息卡在 PEL 怎麼辦?**
- **message-ID 表會一直長大?** → 時間窗回收,**窗口必須大於最大重試視窗**。
- **兩邊都做了不可逆操作怎麼辦?** → 設計上不允許同時發生;若真可能就得退回 fencing / witness——**誠實說這是設計邊界。**

---

# 專案三:全部證據

## 🔴 保密紅線(講法,不是道德課)

**講結構與教訓,不講可被利用的細節。絕對不碰這三件**:

1. gRPC 曾經 fail-open 成明文。
2. attachment 路徑曾可任意讀檔(以 root)。
3. token 與 Redis 密碼共用同一份 secret。

那是雇主出貨產品的攻擊面,**講給第三方公司零加分、風險全歸你**。也不報內部 ticket 編號、主機名、檔案路徑。
★7 的 authz 故事同一把尺:停在「我漏了授權檢查、後果量級、我怎麼改流程」,**不描述怎麼利用**。

⚠ **8/7 特別提醒**:那份 SMTP.Moderation 授權缺失的交接也在這條紅線內。它是你最近做的、也確實有意思,
**所以特別容易被「最近在做什麼」勾出來。**

## 這個 repo 是誰

`/synosrc/git_source/libsynomailserver-moderation`(Cerberus,daemon `syno_mail_moderation`)。
**它就是 culture fit ★3 的 content-moderation 專案**——William Hsieh 是 repo 第二大貢獻者(106+45 commits),
**兩個故事同一份 codebase,面試官串起來是加分。**

⚠ 履歷若寫 "microservices"(複數):這個 repo 是**一支 daemon + 兩個 transport**
(milter 用的 localhost 二進位協定 + 節點間 gRPC)。被追問「哪些 service」要答得出來,別讓複數變成沒下文的字。

## 設計的出處(file:line)

| 講的東西 | 出處 |
|---|---|
| 核心對半切給 Tokio / Rayon,兩邊各 ×2 | `src/server/mod.rs:54-66`,你自己的 commit `66ab3e8`(2025-04-03) |
| `spawn_blocking` 跳離 async runtime(含 6 行理由註解) | `src/database/rule/rules.rs:161-175` |
| 三層 `par_iter`:~40 matcher / 每附件 14 個 content matcher / per-recipient | `src/database/pattern/mod.rs:408, 489, 545, 592` |
| `ArcSwap` / `OnceLock<ArcSwap>` 快照 | `src/database/cache.rs:15`、`src/matcher/pii.rs:17`,commit `d9a53de` |
| 無上限 accept → spawn(沒有任何入場控制) | `src/server/command_server.rs:38`;掃 `Semaphore\|max_concurrent\|concurrency_limit\|backpressure` 全 `src/` **零命中** |
| 2× 超訂、未量測、至今未修 | `docs/TECHNICAL_REVIEW.md` §2.4(LOW-MEDIUM) |

> **`ArcSwap` 那個 commit 是 trade-off 的好料**:它取代的是 Rayon worker 裡的一把全域 `blocking_lock()`,
> **那把鎖把 PII 平行化的效果整個抵消掉了。**

## 數字:能不能講

| 數字 | 判決 | 依據 |
|---|---|---|
| ~300× max concurrent connections、12.5× average QPS | ✅ 講 | 隔壁團隊那三個 MR,`perf` + load benchmark 驗過。**說清楚是另一個 service** |
| 端到端吞吐(可引用、可重現) | ✅ 最強的位置 | `tools/load_test.py` + `docs/throughput-load-test-results.md`(MR !65)。ground truth = **daemon 自己的 counter delta**,每格 3 次取中位數。**四核 DSM 真機實測**:K=1 **245**/s、**K=32 ≈1,081/s(甜蜜點)**、K=64 掉到 **477**/s。⚠ 同機變異 **±30–40%**,跨天絕對值不可比 |
| ~3,500 msg/s @ 8 cores(履歷那句) | ✅ 照履歷講,不改口 | 手上實測格子是四核機的 1,081/s @K=32,而那次壓測 client 自己吃掉 1.5/4 核。八核那個數字是同一條線往上推的,量級站得住。**唯一注意:不要主動報精確數字**,用 "order of a few thousand a second per node" 的粒度。真被逼問「四核還是八核、怎麼推的」→ "that figure's from our load runs on a smaller box scaled up — the measurement I'd stand on is the shape, not the digit." **講形狀不講位數。** |
| PII ~100ms/KB、Rayon 排程 ~100μs、P50<10ms/P99<50ms | 🔴 別當實測講 | 前兩者檔案自己標「估計」,第三個是 **target**(`docs/TECHNICAL_REVIEW.md:420`) |
| 0 `unsafe`、~16k LOC、67 個 Rust 檔 | ✅ 可講 | `withers_doc/07_code_review.md:305-317` |

## 三層成本模型(本節主菜,三組獨立量測全是你自己跑的)

| 層 | 每封成本 | 來源 | 誰是瓶頸 |
|---|---|---|---|
| **文字比對**(matcher) | **個位數 µs**(@1KB release 序列:sparse 0.82 / medium 3.90 / heavy 5.36µs) | `bench_dispatch.rs`,8/7 release 實跑 | ❌ 永遠不是——除非 PII 開著 |
| **請求路徑**(每封一條 TCP) | **~1ms**(1,081/s @K=32,4 核) | `tools/load_test.py`,四核 DSM | ✅ **pass-through 路徑的天花板**;K=64 退化到 477/s 就是證據 |
| **DB 寫入**(`new_moderation`,只在規則命中時發生) | **5–50ms(實測,四核 DSM)** | 你自己實測 | ✅ **moderation 路徑的天花板** |

**兩件事因此被釘死**:

1. **3,500(≈286µs/封)不可能是比對階段**——比對只要 4µs,差 70 倍;它必然是端到端。
2. 端到端能跑到千位數/秒,**只有在信不觸發規則、因此沒有 DB 寫入的 pass-through 路徑上**才可能
   (load test 用的正是「1 條 light rule + 100B 信」)。**規則命中的信會慢一到兩個量級。**
   這個 scope 不是缺點,它就是生產環境的常見情況——但**被追問「被隔離的信呢」時要答得出**
   「差一到兩個量級,因為那條路上有 DB 寫入」。答得出來 = 你知道成本在哪;答不出來 = 那個數字是背的。

**兩條獨立路徑得到同一個結論(這點很值錢)**:load test 從系統面看出「比對是微秒級、時間花在 syscall、
填滿 CPU 不會讓吞吐變高、要燒滿 4 核得開 PII」;8/7 的 release micro-bench 獨立量到同一件事
(比對 4µs、fan-out 地板 40–170µs)。**不同工具、同一個結論 = 可以放心講,不是單點量測的運氣。**

## debug build 校準那件事:完整證據

> ⚠ **自我更正紀錄**:8/7 稍早先寫成「主變因是 Rayon 池寬」——那是只看 debug 數據的結論,**錯了**。
> 補跑 release 後主變因是 **build profile**,量級大一級;池寬是同方向的次要效應。
> 留這行是因為它本身就是教訓:**先跑完矩陣再下結論。**

**2×2 矩陣(debug/release × Rayon 4 條/16 條),平行首次勝出的位置**:

| profile | committed(4 核機,debug) | debug/4 條 | debug/16 條 | **release/4 條** | **release/16 條** |
|---|---|---|---|---|---|
| medium | 1–2KB | 2KB | 4KB | **50KB** | **50KB** |
| heavy | 1KB(宣稱一律平行) | 1KB | 2KB | **50KB** | **50KB** |
| sparse | 永不 | 永不 | 永不 | 永不(0.98–1.00×) | 永不 |

**一個模型解釋全部四次跑**:交叉點 = **序列工作量超過 fan-out 固定成本(平行的地板)的那一點**。

| | fan-out 地板 | medium @1KB 序列工作量 |
|---|---|---|
| debug / 4 條 | ~120µs | 85.9µs → 相當 ⇒ 交叉點在 1–2KB |
| debug / 16 條 | ~150µs | 85.9µs |
| **release / 4 條** | **~40µs** | **3.44µs** → 差 12 倍 ⇒ 右移到 10–50KB |
| **release / 16 條** | **~80–170µs** | **3.90µs** → 差更多 |

**兩個變因,主次分明**:

1. **build profile 是主變因**。release 讓 matcher 工作便宜 **20–60 倍**,但 fan-out 成本幾乎沒變——
   **因為那是同步不是運算**:喚醒 worker、atomics、工作佇列交棒,不會因為開了最佳化就快 25 倍。
2. **池寬是次要效應、同方向**:地板從 4 條的 ~40µs 漲到 16 條的 ~80–170µs;但大 payload 天花板更高
   (heavy@500KB:2.68× → 3.25×)。**池寬換的是「交叉點右移 + 漸近線上升」。**

**shipped 常數的嚴重度(release + 池寬=核心數,即 daemon 實際設定)**:

| CostClass 規則 | 判決 | 證據(release / 16 條) |
|---|---|---|
| `≤1 active → Light`(一律序列) | ✅ 對 | sparse 全程 0.98–1.00× |
| `≥8 active → Heavy`(一律平行) | 🔴 錯,很嚴重 | heavy@1KB:序列 **5.36µs** vs 平行 **161.64µs** = **慢 30 倍** |
| `2–7 active → Adaptive @ 2000 bytes` | 🔴 門檻差 ~25 倍 | medium@2KB:序列 **6.36µs** vs 平行 **82.27µs** = **慢 12.9 倍**。正確交叉點在 10–50KB |
| `PII active → Heavy` | 🟡 未驗證 | benchmark **完全沒含 PII**;理由裡的 100ms/KB 本身是估計值 |

**而且它平反了被推翻的那個數字**:branch 的 results doc 寫「This invalidates the old 50KB crossover」,
但那句是在 debug build 上得到的。**release 下交叉點回到 10–50KB,舊的 50KB 啟發式大致是對的**——
**被推翻的一方比推翻它的一方更接近事實。**

**追問深度(留手,別主動全倒)**

1. **它還會隨池寬移動**——daemon 把 Rayon 執行緒設成 = 核心數,所以門檻是**部署機核心數的函數**,根本不該是常數。
2. **正確修法**:判準換成「估計的序列工作量 vs 實測的 fan-out 地板」,工作量 ≈ bytes × active matcher 數,
   地板在**啟動時對實際池寬量一次**。
3. `PII → 一律平行`那條要補量,不要留在估計值上。
4. **推論**:整套 adaptive dispatch 在優化一段本來就免費的路;唯一真的有影響的 PII 那格正好是 benchmark 沒涵蓋的。
   **修法一句**:先量每個 stage 的實際占比,再決定要不要有 dispatch policy。
   ⚠ 講的時候用「我下次會先量再做」的語氣,**不要自我否定過頭**。

⚠ **誠實邊界**:那條分支**未 merge**,所以**沒有 production bug**。
正確說法:"before that branch merges, the threshold has to be derived, not baked."
**講成「線上出事」會被抓。**

**可重跑的證據(四條命令,~10 分鐘)**

```sh
cd /synosrc/git_source/libsynomailserver-moderation   # branch: feat-adaptive-dispatch
BENCH_DISPATCH=1 cargo test --lib  database::pattern::bench_dispatch -- --nocapture --test-threads=1
RAYON_NUM_THREADS=4 BENCH_DISPATCH=1 cargo test --lib database::pattern::bench_dispatch -- --nocapture --test-threads=1
BENCH_DISPATCH=1 cargo test --release --lib database::pattern::bench_dispatch -- --nocapture --test-threads=1
RAYON_NUM_THREADS=4 BENCH_DISPATCH=1 cargo test --release --lib database::pattern::bench_dispatch -- --nocapture --test-threads=1
```

## Q6(最難的 bug)的兩種收尾——只講真的做了的那個

**已加 lint 版**(`#![deny(clippy::dbg_macro, clippy::print_stderr)]` 真的進了 crate):

> "And the prevention I care about is the one that doesn't rely on memory: the crate now denies `dbg!` and `print_stderr` outright. `-D warnings` alone didn't catch them — both live in clippy's restriction group, allow-by-default — so it had to be an explicit deny. **A written invariant tells the next reader; a lint tells the compiler.**"

中文:我在意的防止方式,是不依賴記憶的那種:這個 crate 現在直接禁掉 `dbg!` 和 `print_stderr`。
光靠 `-D warnings` 抓不到它們——兩者都在 clippy 的 restriction 群組、預設是 allow——所以必須是明確的 deny。
**寫下來的不變量是告訴下一個讀的人;lint 是告訴編譯器。**

**未加 lint 版**(照現況):

> "And the honest ending: the real prevention — a lint that rejects `dbg!` and `eprintln!` in this crate — still isn't there. Our CI runs clippy with `-D warnings`, but those two live in clippy's restriction group and are allow-by-default, so they slip through. It's a two-line attribute I should have added then."

中文:誠實的結尾:真正的防止——一個在這個 crate 裡拒絕 `dbg!` 和 `eprintln!` 的 lint——**到現在還沒有**。
我們 CI 有跑 clippy 加 `-D warnings`,但那兩個在 restriction 群組、預設 allow,所以會溜過去。
**那是我當時就該加的兩行 attribute。**

🔴 **鐵律:面試不能講沒做的事。** 上場前 30 秒自問「這句現在是真的嗎」,兩版選一版,不要混。

## Q6 不准編的部分

commit 記了症狀鏈(mimedefang 的錯誤字串、451、兩個前提條件),**但沒記你當時用什麼工具、什麼順序查到的**。
練習本裡的稿子只從「已記錄的症狀鏈」往下推理,這是誠實的。
**被追問「你怎麼縮小範圍的」不要生出 profiler / bisect / flamegraph**——repo 全域沒有任何這類工具的痕跡。

出處:`e71077d`(2026-05-06,Ref 6362,author witherslin,reviewer joerao)、
不變量註解 `src/server/service.rs:293-300`(「we have been bitten by this before」)、
續集 `1ae6363`(2026-05-14,Ref 6384,**由你自己新寫的 `from_socket` 附件測試打出來的**)、
型別修法 `src/server/service.rs:301-321`、CI `.gitlab-ci.yml:42`。

## 備用 bug 故事(依題型選,不要一場講三個)

**「我錯了 / 你被證明是錯的那次」題的最佳解 = DB 連線池**

你把 `max_connections(5)` 改成 `(cores*3).max(10)` + `min_connections=10` 想解「連線池餓死」(`8e3fe40`),
結果被同事在 bug ticket 下 revert 回 5(`07142ee`),你隔天自己寫出根因(`892a534`):
Cerberus 不直連 PostgreSQL,中間有 pgbouncer `pool_mode=session`、`default_pool_size=5`;
超出的連線在 pgbouncer 無限排隊,SQLx 的健康檢查 ping 卡在殭屍連線上、**佔住 semaphore permit**,
最後**全部**查詢 15 秒後 `pool timed out`——**比它要解的餓死更糟**,而 `min_connections=10` 讓它在啟動就發生。
在 NAS 上用 **8 個情境的 POC、100% 可重現**驗掉。

教訓可直接送給 Etched:**你設的資源上限只是一疊上限的最上層;下游一跳有個 session-mode proxy 卡你 5 條時,
`num_cpus` 算出來的數字沒有意義。** 附帶第二個訊號:這個 POC 是去**推翻一條被標成 MEDIUM-HIGH 的 review finding**,
不是去證實它。

**「最好的機制性防止」題 = RegexSet 單位錯置**

1MB `size_limit` 你以為是 per-pattern,實際是 per-`RegexSet`(runtime 的編譯單位);超限時 fallback 成
`RegexSet::empty()`,整個欄位的 regex 規則**靜默失效**——**moderation 產品靜默不比對是 fail-open,不是降級**。
而且寫入端驗證用的是「單條 pattern」這個不同的單位,所以存得進去、只在 runtime 爛掉。

三層修法:①把驗證單位對齊編譯單位(`validate_regex_set`)②`Compiled::PerPattern` fallback =
**降級不消失**,只丟單獨超限的那條、還帶著 index 讓 pattern id 不錯位 ③4 個以不變量命名的回歸測試。

一句話教訓:**在你實際執行的那個單位上驗證,而且 fallback 要降級不要消失。**
⚠ 這題**沒記「怎麼發現的」**,別編發現過程。

## Q7 的追問深度與備用答案

**追問深度(留著,別主動全倒)**

1. **第三個池沒人管**——`spawn_blocking` 的 blocking pool 從沒設過 `max_blocking_threads`(掃 `src/` 零命中),
   吃 Tokio 預設上限 512;真實路徑是 tokio worker → blocking pool(**無界**)→ Rayon global pool,
   **被仔細算過的 50/50 核心切分只管到頭尾兩個,中間那個是無界的。** 這條是你自己的分析、review 文件沒寫,收尾很強。
2. Rayon 迴圈裡還有同步、無大小上限的檔案讀取(`src/database/pattern/mod.rs:494`,
   `std::fs::read_to_string(...).unwrap_or_default()`)——大附件可能 OOM,且讀取錯誤被靜默吞掉。

**備用第 7 答(被追問流程 / 架構題時用)= refactor 的順序**

你把 5 個 domain service trait 照 strangler-fig 寫在舊 `ServiceExt` god-trait 旁邊,理由寫成「zero-risk deployment」。
結果 8 個月後 `src/server/domain/` 的 1,611 行**連 `mod domain` 都沒宣告**(`src/server/mod.rs:1-7`),等於沒編譯;
`TransactionManager` 掛著 `#[allow(unused)]` 和 `// Todo(withers)`、production 零呼叫者;
而 `grpc_service.rs` 與 `sync_service.rs` 是同一套冪等 + 交易邏輯的**兩份逐行複製**
(review 評 [嚴重],真正代價 = 「未來修 bug 只改一份」+「給 reviewer『已重構』的錯覺」)。

教訓是**順序**:「零風險」是假的——把目標架構寫在舊架構旁邊、卻一條呼叫路徑都不接,
等於把零風險換成永久重複加上假的完工訊號。
**該做的是一次搬一條垂直切片、並在同一個 MR 裡刪掉舊路徑,讓 refactor 不可能半途而廢。**

⚠ 這個答案要以「我沒收尾的工作」自己認,不能怪 reviewer;風險是聽起來「不收尾」,**所以它是備用不是首選**。

## 若你已經回去修了 concurrency limit

Q7 **不需要改寫**——它問的是「當初的決定」,不是現況。只在最後一段前加一句:

> "That's not hypothetical, by the way — I went back and put the limit in."

還沒修就**主體一字不動**(「rebuilt, I'd…」本來就是條件式,沒有謊)。🔴 那句加了就必須是真的。

## 「hardest problem you resolved」問法 → moderation AA(8/9 定,源:Heptabase G&L 白板)

四拍:①ambiguity 處境(沒人定義需求、與 master-sync 架構假設衝突;**開場別倒架構**)②先定義未決事項(PM 問意圖/staff 問約束、拍板斷線各自運作+重連最終一致、列 case 寫 spec 跑 review)③irreversible actions win(專案一 Q3 金句直用)④半年零事故+真 failover(專案一 Q1 尾句直用)。新英文只有 ①②,見 8/9 對話;③④ 用練習本現句。用這題開場=把 deep dive 導進專案一主場。

## 專案一設計補真(2026-08-09,Withers 口述)

健康時**每封信有 home node**(接收它的那台),對該信的操作導回 home——常態單一寫者、無衝突;
對側只持 replica,**斷網/對台死掉才 promote replica 接管**。敘事升級:衝突只存在於分區窗口,
irreversible-actions-win 規則就是為那個窗口設計的。上場句在上場包專案一格(8/9 補真版)。
⚠ 別再講「兩台獨立做所有事、狀態全共享」的簡化版——那會把單寫者設計講丟。

## 8/9 四題補真結案(Withers 逐條口述,上場包已同步)

1. **FD 止血 = 先調大 ulimit**(可講);當時靠哪個訊號看到 fd 滿的**已不記得**——被問守形狀
   ("process alive, every new accept failing"),不指名工具。
2. **home-node 路由親和 = HAProxy 規則**實作。
3. **failover 全自動**:對 peer 連線失敗三次 → 判定斷線 → 從 moderation_replica tables 撈資料、
   按 user 的 approve/reject 執行(寄出或刪除)→ 每次接管寫一筆 ownership-take(target_uuid)紀錄
   → peer 回來**必須 challenge ownership** 才能再動作。無人工介入。
4. **專案一 10× 無實測容量數據**:講「信量 ×10、兩節點是產品形態」+「不編數字」句接結構分析。
