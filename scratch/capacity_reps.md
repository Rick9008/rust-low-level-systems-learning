# 容量速算 reps(7/21 出題;5 分鐘/題,結帳表紀律)

規則:每題交出**算式 + 數字 + 荒謬檢查 + 一句裁決**。英文寫。
對答案:寫完喊 Claude 批改;沒過結帳條件(數字/式子/裁決三缺一)重寫該題。

## R1 metrics agent

An agent samples 2,000 counters every 10s, 32 bytes each, ships in 60s
batches. The collector can be unreachable for up to 5 minutes.
Size the buffer. State your drop policy and when it activates.

Answer 7/31:

We have 2000 counters need to memorize,

For the data size: 2000 \* 32 bytes = 64000 bytes ~= 64 KB per 10s
6.4 per sec

Ships once in 60s: 6.4 \* 60 = 384 KB
and we have 5 minutes = 384 \* 5 = 1920 KB ~= 1.9 MB
So the buffer might be 1.9 MB if we must to keep every records
If we can aggregate the counter, we can only use 64 KB buffer because the counter .

So if we cannot aggregate the counter, under pressure we can drop the oldest data when the capacity is full, and record how many times we drop data.

If we can aggregate the counter, then drop policy doesn't need, because we can use conflation slots to resolve this problem.

Final, we don't have to aggregate because memory usage is low,
so we use 2MB ring, zero loss within the 5-min spec. Beyond spec, conflate to latest-per-counter and count drops.

> **批改 2026-07-31(v5)——✓ 收**
>
> - 主鏈修復(÷10s 那步寫出來了)、conflation 64 KB ✓、裁決句(2MB ring/超規降級)✓。
> - 記檔兩小件(不擋收帳):① `6.4 per sec` 單位又掉——寫全 `6.4 KB/s`,這正是本卡病根;
>   ② Cross 行仍沒寫(`200/s × 60 s = 12,000 × 32 B = 384 KB` 那條)。
> - line 22 斷句補完:because counters are **cumulative** — the latest value is enough。
> - 下次動筆先寫五行頭:**Given / Chain / Cross / Sanity / Verdict**(骨架已入 capacity-four-shapes.html)。

> **批改 2026-07-31(v4)——⚠ 主帳 1.9 MB ✓,補四行**
>
> - ✓ 1.9 MB、drop-oldest+計數、conflation 高分件(想法)全到位。
> - ✗ `2000 × 32 = 64,000 B ≈ 6.4 KB` 斷環:64,000 B = **64 KB/snapshot**;你腦中做了 ÷10 s 沒寫出來。
>   正確行:`64 KB / 10 s = 6.4 KB/s`。下一行能對是因為 6.4 被當 KB/s 用——印出來的帳仍是錯的。
> - ✗ 同一斷環傳染:conflation 塌縮底 = `2000 × 32 B = 64 KB`,不是 6.4 KB。
> - ✗ 兩路互驗沒寫:記錄法 `200/s × 60 s = 12,000 × 32 B = 384 KB/批` 必須和 rate 法同歸。
> - ✗ 荒謬檢查第四度缺席 → 新規:裁決句前固定一行 `Sanity:`,沒有不收筆。
>   (這題的一行:1.9 MB 對一個 host agent 毫無壓力;若答案出現 GB 級才要警鈴。)

Answer:
200 counters per second, 32 bytes each counter, ships in 60s batches
so we have 60 \* 200 \* 32 bytes per batchs, and 1 minutes send once.
12000 \* 32 bytes ~= 320 KB per batch one minute
and the collector be unreacheable for up to 5 minutes, so we need 32 KB \_ 5 = 160KB capacity size
so we can hold 160 KB size for the buffer.
And the drop policy we might choose drop the oldest data, and when the collector cannot consume the data fast enough, which cause the buffer reach maximum size, it will activates

> **批改 2026-07-30(v3)——✗ 重寫數字鏈 + 補荒謬檢查**
>
> - ✓ 正規化 2,000/10s → 200/s 合法(批改者前兩輪誤判為抄題錯,收回;v2 的 1.8 MB 其實已是對的帳)。
> - ✗ **掉零**:60 s × 200/s = **12,000** 筆/批,不是 1,200 → 12,000 × 32 B = **384 KB/批** → ×5 ≈ **1.9 MB**,不是 160 KB。
>   另:38,400 B 捨成 "32 KB" 太野——保留兩位有效數字,免費。
> - **兩路互驗**:rate 法 6.4 KB/s × 300 s ≈ 1.9 MB,批次法必須殊途同歸;對不上就是有一條錯。
> - ✗ 荒謬檢查三度缺席(結帳必交)。注意:160 KB 在合理量級內,**荒謬檢查抓不到掉零**——逐步帶單位的帳本才是主保險。
> - policy:drop-oldest ✓ 在這裡**合法**(自家 userspace buffer,你有 pop 權;對比早上 ISR 的 try-push-only ring——講得出這個對比是加分)。
>   「when it activates」翻成規格語言:buffer 照 5 min 配 → **啟動 = 斷線超過 5 分鐘,規格內零丟失**。
>   高分件(沒拿):counter 是累計值 → 降級天生是 conflation(每 counter 留最新),記憶體塌縮成 O(counter 數) = 64 KB。

## R2 connection memory

A TCP proxy holds 50,000 concurrent connections; each needs a 16KB read
buffer + 16KB write buffer. Does it fit in 4GB RAM? If not, what do you
change first? (荒謬檢查這題是主角)

Answer 7/31:
for the user space we use 50000 \* 32 KB size ~= 1.6GB, it will fit when the connections are almost suspend rather than busying.
if it comes to busy, we should dynamic allocate the buffers and back pressure for the connections data input.
If we must keep every connection, we should use cluster nodes to receive the connections.
1.6 GB / 4 GB = 40% userspace; if kernel has same order of magnitude: the send / receive buffer. Then the memory will go double.

> **批改 2026-07-31(v5)——✓ 收(v3 的 ⚠ 清帳)**
>
> - 40% userspace + kernel 同量級 + 翻倍,齊了。
> - 潤兩處:去掉 "if"——kernel per-socket buffer 是事實不是假設;句尾掛 `ss -m` to verify,免費加分。

> **批改 2026-07-31(v4)——✗ 欠的那行還是沒來**
>
> - 你補的是 cluster scale-out(可以,但 "change first" 已由 lazy alloc+backpressure 答掉,這句是延伸不是欠帳)。
> - 欠的仍是**寫出來的餘裕檢查**:`1.6 GB / 4 GB = 40% userspace;kernel 每 socket 另有同量級
收/送 buffer(重傳底稿/收貨窗口/亂序重組),全忙總量翻倍以上;ss -m 驗證`。補這行即收。

Answer:
For the user space, we use 50000 \* 32 KB size ~= 1.6 GB, it will fit when the connections are almost suspend rather than busying.
If it comes to busy, we should dynamic allocate the buffers and back pressure for the connections data input.

> **批改 2026-07-30(v3)——⚠ 補一行即收**
>
> - ✓ 1.6 GB 對;idle/busy 分岔 ✓;lazy allocate + backpressure 的裁決 ✓(backpressure 是自己加的,好)。
> - ⚠ 缺**寫出來的**餘裕檢查(題目明標它是主角):`1.6 / 4 = 40%;kernel 每條 socket 另有同量級的收/送 buffer
(重傳底稿/收貨窗口/亂序重組),全活躍時總量翻倍以上` + 驗證工具一句(`ss -m`)。補這一行,此題收。

## R3 prober 反推

Dead nodes must be flagged within 20s. You chose debounce N=3 and
per-probe timeout 1s. Derive: max probe interval, probes/s for 500
targets, and worker-pool size if a probe can hold a worker for 1s.

Answer 7/31:

Assume each round waits out the full 1s timeout before the next starts,
max probe interval: 3T + 3 <= 20s, so we can pick T as 5 sec

probes/s for 500 targets: 500 / 5 = 100 probe/s

worker-pool we can use maybe 2 ~ 4 are enough, which is the regular rate \* probe time(maybe only 20 ms?) ~= 2 busy thread
but when every probe try to connection holds full 1s timeout, we need 100/s \* 1s = 100 workers
100 \* 8 MB stack = 800 MB, too large. -> Because the probe operation is IO-bound, we should go async.

> **批改 2026-07-31(v6)——✓ 收,全案收攤**
>
> - 假設句 ✓、風暴劇本(全部吃滿 1s timeout)✓、裁決 async ✓。
> - 潤一句:async 之後補 "still sized for ~100 in-flight probes"——async 省的是 thread 成本,不是併發數。
> - **三題結案(7/31)**:R1/R2/R3 全收。後續維護走五行頭骨架(capacity-four-shapes.html 分頁②),
>   8/4 背靠背模擬埋 sizing 小題實戰、8/5 taper 默寫行頭;本檔不再開新題。
>
> - ✓ pool=100 主答案到了、800 MB → async 的 Sanity 到了,順序也對(平時 2–4 退居第二句)。
> - ✗ 模型假設句**二連缺**:`3T+3` 前加 "Assuming each round waits out the full 1s timeout
>   before the next starts: 3(T+1) ≤ 20 → T = 5."(或改 3T+1 ≤ 20 → T=6,講出選哪個)。
> - 風暴劇本更正:是「全部吃滿 1s timeout(斷網)」,不是 "connect at the same time"。
> - 補一句假設 + 一句裁決(provision 100 workers, or go async)→ 全案收攤。

> **批改 2026-07-31(v4)——⚠ 前兩環過,第三環答錯題**
>
> - ✓ `3T+3 ≤ 20 → T=5` 數學對(串行保守模型);✗ 但**模型假設句沒講**——「我假設每輪等完
>   timeout 才起下一輪」那句才是分數所在(另一合法模型 `3T+1 ≤ 20 → T=6`,講出選哪個就好)。
> - ✓ 100 probes/s。
> - ✗ pool 主答案漏了:題目給定佔用 **1 s**,問的就是風暴配置 → `100/s × 1 s = 100 workers`(Little's law)。
>   你的 2–4 是「平時忙碌數」——那是第二句,不是主答案。正確順序:pool=100(全 timeout 風暴)
>   → 平時 rate × 20 ms ≈ 2 個在忙 → Sanity: 100 × ~8 MB stack ≈ 800 MB 太肥,probe 是 IO-bound → async。
>   (你把 async 那句寫出來了 ✓,但它掛在 2–4 上,失去了它要救的對象。)

max probe intervals should be 20 / 3 = 6.667 secs

probes/s for 500 targets: 500 / 20 \* 3 = 75 probe/s

worker-pool size is 75.

> **批改 2026-07-30(v3)——✗ 第一環忘了 timeout;後兩環形狀全對**
>
> - ✗ `20 / 3` 沒把 timeout 放進鏈。驗算你的 T=6.67:最壞偵測 = 3×6.67 + 1 = **21 s > 20 s,爆 SLA**。
>   正確鏈(最壞起點=死在剛探測完):`3T + 1 ≤ 20 → T ≤ 6.33 → 取 6 s`;
>   保守模型(串行發射)`3(T+1) ≤ 20 → T ≤ 5.67 → 取 5 s`。**講出你選哪個模型假設,分數在那句。**
> - ✓ rate = 500/T、pool = rate × 佔用 1s——這就是 Little's law(形狀 4),你已經在用了;
>   數字隨 T 重算:T=5 → **100 probes/s、pool = 100**。
> - 補一句才算答完:pool 是為「機房斷網、全部吃滿 timeout」的風暴配的;平時 rate × 20 ms ≈ 2 個在忙。
> - 荒謬檢查:pool=100 個 thread 合理嗎?→ 100 × ~8 MB stack ≈ 800 MB?→ 講一句「probe 是 IO-bound,
>   換 async 或縮 stack」就是額外加分。
>
> **三題共同帳**:荒謬檢查 0/3 出現——它是結帳表必交項,寫一行也要寫。
> 詳解與錨點數字庫:`html_p/capacity-four-shapes.html`(2026-07-30 夜讀)。
