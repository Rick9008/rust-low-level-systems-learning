# 容量速算 reps(7/21 出題;5 分鐘/題,結帳表紀律)

規則:每題交出**算式 + 數字 + 荒謬檢查 + 一句裁決**。英文寫。
對答案:寫完喊 Claude 批改;沒過結帳條件(數字/式子/裁決三缺一)重寫該題。

## R1 metrics agent

An agent samples 2,000 counters every 10s, 32 bytes each, ships in 60s
batches. The collector can be unreachable for up to 5 minutes.
Size the buffer. State your drop policy and when it activates.

Answer:
200 counters per second, 32 bytes each counter, ships in 60s batches
so we have 60 \* 200 \* 32 bytes per batchs, and 1 minutes send once.
12000 \* 32 bytes ~= 320 KB per batch one minute
and the collector be unreacheable for up to 5 minutes, so we need 32 KB \_ 5 = 160KB capacity size
so we can hold 160 KB size for the buffer.
And the drop policy we might choose drop the oldest data, and when the collector cannot consume the data fast enough, which cause the buffer reach maximum size, it will activates

> **批改 2026-07-30(v3)——✗ 重寫數字鏈 + 補荒謬檢查**
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

Answer:
For the user space, we use 50000 \* 32 KB size ~= 1.6 GB, it will fit when the connections are almost suspend rather than busying.
If it comes to busy, we should dynamic allocate the buffers and back pressure for the connections data input.

> **批改 2026-07-30(v3)——⚠ 補一行即收**
> - ✓ 1.6 GB 對;idle/busy 分岔 ✓;lazy allocate + backpressure 的裁決 ✓(backpressure 是自己加的,好)。
> - ⚠ 缺**寫出來的**餘裕檢查(題目明標它是主角):`1.6 / 4 = 40%;kernel 每條 socket 另有同量級的收/送 buffer
>   (重傳底稿/收貨窗口/亂序重組),全活躍時總量翻倍以上` + 驗證工具一句(`ss -m`)。補這一行,此題收。

## R3 prober 反推

Dead nodes must be flagged within 20s. You chose debounce N=3 and
per-probe timeout 1s. Derive: max probe interval, probes/s for 500
targets, and worker-pool size if a probe can hold a worker for 1s.

max probe intervals should be 20 / 3 = 6.667 secs

probes/s for 500 targets: 500 / 20 \* 3 = 75 probe/s

worker-pool size is 75.

> **批改 2026-07-30(v3)——✗ 第一環忘了 timeout;後兩環形狀全對**
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
