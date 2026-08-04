# 8/5 Taper Kit——大複習包(填空版,無純默寫)

> **用法**:8/5 taper 日的唯一複習材料(互動版:`html_p/taper-0805.html`)。
> 鐵律照舊:**不碰新題、記洞不修洞、不開實作碼**。翻到不穩的,寫進 §E 檢查表帶進場,
> 面試時多問一句 clarify 就能繞。
> **填空規則(8/4 定):先想後翻,答案全部在 §B′(隔了一整節,不會瞄到);
> 名字洞 = 形狀對 + 回收句 = PASS,零扣分。**
>
> **動線建議**:白天公司(打字場)= §A 翻閱 30m + §B 填空 20m;
> 晚上(出聲場)= §C 口說一組 3 遍 + §D 掃 60 秒/塊;睡前 §E 過一遍。
> 8/6 晨間動線照 `scratch/recall_checklist.md` §0,材料換本檔 §E。

---

## §A 認題定界輪(a–o + 認題卡 + 四主題家族)

卡片對應肌肉:SP=signal_pipeline|FP=c|FR=e2|TA=f|TQ=h|HW-L=l|HW-M=m|AG-R=route_planner|AG-T=aggregation_tree。

### a–h(+e2):九題型

| 題 | 認題訊號 | 一句定界 | 首要陷阱 |
|---|---|---|---|
| a ring_drop_oldest | "continuous stream" / "most recent N" / sensor | 固定容量 ring + drop-oldest;canonical=`head+len` 索引算術,**不是 VecDeque** | 滿了=寫在 head 再推 head(淘汰+收貨同一步);`dropped` 計數別忘 ++ |
| b pool_shutdown | "concurrently" / "health checks" / "no external libraries" | 固定 worker pool,重點是 shutdown 合約(收到的必跑完、二次呼叫安全) | **兩條件句**(§C-5);執行 job 前一定放鎖;drain-then-exit |
| c frame_parser | "byte stream" / "protocol" / "frames" | 增量 framer:緩衝+迴圈切完整幀;`len==0`=heartbeat 同路徑掉出 | 第一問永遠是「len 含不含 header」;cursor 攤銷 O(n),per-frame drain 是 O(n²) |
| d tokio_frame_server | tokio 可用 / "many devices" / idle timeout | task-per-connection:accept loop + spawn,每 task 重用 c 的 framer | `timeout(idle, read)` 就是 idle timer——**任何 bytes 含 heartbeat 都重置** |
| e event_registry | "event id" / "handlers" | dispatch 表:id→有序 handlers,跑完投票 Keep/Remove | `retain_mut` 一趟跑+刪;dense 小 id→`Vec` 直索引,sparse→HashMap(第一問) |
| e2 fd_registry ⭐JD | fd recycling / stale events / u64 token | **generational slot map**:token=`(gen<<32)\|fd`,slot 記當前 gen,對不上=死連線→None | `wrapping_add` 防 gen 繞回;JD 句見 §C-6 |
| f telemetry_aggregator | "can't store them all" / "aggregate" / "windows" | 固定 W 桶 ring:記憶體 O(windows),**與樣本數無關**(這句就是整個設計) | slot 要**另存自己的 window_num** 防上一圈殘影;空窗回 None 不回 0(min/max 毒藥) |
| g bounded_channel | "producers block when full" | bounded MPSC:`Mutex<VecDeque>` + **兩顆** condvar(not_full/not_empty) | **掉線的一方必須 notify 對面**,否則永眠;predicate loop 不是 if |
| h timer_queue | "periodic" / "interval" / "what runs next" | min-heap:`BinaryHeap<Reverse<(deadline,id,interval)>>`;peek=park 目標 | reschedule 從**舊 deadline** 起算(now 起算會飄移);大 N 逃生門=hashed timer wheel |

### i–o(R2 sim 系)

| 題 | 一句定界 | 首要陷阱 |
|---|---|---|
| i DMA dispatcher | 三表 free/owner/reqs;完成判定=**per-request `done==blocks_total`** | engine 閒 ≠ request 完(R1 洞本尊);cancel=惰性收尾(硬體無 preemption);FIFO 餓小單→RR |
| j ISR pipeline | ISR=劫持一顆核:只搬+計數+叫醒,O(1);滿→drop-newest+計數(有帳) | shutdown=**先 flag 後 wake**;sticky flag 合併、只夠醒一次;wake=hint 睡前必 re-check |
| k per-core fan-in | N 條 SPSC → 一個 aggregator;producer 永不阻塞(try_push 失敗→dropped++) | **全零掃描才准睡**(Waker flag 蓋掃描→睡的縫);per-ring budget 防熱核餓冷核 |
| l MMIO cmdq | 「這是一條 SPSC ring,只是消費者是矽」 | 順序即法律:fill descriptor → **barrier()** → doorbell;滿判定用自由跑計數器 `tail-head==cap`;completion 亂序靠 tag 路由 |
| m watchdog | sim i 三表 + **第三種 state:時間**(per-engine deadline) | 「slow≠dead,無 fence 只能 bound risk」;超時 engine **隔離不回池**;殭屍 done=生存證明→復活但帳不碰;3-strikes 後 fail 整單+清帳 |
| n scheduler | 兩層閘:**DAG 入場閘**(indegree)→ **priority 閘**(heap+seq 破平手) | seq **到站蓋章**不是入場蓋章;`completed` 集合必備(deps 過去式);表只進不出=刺 |
| o boot planner | Kahn 一趟三答案:波次(frontier 整批)/makespan(DAG 最長路徑)/環回報 | 「等全部」=AND=indeg 歸零才入隊;環要報**實際環**不是 bool;找環用邊的存在性不能用執行痕跡 |

### AG 兩卡(algo 系)

| 卡 | 認題訊號 | 定界 | 陷阱 |
|---|---|---|---|
| AG-R widest path | 題面出現 **"minimum along the path"** | 瓶頸最寬路:Dijkstra 兩處變形——鬆弛 `min(bott[u],w)`、heap 換 **max**-heap(no Reverse) | 不是最短路;no decrease-key→lazy deletion;max-flow/ECMP 宣告 out of scope |
| AG-T tree repair | collector 死了、子樹 rehome、fan-in 上限 F | 剝掉樹皮=**容量受限指派**(bin packing 貪心殼),不是樹 DP | **必問「最小化什麼」**(spec 故意沒寫);禁區=孤兒自己的子樹(否則成環);容量帳算**連線數**不算載重;死者的 parent 因它離開多一空位 |

### async 兩皮定界(8/5 指定項)

- **tokio 三句(d 題)**:_"Task-per-connection with tokio: an accept loop, `tokio::spawn` per connection, each task reuses the framer on its own buffer. Idle handling: wrap the read in `tokio::time::timeout` — any bytes, including heartbeats, reset it by construction. Tasks are KBs, not MBs — that's why this scales where thread-per-connection dies."_
- **純 std block_on/poll 合約兩句**:_"async/await is pure composition — the compiler-generated state machine only forwards poll; exactly two places touch the Waker: the executor that creates it and the leaf future that stashes it."_ / _"A wake is only a hint that re-polling is worth it — poll is the single source of truth; it's the condvar predicate-wait contract, one level up."_

### 四主題家族判準(timer wheel / time slot / conflation / aggregation window)

**判準句:同 key 的舊資料被結構性覆蓋/合併、且這是刻意語意——才算 conflation 家族。**

| 結構 | 算 conflation 嗎 | 為什麼 |
|---|---|---|
| timer wheel(time slot) | ✗ | slot 是「**桶**」:按到期時間索引,timer 全收不丟、一個都要 fire。解的是**按時間查找**,不是過載保最新。形狀像(固定格+索引),語意不同 |
| aggregation window | 半個(表親) | per-key 摺疊 ✓,但摺疊函數是**可結合 merge**(sum/max=保留全部事件的統計);觸發也不同:關窗即吐 vs consumer 拉 |
| conflation slot | ✓ 本尊 | merge 退化成 `f(old,new)=new` 的特例;能用的三條件:consumer 只要現在值 + payload 是絕對快照 + key 基數有界 |
| sticky wake token(sim j) | ✓ 退化版 | 單 key、payload=unit:N 次 wake 摺成一個 token;epoll LT readiness、dirty-rect、LWW register 同族 |

conflation 認題一句(出聲):_"Do you need **every** record, or only the **latest** per key? That changes the loss model: with a ring, loss is accidental and I count it; with a conflation slot, loss is by design."_

---

## §B 填空暖手(18 題,決策點填空;答案在 §B′,先想後翻)

### 圖論皮

**B1 Kahn(sim o)**:入隊條件 `indegree == ①`;「等全部」= ② 閘(AND/OR);環偵測:收尾時 `processed ③ n` ⇒ 有環;波次 = 同一輪 frontier ④ 彈出。

**B2 DAG 最長路徑**:沿 topo 序:`dist[v] = max(dist[v], dist[u] + ①)`;一般圖不能做的理由(面試句):「環逼子問題②,DAG 讓它免記」。

**B3 AG-R widest path**:鬆弛式 `cand = ①(bott[u], w)`;heap 用 ②-heap;認題訊號:題面出現 "③ along the path"。

**B4 AG-T**:本體是「①」問題(不是樹 DP);樹只貢獻三樣:孤兒名單、②、空位表;容量帳算 ③ 不算載重。

### Condvar 系

**B5 pool 兩條件(英文,招牌)**:A worker **exits** only when `① && ②`; it **sleeps** only when `③ && ④`.

**B6 8/4 admission gate**:等待條件 = `running >= N ① tenant_cnt >= M`(AND/OR?);處方:先寫 ② 條件,再 ③ 取反。

**B7 shutdown 協定**:先 ① 後 ②;被喚醒第一件事查 ③;bounded_channel 加一條:最後一個 Sender drop 時必須 ④,否則 consumer 永眠。

**B8 conflation 兩紀律**:recv 的「pop + 讀值 + 清旗標」必須 ①,拆開就是 ②(多執行緒隨機測抓不到);通知可以 ③、不可以 ④。

### Lock-free 序

**B9 spsc 四格表**:producer 寫槽後存 tail 用 ①;consumer 讀 tail 用 ②;consumer 讀完槽存 head 用 ③;producer 讀 head 用 ④。

**B10 對稱律**:producer 先 ① 後 ②;consumer 先 ③ 後 ④。(提示:值/訊號)

**B11 tokio broadcast 內臟(8/4 課)**:每個 receiver 自帶 u64 游標,靠 slot 上蓋的 ① 偵測自己 Lagged;值的釋放時機 = min(②, ③);「sender 永不阻塞」的代價 = ④。

### async 兩皮

**B12 block_on**:`Poll::Pending` 臂做 ①(絕不准 ② future——進度歸零);每輪 poll 的永遠是 ③;Waker 的來源:`impl ④ for T`(self 形狀=⑤)再 `Waker::from(⑥)`。

**B13 tokio 三處**:async read/write 方法住在 ① 上(唯一死記);`accept()` 回 `Result<(②, ③)>`,配 ④ 迴圈不配 `while let Some`;`TcpListener::bind(...)` 後面要接 ⑤。

### 協定 / 時間結構

**B14 length-prefix 三動作**:① 防溢位 → ② 防越界 → ③ 解 bytes;型別口訣:wire 上是什麼就用什麼解(④),再 `as` 到 ⑤。

**B15 timer_queue**:heap 元素 `Reverse<(①, ②, ③)>`(tuple 順序免費送 tie-break);reschedule 從 ④ 起算(否則 ⑤);大 N 逃生門:⑥,攤銷 O(1),拿 ⑦ 換 throughput。

**B16 家族判準**:timer wheel 的 slot 是「①」(全收不丟);conflation = merge 退化成 `f(old,new)=②`;aggregation window 的摺疊函數要求 ③;TA 的 slot 要另存 ④ 防上一圈殘影;空窗回 ⑤ 不回 0。

**B17 fd_registry**:token = `(① << 32) | ②`;stale 判定:`slot 的 gen ③ token 的 gen → None`;gen 遞增用 ④ 防繞回。

### 單線 DS

**B18 DSU + LRU**:find 路徑壓縮一行 `parent[x] = ①`;union by ②;LRU unlink 之後要 ③(兩側、雙向);淘汰+插入是 ④ 筆交易。

---

## §C 英文口說指定句(晚上出聲場;每晚一組、每句 3 遍,第 3 遍不看稿)

### 第一組:開場與定界

1. _"Let me **restate** to make sure I've got it: we need a ___ that ___. Before I start, let me make sure I understand the constraints."_
2. _"I'll ask a few questions first, then **walk you through my plan** before writing code."_
3. _"When you say ___, do you mean **A or B**?"_(把模糊詞逼成二選一,最好用的一句)
4. _"Three **lifecycle** questions: how long does this run, how do we stop it cleanly, and who owns the in-flight data at shutdown?"_

### 第二組:招牌句(講出來就是分)

5. _"A worker **exits** only when shutdown is set **and** the queue is empty; it **sleeps** only when the queue is empty **and** we're not shutting down. Those two predicates are the whole problem."_(b 題)
6. _"An **O(1) generational slot map** beats an O(n) scan on every event, and it **rejects stale tokens for free** — the generation check is one compare."_(e2,JD 句,背到逐字)
7. _"Is a wake **latched** if nobody's sleeping? Do wakes **coalesce**? Any **spurious** wakeups?"_(wake 三問)
8. _"Slow and dead are **indistinguishable** — without a fence or reset I can only **bound** the risk, not eliminate it."_(sim m 最高分句)

### 第三組:救場與裁決(8/4 新句入列)

9. _"It's the trait in `std::task` where self is `Arc<Self>` — I forget the exact name, **mind if I check?**"_(名字洞回收句範本:形狀先講、名字再要)
10. _"The stated assumption is ___ — my design doesn't work that way. Want me to size **the stated version, mine, or both**?"_(Part B 基準不一致,8/4 洞的解藥)
11. _"**Timeout to kick is fine; timeout to retry is not** — TCP already owns retransmission, and a half-done `write_all` would tear the frame."_(8/4 裁決)
12. _"I'll keep this **inline for time**; extracting `on_done` would be my first cleanup."_ / _"I'm running short on time, so let me describe **what's left and where the holes are**."_(收尾兩句)

加映(隨手可插):_"I'll let the compiler remind me of the exact import."_ ・ _"At huge N I'd move to a hashed timer wheel — amortized O(1), trading precision for throughput."_ ・ _"I've been bitten by ___, so I ___."_(傷疤句型=finding-your-own-bug 的口述版)

---

## §D 裁決句庫(掃讀 60 秒/塊)

**shutdown / wake 合約**:先 flag 後 wake(=先值後訊號的同一條定律)|醒來=hint 不是保證,睡前必 re-check|sticky flag 是合併的,只夠醒一次|對稱律:producer 先值後訊號、consumer 先銷訊號後取值|通知可以多、不可以少(spurious 對 conflation 冪等;lost wakeup=資料永久遺失)。

**超載三路**:只有三路——丟(counted drop)/摺(fold,M 筆變 1)/批;三選一的裁決權在三個門檻問題:**sink 成本結構/資料語意/SLO**,spec 沉默就問,沒得問就具名假設|update 是 replace 就有損、是 merge/fold 就無損|誠實 policy=drop-newest+dropped 計數|無界暫存=把有界管線悄悄改無界:過載走向 OOM(隱形),不是受控丟棄(有帳)。

**狀態設計**:與其寫 code 調解兩份狀態的漂移,不如選一個**讓漂移不可表示**的狀態設計|tuple>3 欄→具名 struct(消滅的是「整類」bug)|state ≤3 件 flat 合法、≥4 件 struct 先立;抽 fn 是 refactor 不是前置投資|表要「只進」也要「有出」:完成/放棄兩個刪除點(sim m tries、n dependents、8/4 tenant_running×2=同根刺四現)|通知越貧瘠,state 表越重要。

**時間與 timeout**:timeout=數倍 p99(太短誤殺、太長拖住);**數字不重要,理由那句話才是分**|pop 出來第一件事永遠是「還在飛嗎」(stale 鬧鐘)|殭屍=生存證明:隔離到 proof-of-life,帳一根指頭不碰|reschedule 從舊 deadline 起算|「fail 整單」的另一半是清帳。

**測試**:黑箱三律:觀察輸出面、poll with deadline、斷言不變量;永遠不要斷言「它何時睡」|沒有比較的賦值=「誰後到誰贏」紅旗——故意把錯的候選人排最後餵它|my tests verify ordering; utilization needs a different probe|樣本守恆:read 出的每筆=pushed+dropped,帳永遠平。

**量級(五行頭配套)**:單位跟著每一步(裸數字=病根)|掉零是主敵,荒謬檢查抓不到掉零——逐步帶單位的帳本才是主保險|兩位有效數字,免費|kernel per-socket buffer 是事實不是假設(收/送各一份,全忙翻倍;`ss -m` 驗證)|Little's law:pool = rate × 佔用|async 省的是 thread 成本,不是併發數。

**8/4 新增**:異質 predicate → notify_all(有人卡 N、有人卡 M,notify_one 會叫錯人)|AND/OR 閘處方:先寫**進場**條件,De Morgan 取反成等待條件,不直接寫等待|讀 predicate 禁 `entry().or_default()`(看一眼就插 0 條目)——讀用 `get().copied().unwrap_or(0)`,entry 留給入場後 +1|broadcast:per-slot 絕對序號=Vyukov 同族;釋放=min(最慢讀者 rem→0, 被覆寫);subscribe() 游標生在當前 tail=join 語意的實作位置。

---

## §E 帶進場檢查表(8/6 晨讀本體)

**開場動線(45m 的前 10m)**:① restate 一句 ② clarify:名詞圈選→lifecycle 三問(跑多久/怎麼停/停時在途資料算誰的)→失敗後路徑三問(偵測到做什麼/重試幾次誰決定/放棄時通知誰清哪些帳)→單位掃(bytes or blocks?)→「當 X 你是指 A 還是 B?」 ③ 宣告假設:"I'll assume ___ unless you tell me otherwise." ④ **state 表**(紙上,≥4 件→struct 謄寫) ⑤ 骨架+todo!()。

**英文三對策(8/2 定,8/4 實測仍要催)**:裁決**當場**抄紙(≤5 字/條,抄完複誦)|規則 read back("So timeout 1 and 2 → redispatch, only the 3rd → error, correct?")|多問訊息**編號逐條回**,跳過就明說 deferred。

**量級五行頭**:Given(重述全部參數;數字用一個劃一個,有剩=你在答別題;**跟你設計矛盾的假設在這行現形**)→ Chain(每步帶單位)→ Cross(兩路殊途必同歸)→ Sanity(荒謬檢查一行,沒有不收筆)→ Verdict(policy+何時啟動+代價)。

**骨架抽查殘項(8/5 豁免項,填空版見 §B)**:B5 pool 兩條件、B12 block_on(③assisted 重驗)、B13 tokio 三處(Ext 二連)、B14 length-prefix。

**心理帳**:首打超時是預期結果,不是能力判決|拿到陌生 spec 立刻做具體小事(clarify 五問/state 表/30 秒定界)——定界句的本質=把面試官的散文翻成自己的題面(LC 翻譯)|7 錯出現在 8/3 半夜,就不會出現在 8/6 09:15。

---

## §F 修 code 段(8/4 加訂:taper 也要動手,量控制在 ~20m)

檔案:`rehearsals/examples/bughunt_0805.rs`(LSP 全套、gate 綠、跑起來會炸)。
兩題各埋 **2 個 bug**,全在邏輯層。流程:
① **先只用眼睛讀**,把 4 個 bug 圈出來(這就是面試的 review 肌肉)
② 改碼 → `cargo run -p rehearsals --example bughunt_0805` → 直到印出「全綠」
③ 圈完/修完才准開下面 §F′ 對答案。

---

## §B′ 填空答案(對完記 ✓/⚠/✗;✗ 寫進 §E 帶進場)

**B1** ①0 ②AND ③<(processed < n) ④整批。
**B2** ①w(邊權) ②記歷史(有環→NP-hard;DAG 免記=P)。
**B3** ①min ②max(no Reverse) ③"minimum"。
**B4** ①容量受限指派(bin packing 貪心殼) ②禁區(孤兒自己的子樹,防成環) ③連線數。
**B5** ①shutdown ②queue is empty ③queue is empty ④not shutdown。
**B6** ①OR ②進場(running<N && tenant<M) ③De Morgan。
**B7** ①flag ②notify_all ③shutdown 旗 ④notify(叫醒睡著的 consumer)。
**B8** ①同一個 critical section ②lost update ③多(spurious 冪等) ④少(lost wakeup)。
**B9** ①Release ②Acquire ③Release ④Acquire。
**B10** ①值(push/寫槽) ②訊號(存 tail/wake) ③銷訊號(推 head/清旗標) ④取值。
**B11** ①絕對序號(slot.pos==next 驗證) ②最慢讀者讀完(rem→0 當場 drop) ③被新訊息覆寫 ④感受不到 consumer 慢/Lagged 才發現(backpressure 要另外走)。
**B12** ①park ②drop ③同一顆 pinned future(`as_mut()`) ④Wake ⑤`Arc<Self>` ⑥`Arc<T>`。
**B13** ①Ext trait(`AsyncReadExt`/`AsyncWriteExt`) ②TcpStream ③SocketAddr ④loop ⑤`.await?`。
**B14** ①`checked_add(4)?` ②`buf.get(ptr..end)?` ③`u32::from_be_bytes(... .try_into().ok()?)` ④u32(wire 型別) ⑤usize(host 型別)。
**B15** ①deadline ②id ③interval ④舊 deadline ⑤每次 firing 飄移 ⑥hashed timer wheel ⑦精度。
**B16** ①桶(按到期時間索引) ②new ③可結合(associative merge) ④自己的 window_num(epoch) ⑤None。
**B17** ①generation ②fd(slot index) ③!=(mismatch) ④`wrapping_add`。
**B18** ①find(parent[x])(遞迴掛根) ②rank/size ③兩側鄰居雙向重接 ④同一(寫完立刻 get 新 key 自驗)。

---

## §F′ bughunt 答案鍵(圈完/修完才開)

**練習 1(Ring)**:
- Bug A:滿分支蓋掉的是 `tail`(最新)不是最舊——那是 drop-newest 冒充 drop-oldest。
  正解=「寫在 head、再推 head」:`self.buf[self.head] = v; self.head = (self.head + 1) % cap;`(淘汰+收貨同一步,§A a 列原話)。
- Bug B:滿分支 `dropped` 沒 `+= 1`(7/31 傷疤原話「dropped 整條忘 ++」)。

**練習 2(Kahn)**:
- Bug C:**OR 閘**——`next.push(v)` 跟在每次 `indeg[v] -= 1` 後面,每砍一條邊就入隊;「等全部」=AND=**歸零才入隊**:`if indeg[v] == 0 { next.push(v) }`(sim o v1 主洞本尊)。
- Bug D:**環被吞掉**——迴圈自然結束就 `Ok`,少了 `processed` 計數與收尾檢查:`if processed < n { return Err(CycleError) }`(環偵測=處理數<n;要報環時「用邊的存在性,不能用執行痕跡」)。
