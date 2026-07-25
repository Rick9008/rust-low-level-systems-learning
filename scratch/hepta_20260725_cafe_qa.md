# 咖啡廳 Q&A —— 2026-07-25 下午沉澱(§8 冷診斷 + TCP/wheel/aggregator 場邊問答,壓縮 7 卡)

> 場景:口袋件補課(ds_sync §8 先答再翻)→ TCP 骨架默寫 6 輪 → aggregator 填綠。
> 源碼證物:`drills/src/ds/telemetry_aggregator.rs`(含自寫鬼資料紅測)|`scratch/tcp_skelton2.rs`(默寫批改在檔頭)
> 相關卡:`hepta_20260724_threadpool_full.md` 卡2(lost-wakeup)|`hepta_20260724_timer_wheel_qa.md` 卡2/3(wheel 選型)
> 待辦:7/26 ds_sync 補洞環(讀 30m + 下午閉卷重烤 + transfer 變體)——**待用戶點頭才排**。

## 今晚出聲場(回家 20:00 開錶 → 00:30 熄燈/07:30 起)

1. **卡#5 口述設計版**(40m):sensor bridge——分 threads/tasks + 定通訊協定 + 五問(JD 複核 #4,首做)。
2. 🔴**e2#2**(45+20):三目標 = clarify 英文出聲 ≥3 問|**boundary 段跑滿**(e2#1 兩洞恰在沒跑到的角落)|trade-off 招牌句。⚠ 參數名 `generation` 非 `gen`(edition 2024 保留字)。
3. 🔴**d#1 tokio_frame_server**(45+20,d 題型首寫;今天默的 TCP 骨架就是它的前置)。
4. **口述錄音 ~45m**(原 55 縮 10):ordering / Waker 鏈 / 選型 + executor×reactor + 五 server p99.9 + unsafe impl 三段式 + litmus + signal_pipeline 扇入 + **Q1 why 層 30 秒英文複測(先講再對答案)**。
5. (提案待裁)卡1+卡2 **口述重打** ~15m 併錄音尾;卡4/卡6 重打放掉;漏問模式表 → 明天 10m 從既有批改整理。
6. 收帳 commit(SCHEDULE 定帳:endian → 明早 08:20|wheel 陣亡|五卡裁決|ds_sync 補洞環要不要排)。

閥門:晚場崩 → 錄音再縮 → 卡#5 縮 20m;**e2#2/d#1 不動**。

## 一句話骨架

**先問「這個寫入承不承載不變量」**——承載 → CAS 失敗必須重試、ordering 必須給足;只是 hint → 失敗棄之、Relaxed 即可。§8 四題、CLOCK flag、pool 的 fast-path check、aggregator 的 epoch 驗印,全是同一根脊椎的切面。

## 卡 1:§8 冷診斷帳(1✓ 2✗ 3半 4半)

- **Q1 DSU ✓**:link CAS 承載連通性不變量(失敗 = expected 永久失效 → 重 find 重試);halving 只是效能 hint(parent 鏈單調向根,棄寫後結構反而更好)。缺的只有「invariant vs hint」總綱詞彙。
- **Q2 CLOCK ✗**:答成「mutex 保證」。正解:referenced flag **不承載任何正確性**,丟 flag 最壞 = 熱 entry 提早逐出,cache 語意允許(miss 只是慢不是錯)→ 無不變量掛在可見性上 → Relaxed 剛好。
- **Q3 arena 半**:方向反了——是「先還槽 → 別人 alloc 同槽開寫 → 跟**自己還沒做的** `assume_init_read` 撞 data race(UB)」。`free_slot` 的 Release-CAS = 「我讀完了」的發佈點,必須排在讀之後。
- **Q4 sharded LRU 半**:答成 contention(吞吐軸);題目問**逐出品質軸**——熱 key 聚簇撞單 shard、容量靜態切分借不過去,等效容量 cap → cap/N。
- 漏法歸類:Q2/Q4 是「軸認錯」不是「不會」;Q1/Q4 純推理可達,Q2/Q3 需先認識模組長相(→ 補洞環的理由)。

## 卡 2:CLOCK = 近似 LRU(1 bit + 時針)

- 痛點:精確 LRU 的 get 是**寫操作**(unlink + 插鏈頭),讀多寫少的 cache 反而在「讀」上打架 → 無鎖版不存在(ds_sync §6)。
- 做法:每 entry 一個 referenced bit;`get` = `store(true, Relaxed)` 完事;逐出 = 時針掃環,見 1 清 0 饒命(second chance),見 0 逐出。
- 本質:精確 LRU 把 recency 編碼在**結構**(鏈序),CLOCK 降維成**一個 bit**——降掉的維度就是付出的逐出品質(分不出 1 秒前 vs 59 秒前)。逐出的是 not-recently-used,非嚴格 least。
- 錨:Postgres buffer pool 就叫 clock-sweep;OS 分頁置換(硬體只給 accessed bit);Redis maxmemory-lru = 抽樣近似。production 幾乎沒人用精確 LRU。
- **待答預測題**:CLOCK 逐出一次最壞掃幾格?什麼情境踩到?(併 7/26 烤場)

## 卡 3:lost-wakeup 卡對回 thread_pool2.rs(店內實地驗證)

- `shutdown()` 那對大括號 = 「store 進鎖」擺法本人;縫 = worker 的 `wait_while` 評完 pred → 掛上等待佇列之間。notify 掉進縫 = 打空佇列蒸發。
- acq/rel 答「我讀到什麼值」;lost-wakeup 問「notify 落地時我掛上佇列了沒」——**read 當下值是新鮮的也沒用**,值是在 read 之後才變的。
- 檔內 acq/rel 幾乎不承載正確性:worker 迴圈兩次 load 都在 jobs 鎖內(鎖給 HB);`execute`/`submit` 開頭的鎖外 load 與 `lock()` 之間**本就有 TOCTOU 縫**(SeqCst 也關不掉)→ 它是 best-effort 禮貌拒絕,漏網 job 由退出條件 `!empty || !shutdown` 兜底跑完。全部 Relaxed 化,行為合約四條一條不破。
- 更狠一層:`shutdown(&mut self)` vs `execute(&self)`——borrow checker 直接把兩者序列化;`Arc<ThreadPool>` 下 Drop 也只在最後一個 handle 消失時跑。**Rust 的 aliasing 模型先幫你判掉一半的並發問題**(rust-five-axis 口袋句)。
- 面試句:"That fast-path check is best-effort by construction — there's a TOCTOU window between the load and the lock — so its ordering carries no invariant. Correctness rides on the mutex and the drain-on-exit condition."

## 卡 4:wheel「O(1) 攤還」的記帳法

- schedule 側是**硬 O(1)**(除法取模 + 掛 list),與現存 timer 數無關;「攤還」是替 fire 側說的——貴的瞬間藏在:①階層輪 cascade 跨層搬一整格 ②rounds 款每圈重摸整條 slot list。
- 記帳法:**成本記在 timer 頭上,不記在 tick 頭上**——一個 timer 一生 = 插入 1 + cascade ≤ L(層數常數)+ 開火 1 = 常數次;heap 每個 op 誠實付 O(log n),n 越大越貴。
- 代價三件:空轉稅(空 tick 也付常數)、next_deadline 最壞 O(SLOTS) 掃(heap peek O(1))、精度被 tick 量化。
- **rounds 款的 O(1) 是有界 delay 下的承諾**(delay ≳ 一圈 → 重摸 delay/(tick×SLOTS) 次);階層輪用 L 層把重摸壓回常數上限。「你的 wheel 什麼時候退化」答這句。

## 卡 5:pow2 mask trick 的適用判準(wheel 用、aggregator 不用)

- wheel 用的理由:SLOTS 是**實作者內部常數**(零合約成本)+ tick=1 下 modulo 是 tick 路徑唯一算術 + const 2 冪編譯器自動強度削減(`% 256` → `& 255`)。
- aggregator 不用的理由:num_windows 是**呼叫方的保留量合約**(動它 = 動「太舊」邊界語意)+ `ts / window_ms` 真除法躲不掉(runtime 值)+ 每筆一次的冷路徑。
- 判準一句話:**除數是不是編譯期已知的 2 冪常數**;部署條件 = wrapping counter + 路徑真熱。
- kernel 深層:階層輪各層取 2 冪 = 時間拆成 base-2^k 位數,每層 index = expiry 的 bit-field 抽取,cascade 邊界對齊 bit 進位——不只省除法,是整個結構把時間當二進位位數切。
- 面試:「認出最佳化」與「判斷值不值得部署」是兩塊肌肉,考的是第二塊;說出「我知道這招、為什麼這裡不用」比用了更加分。

## 卡 6:aggregator 的 epoch = generation tag(本週同脊椎第四次現身)

- ring 桶跨週期重用 → 印章驗身:讀 = epoch 等值否則 None;寫 = 驗印失敗**先重置再入住**;`u64::MAX` = 無主哨兵。同族:e2 slot generation、timer lazy-delete 墓碑、CLOCK 掃針。
- lazy 的完整合約:**「清」= 保證不可觀察,不是抹掉**。三道防線:retention 減法(`latest − e ≥ N`)/ epoch 等值 / 下任入住重置。幾何保證:`e + N ≡ e (mod N)` → 「恰好掉出視野」與「新 epoch 要住它的桶」永遠同時發生。
- **lazy 的合法邊界**:殘留物①不持有資源(POD、無 Drop)②不可被觀察——兩條有一破就回 eager(桶裡是 String = 真 leak;e2 的 fd 必須即刻 close;timer 墓碑 = 有界延遲持有,可接受)。
- 時鐘語意:**data-clock**(錨 = 見過的最大 ts),不是牆鐘、不是 LRU/TTL——window 被更新得再勤也救不了落後;feed 停 = 時鐘停 = 什麼都不過期 → silence detection 是另一個機制(卡1 clarify Q5 存在的理由)。意圖 = 丟落後者;機制 = data-clock 滑動視窗;**邊界行為由機制決定,不是意圖決定**。
- 戰報:「綠 ≠ 沒洞」又一集——提供的 5 測沒查**同餘撞桶**(record e0 → 跳 e9 → 查 e8,8%4=0 躺著 epoch 0 殘料);自寫紅測 `stats(800)==None` 先紅,補 `bucket.epoch != e` 驗印轉綠。順手抓:`Bucket::empty()` 的 min/max 必須是 `i64::MAX/MIN` 哨兵,0 初始會污染全負數 window 的 max。

## 卡 7:std accept 叫不醒 + ext trait 機制(TCP 默寫場邊)

- **每種阻塞等待都需要一條設計好的喚醒通道**(五睡法延伸):condvar → notify(進鎖關縫)|epoll_wait → eventfd 門鈴|阻塞 accept → **std 沒給通道**:self-connect 戲法 / `set_nonblocking` 輪詢 / 改 event loop|tokio → `select!` 讓 accept future 跟 shutdown channel 賽跑(async 模型真紅利,d 題可用)。
- 骨架不做 graceful = 合法 scope:daemon 語意,process exit 收 thread;阻塞在 syscall 裡 = kernel park,零 CPU。面試 clarify 先問「shutdown 要 graceful 還是 process-exit 就好」,一句省十分鐘。
- `incoming()` = `accept()` 的迭代器皮(`next()` 內部就是 accept,丟 addr、永不 None);tokio 1.x 沒有 incoming,`loop { accept().await }` 是慣用語。
- `AsyncReadExt/WriteExt` = **ext trait + blanket impl** 模式:核心 trait 只有 poll_* 給實作者,Ext 全是預設實作方法,`impl<R: AsyncRead + ?Sized> AsyncReadExt for R {}` 點亮全世界。肌肉句:「`.read()`/`.write_all()` 不見了 = 忘了 use Ext trait」(std 的 `use std::io::{Read, Write}` 同理)。
- TCP 默寫 6 輪帳與五條傷疤:見 `scratch/tcp_skelton2.rs` 檔頭(7/26 d-std 前 5m 重默驗收)。meta:錯誤模式 = 每輪即興重組,處方 = 一個形狀零變體。
