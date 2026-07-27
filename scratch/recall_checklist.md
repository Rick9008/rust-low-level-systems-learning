# 7/28 晨讀本——認題→開場檢查表(完整版)

> 8:00 起床版。本檔 = 明早唯一要讀的東西。餵給 chat 做 HTML stepper 時:
> 「晨間動線」做成 timeline、「暖手默寫」做成先默後翻的 stepper、「九題型」做成卡片。

---

## 0. 晨間動線(8:00 起床 → 8:45 上場)

| 時刻 | 動作 |
|---|---|
| 8:00 | 起床、水、洗臉。**不看手機訊息**。 |
| 8:08 | 開電腦:CoderPad 連結、Meet、耳機測一次(§7 裝備清單)。 |
| 8:12 | 讀 §1 暖手默寫(**動手寫,12 分鐘**)——喚醒的是手,不是眼睛。 |
| 8:24 | 讀 §3 九題型速查 + §4 金句(出聲唸粗體英文句,10 分鐘)。 |
| 8:34 | 讀 §5 流程鐵律 + §6 漏問表(3 分鐘)。 |
| 8:37 | 廁所、水杯裝滿、深呼吸。想一件跟面試無關的開心事。 |
| 8:43 | 坐定。你 10 場計時彩排、最近一場 30 分鐘零洞提前收工——**你是有牌的人**。 |

**⏰ 7:45 起床才解鎖的加碼區**:spsc use 塊+簽名全默一遍(7/26 首編 0 錯的那套)+ pool 兩條件。8:00 起床**不要**碰這區——寧可少默一格,不要帶著趕的心率上場。

---

## 1. 暖手默寫(12 分鐘,白紙/空檔案,寫完才看 §2 對答案)

### 1a. length-prefix 三行(⚠ 昨天唯一 ✗:usize::from_be_bytes 真洞)

默:從 `buf` 的 `ptr` 位置安全讀出 4-byte BE 長度。三個動作:防溢位加法 → 防越界切片 → 解 bytes。

### 1b. TCP accept-loop 六行(⚠ 昨天 4 手滑:外層多套 loop / 少 &mut / 少 & / 忘 trait import)

默:std echo server 的骨架,從 `for stream in listener.incoming()` 開始。

### 1c. bounded_channel Sender Drop 六行(⚠ 昨天方向默反:==1 的人跑掉了)

默:多 Sender 的 Drop——誰關燈?怎麼關?

---

## 2. 對答案(默完才看;⚠ = 你踩過的原話)

### 2a. length-prefix

```rust
let end = ptr.checked_add(4)?;                          // Option,fn 回 Option 最省
let slice = buf.get(ptr..end)?;                         // 越界 → None,不 panic
let len = u32::from_be_bytes(slice.try_into().ok()?) as usize;
```

⚠ **`usize::from_be_bytes` 是你 7/27 的真洞**:陣列長度由**目標型別**決定——usize 在 64-bit 上要 `[u8; 8]`,你的 slice 只有 4 → `try_into` **每次 runtime Err**,編譯器不會救你。口訣:**wire 上是什麼型別就用什麼解(u32),再 `as` 到 host 型別**。

### 2b. TCP 六行

```rust
use std::io::{Read, Write};                              // ⚠ 頭號手滑:忘 import trait
fn serve(listener: TcpListener) {
    for stream in listener.incoming() {                  // ⚠ 不要外面再套 loop——incoming 本身無限
        let Ok(mut stream) = stream else { continue };   // ⚠ Err 是 continue 不是 break:accept 失敗多半暫時,server 不該為它死
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match stream.read(&mut buf) {            // ⚠ &mut buf(昨天寫成 &buf)
                    Ok(0) => break,                      // EOF = 正常收尾,不是錯誤
                    Ok(n) => { if stream.write_all(&buf[..n]).is_err() { break; } }  // ⚠ &buf[..n] 的 &
                    Err(_) => break,
                }
            }
        });
    }
}
```

測試用 `"127.0.0.1:0"`(要臨時 port)+ `local_addr()`——⚠ d#1 寫死 port 吃過 `AddrInUse`。

### 2c. Sender Drop

```rust
impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if self.shared.sender_cnt.fetch_sub(1, Ordering::Release) == 1 {
            let _g = self.shared.deque.lock().unwrap();  // 拿鎖放鎖:關掉「檢查完、還沒睡」的縫
            self.shared.wait_not_empty.notify_all();
        }
    }
}
```

⚠ **昨天默反**:`== 1` 的人是**最後一個 = 要關燈的人**,你寫成 `== 1 { return }` 讓他直接走人。記法:**fetch_sub 回傳 1 → 我是末代 → 我負責通知**。用**回傳值**判斷,不要 sub 完再 load(兩次讀之間有窗)。
⚠ Ordering 用 **Release**(遺言:我做過的一切發佈在退出之前);Clone 側其實 Relaxed 就夠(能 clone 代表 count≥1,所有權論證)——但被追問才講,預設寫 Release 不虧。

---

## 2.5 String / &str / HashMap 教學節(昨天 toposort 挖到的洞)

**心智模型一句話**:`String` 是**擁有者**(管 heap);`&str` 是**視圖**——一根 `(ptr, len)` 胖指標,**Copy**,免費抄。

**轉換速查**:

| 方向 | 寫法 | 什麼時候用 |
|---|---|---|
| `String` → `&str` | `s.as_str()` / `&s` / `&s[..]` | 借視圖,零成本 |
| `&str` → `String` | `.to_string()` / `.to_owned()` | 要**存起來**(進長壽容器)才付這筆 heap 錢 |
| `&String` → `&str` | **函式參數位自動**(deref coercion) | 但**泛型查表位常常不推**→ 手動 `.as_str()` |

**HashMap 的 key 怎麼選(昨天整晚在打的仗)**:

1. **`HashMap<&str, V>`——視圖當 key**:map 只是**指向別人資料的索引**時用(你的 toposort:資料本體住在 `tests`,map 全是薄參照)。代價:map 活不過資料來源(lifetime 綁住)。
2. **`HashMap<String, V>`——擁有 key**:map 要**長壽/跨函式**時用。查表免費:`map.get("k")` 直接用 `&str` 查(`Borrow<str>` 幫你),**不用** to_string;但 `entry()` 要吃 owned key → `entry(k.to_string())`——這就是「查便宜、插要付錢」的不對稱。
3. ⚠ **你昨天的兩滑**:
   - `HashSet<&str>` 的 `contains` 要 `dep.as_str()`——泛型位不自動 coerce。
   - **iter 借出的參照別存過夜**:`for (name, _) in &map` 的 `name` 是 `&&str`,押著整張 map 的借用 → 之後 `get_mut` 全被擋。**在邊界 `*name` 抄出來**(`&str` 是 Copy),借用鏈當場斷——pattern 寫法 `for (&name, v) in &map` 更順。
4. **場上預設**:輸入是 `&[(String, ...)]` 之類 → 索引用 `&str` 視圖(方案 1);要回傳/存活 → `String`(方案 2)。想不清楚就全 `String` + `.clone()`——**正確 > 優雅**,clone 的錢 45 分鐘內不會破產。

---

## 3. 九題型速查(30 秒開場句 → 選型 → trade-off → 我的傷疤)

**上場資訊**:8:45–9:30 CoderPad(Rust 1.92 / edition 2024,tokio 有、無 libc/mio、無 LSP)。
**時間預算**:0-3 讀題|3-5 clarify|5-10 skeleton|10-35 core(**邊寫邊講**)|35-40 boundary(**自己站起來**)|40-45 trade-off。

### 情報加權(7/27 coffee chat,software head 親口)

- **「tests 決定執行順序」= toposort**(昨晚已補練到綠):兩張 HashMap(dep→誰等它 / in-degree)+ VecDeque;`entry().or_default()` 讓零依賴者也進表(⚠ 昨天第一版漏了);Kahn O(V+E);**環 = 輸出長度 < 節點數**;環內容 = 從任一 stuck 節點沿 stuck dep 走到撞鬼;決定性輸出 → 佇列換 BinaryHeap O(V log V);同波歸零 = **可平行一批** → 接 thread pool(跟 b 題握手);⚠ **幽靈依賴要 skip**(你 dry-run 抓的)+ **checker 先驗長度再驗每條邊**(斷言合約不斷言實例)。
- **「DS 改 concurrent」= 升級階梯**:粗 Mutex(_"correct by construction, fine at this scale"_)→ contention 故事 → RwLock/shard → 熱欄位原子化 → lock-free(講價格)。**判決不看讀寫比,看 read path 動不動結構**:LRU 的 get 會 promote → RwLock 陷阱(→ shard 整台 / CLOCK 一 bit 換掉 list);config registry 的 read 是真 read → RwLock 正解(→ 再上去 snapshot publication / rcu)。

### 九題

| 題 | 定界句(第一句) | 選型 | trade-off | ⚠ 我的傷疤 |
|---|---|---|---|---|
| a ring | fixed-capacity ring — policy first: drop-oldest / newest / block? | Vec+mask, head/tail | 下界=無界 Vec(OOM);上界=lock-free。**drop-oldest 逼 producer 動 head → 退化 SPMC → CAS:policy decides the synchronization structure** | pop 判空 head==tail 二義;drop_cnt 忘 ++;head/tail 座標系開寫先釘一句 |
| b pool | **exit = shutdown ∧ empty;sleep = empty ∧ ¬shutdown**(兩條件是整題) | Mutex<VecDeque>+Condvar | 下界=spawn-per-job;上界=work-stealing | store+notify 不拿鎖=lost-wakeup;鎖圈 job 全池串行(0.40→0.10s);盲 unwrap pop 毒鎖 |
| c framer | TCP is a byte stream with no boundaries → internal buffer + loop cut | Vec buffer+drain / ptr+compact | 攤銷拷貝 vs 原地 compact;max-len 是安全閥 | **len 含不含 header 是第一問**;may_compact drain 後 ptr=0 |
| d server | thread/task per connection + framer 重用 | tokio select / timeout | **idle timer 答「peer 活著嗎」;frame-age 答「frame 拖太久嗎」——不同問題不同 timer**(timeout 每圈重上發條 vs timeout_at 一次定死) | idle_timeout 整條蒸發(clarify 沒問的需求恰是掉的);echo 掉 wire format;`:0` 埠 |
| e registry | dispatch table: handlers on ids, run in order, vote Keep/Remove | HashMap+`Box<dyn FnMut>`+retain_mut | dense→Vec / sparse→HashMap 一句定存儲 | per-id 漏讀(**參數沒用到=漏讀警報**);re-entrancy 要問 |
| e2 fd_reg | **generational slot map**: token=(gen<<32)\|idx, mismatch = 死人的信 | slot Vec+free list | O(1) 勝 O(n) 掃;stale 免費擋;`wrapping_add` 定義 wrap | len 帳逃出 `is_some` 守衛;mask 少 1 bit(`as u32` 截斷天生對);**`gen` 是保留字用 `generation`** |
| f aggr | can't store all → fixed buckets, aggregate on write | 桶陣列+epoch 牌 | **lazy validation 勝 eager 清掃:record 嚴格 O(1)**;代價=每存取多一比較 | 同餘撞桶鬼資料(三現身):**桶不是你的,牌對了才是你的** |
| g channel | bounded MPSC: 兩把 Condvar + 計數器當所有權帳本 | Mutex<VecDeque>+2 Condvar | close 語意:**drain 完才 None**——醒來後佇列自己就是答案,別用旁路變數改判 | recv early-return 蓋過有貨;雙 Drop 見 §2c;沒 join 的斷言不是斷言 |
| h timer | min-heap `Reverse<(deadline, id)>`, park until peek | BinaryHeap+lazy-delete 墓碑 | 下界=thread-per-timer(3000 stacks);上界=wheel(**拿精度換吞吐**) | **reschedule from the old deadline, not now**;**wait_timeout + re-checked predicate**(新 timer 插隊要叫醒睡的人);次鍵是 id 不是 interval |

---

## 4. 口述金句(出聲唸一遍,肌肉在嘴上)

1. 抓自己 bug 三拍:_"Wait — I think I have a bug here."_ → **點名場景** _"this breaks when …"_ → _"Let me fix that before moving on."_ **不道歉、不默改**——finding your own bug = stronger signal。
2. 傷疤紀律:_"I've been bitten by this exact off-by-one before, so let me double-check the boundary."_
3. 轉場(35 分那格,自己說):_"Now let me dry-run the boundaries."_
4. lock-free 精確版:_"Lock-free buys a **worst-case guarantee**, not automatic average speed — a parked waiter can beat a spinning CAS loop. A true SPSC ring wins both: no CAS at all."_
5. 快照語意:_"Readers may see a **stale but consistent** view — stale by one publish, never torn."_
6. 簽名句:**"The policy decides the synchronization structure."**(drop-oldest→SPMC、idle vs frame-age、LRU→CLOCK,三案同一條)
7. 收尾三拍:**價格**(Big-O 每個字母指認)→ **沒走的路 ≥2**(下界暴力解帶數字否決 + 上界升級解)→ **有效範圍**(_"This assumes a trusted peer…"_)。

---

## 5. 流程鐵律

1. 動筆前 30 秒:**clarify 清單對讀需求清單**——d#1 的 idle_timeout 就是這樣蒸發的。
2. 說「綠」之前,終端機要有那行 `test result: ok`。
3. 測試裡每個 spawn 必接 join——**沒 join 的斷言不是斷言**。
4. 邊寫邊講(narrate 是考試本體,不是加分項)。
5. 寫完每個 fn 掃簽名:**每個參數都要有下落**(unused = 漏讀警報)。

---

## 6. 漏問模式表(clarify 前掃一眼)

| 類 | 明天的那一問 |
|---|---|
| SLA | _"What's the latency budget — p50 or p99?"_ |
| 容量立式 | _"How long a burst must we absorb? capacity = rate × duration"_ |
| 併發上限 | _"Is there a cap on concurrent X?"_ |
| 掉不掉 | _"Do all samples matter, or **only the latest**?"_(conflation 一問值一個量級) |
| 答了要複誦 | _"So to confirm: recv drains, then None — noted."_(⚠ g#1:答了還掉) |

---

## 7. 裝備(前晚睡前 + 8:08 各過一次)

☐ CoderPad 連結開得起來 ☐ Meet ☐ 耳機麥克風 ☐ 水 ☐ 紙筆(dry-run 用) ☐ 手機勿擾 ☐ 鬧鐘 8:00

---

## 8. 漏洞卡全集(wrong → right 對照;10 場彩排全部的血;stepper 翻牌區)

> 明早時間不夠就只看每格的 ⚠ 行;code 給 HTML stepper 用。

### a · ring_drop_oldest(7/19,oracle 4 紅)

**⚠ 滿=空二義**——masked index 下 `head == tail` 同時是滿和空:

```rust
// ✗ if self.head == self.tail { return None; }        // 滿的時候也成立 → FIFO 全毀
// ✓ 單調計數器不 mask,len 用減法:
fn len(&self) -> usize { self.tail - self.head }        // 滿 = len == cap;空 = len == 0
// mask 只在「碰 buf」的最後一刻:buf[idx & self.mask]
```

**⚠ drop_cnt 整條沒 ++**——drop-oldest 觸發時忘了記帳:`self.drop_cnt += 1;` 跟 head 前進同一行動作。
**⚠ Part 2 擅改 contract 成阻塞 pop**——clarify 過的合約不准中途自己改,要改先問。

### b · thread_pool(7/20,2 綠 3 紅 + 補課 3 洞)

```rust
// ✗ if shutdown { break; }                             // 見 flag 即退,queue 剩 16 筆不清
// ✓ if shutdown && guard.queue.is_empty() { break; }   // 退出 = 兩條件

// ✗ guard = cvar.wait_while(guard, |s| s.queue.is_empty()).unwrap();
//                                                      // 空佇列 shutdown → 永眠 hang
// ✓ ... |s| s.queue.is_empty() && !s.shutdown ...      // 睡 = 空 ∧ ¬shutdown

// ✗ self.shutdown.store(true, ...); cvar.notify_all(); // 不拿鎖:store 掉進「檢查完、還沒睡」的縫
// ✓ { let _g = jobs.lock().unwrap(); /* 或 store 在鎖內 */ } cvar.notify_all();  // loom 三變體裁決

// ✗ let job = guard.queue.pop_front().unwrap();        // 醒來盲 unwrap → panic 毒鎖連環爆
// ✓ 醒來重查 predicate,pop 到 None 就回去睡/退出

// ✗ let g = jobs.lock().unwrap(); (g.queue.pop())();   // 拿著鎖跑 job → 全池串行 0.40s
// ✓ let job = { ...pop... }; drop(guard); job();       // 放鎖再跑 → 0.10s

// ✗ let _ = handle.join();                             // 吞掉 worker panic,測試假綠
// ✓ handle.join().expect("worker panicked");
```

### c · frame_parser(7/23 一次綠;遺留 may_compact 雙洞 7/24 修)

```rust
// ✗ fn may_compact(&mut self) { if self.ptr > 4096 { self.buf.drain(..self.ptr); } }
//    忘了歸零 → 下一輪用舊 ptr 讀已 drain 的位置 → underflow / 讀歪
// ✓ self.buf.drain(..self.ptr); self.ptr = 0;          // 兩行是連體嬰
```

**⚠ 紅測教訓**:全 0 payload 咬不住這洞(mutation 測過)——測資要用 `[20]` 這種非零 payload。
**⚠ clarify 第一問永遠是**:`len` 含不含 header?(heartbeat 反推:本題 payload only)

### d · tokio_frame_server(7/25,三洞全 review 抓)

```rust
// ✗ 需求單有 idle_timeout,code 裡一個字都沒有(clarify 沒問到的需求恰是掉的需求)
// ✓ let n = timeout(IDLE, stream.read(&mut buf)).await??;     // 逐 byte 續命
//    frame-age 版:timeout_at(deadline, ...)——frame 開始定一次,不重上發條

// ✗ stream.write_all(&frame.payload).await?;           // echo 掉 wire format,裸 payload
// ✓ 先回 header 再回 payload:write_all(&(len as u32).to_be_bytes()) → write_all(payload)

// ✗ TcpListener::bind("127.0.0.1:8080")                // 測試寫死 port → AddrInUse
// ✓ bind("127.0.0.1:0") + listener.local_addr()?
```

### e · event_registry(7/26 快寫,真洞 1)

```rust
// ✗ fn handler_count(&self, id: u32) -> usize { self.total_count }   // id 沒用到!回全域
// ✓ self.handlers.get(&id).map_or(0, |v| v.len())
```

**⚠ 警報器**:簽名裡的參數沒被用到 = 漏讀警報(unused warning 會自己叫,聽它的)。

### e2 · fd_registry(7/21 零紅但 2 洞;7/25 洞① 回鍋)

```rust
// ✗ if let Some(slot) = self.slots.get_mut(idx) {
//       if slot.generation == g { slot.value.take(); }
//   }
//   self.len -= 1;                                     // 逃出守衛:偽 token 也扣帳,len=0 時 usize underflow panic
// ✓ if ... && slot.value.take().is_some() {            // 狀態變更押在「確認移除成功」之後
//       slot.generation = slot.generation.wrapping_add(1);
//       self.len -= 1;
//   }

// ✗ let fd = (token & ((1 << 31) - 1)) as u32;         // 少 1 bit:fd ≥ 2³¹ alias 到低位
// ✓ let fd = token as u32;                             // 截斷天生就是 32-bit mask
//   let generation = (token >> 32) as u32;             // ⚠ `gen` 是 edition 2024 保留字
```

### f · telemetry_aggregator(7/26,far-jump 鬼資料)

```rust
// ✗ fn query(&self, w: u64) -> Option<Stats> { Some(self.buckets[w % N].stats) }
//    大跳窗後,slot 裡躺著同餘舊 epoch 的鬼(6%2 == 2%2)
// ✓ let b = &self.buckets[(w % N) as usize];
//   if b.epoch != w { return None; }                   // 讀門驗牌
//   record 側同款:落桶時 epoch 不對 → 重置桶再寫       // 寫門驗牌;兩扇門缺一鬼就進
```

**⚠ 口訣**:桶不是你的,牌對了才是你的。lazy validation 讓 record 嚴格 O(1),勝 eager 清掃。

### g · bounded_channel(7/26,oracle 4 紅同根)

```rust
// ✗ fn recv(&self) -> Option<T> {
//       if self.shared.sender_cnt.load(Acquire) == 0 { return None; }   // 蓋過「佇列還有貨」→ 丟資料
//       ...
// ✓ 睡 = 空 ∧ 還有 sender;醒來後佇列自己就是答案:
//   guard = cvar.wait_while(guard, |q| q.is_empty() && senders > 0).unwrap();
//   let item = guard.pop_front();                      // Some=有貨(drain);None=空∧關門
//   drop(guard); self.shared.wait_not_full.notify_one();
//   item
```

**⚠ 雙 Drop 六行**:見 §2c(昨天默寫方向反的那格——`fetch_sub(...) == 1` 的人**負責關燈**)。
**⚠ 沒 join 的斷言不是斷言**:見 §5 鐵律 3。

### h · timer_queue(7/24)

```rust
// ✗ BinaryHeap<Reverse<(Instant, Duration, u64)>>      // 次鍵放 interval → 同 deadline 排序錯
// ✓ BinaryHeap<Reverse<(Instant, u64)>>                // (deadline, id),tie-break 用 id

// ✗ let next = Instant::now() + interval;              // 每次醒晚一點,永久漂移
// ✓ let next = old_deadline + interval;                // drift-free;落後時 next 仍 ≤ now → pop 迴圈自然補課
```

**⚠ cancel** = lazy-delete 墓碑(HashSet<id>),pop 到再驗屍,不進 heap 挖人。

### executor(7/23,合約級主洞)

```rust
// ✗ fn poll(...) -> Poll<()> { thread::sleep(self.deadline - now); Poll::Ready(()) }
//    「poll 不准等」——poll 是問卷不是等待室,同步等 = 整個 executor 卡死
// ✓ if Instant::now() >= self.deadline { Poll::Ready(()) }
//   else { /* 安排未來的 wake(spawn 鬧鐘 or 登記 waker)*/ Poll::Pending }
```

**⚠ waker 要存最新**:每次 poll 覆寫 `*slot = Some(cx.waker().clone())`——task 會搬家,舊遙控器按了喊醒空位子。

### lru(7/22,零紅但 2 洞)

```rust
// ✗ put 淘汰路徑:淘汰了舊人,忘了 map.insert(新 key) + promote
//    repro:cap=2,put a → put b → put c → get(c) == None,len 縮水,下次 put 反淘汰新 key
// ✓ 淘汰跟插入是同一筆交易的兩半,寫完立刻 get 新 key 自驗

// ✗ unlink 頭/尾分支:殘留鄰居的髒指標(被 push_front 先 unlink 設計暫時蓋住)
// ✓ unlink 後鄰居雙向都要重接;單獨測 unlink,別靠上層動作遮
```

### spsc 骨架(空白 ×3 的傷疤譜系)

```rust
// ✗ fn write_slot(idx: usize, item: T)                 // 少 &self(7/19 三類肌肉傷,7/27 又回鍋)
// ✓ fn write_slot(&self, idx: usize, item: T)          // UnsafeCell 就是為了 &self 下可寫

// ⚠ 座標系開寫先釘一句:"head = next to consume, tail = next to write"
//    (或乾脆 read_idx / write_idx,kfifo 同款,零歧義)
```

---

## 9. 低機率加考題型(認題卡——會認、有 30 秒開場就夠,不用練)

> 原則不變:靠已會的 80% 打。這節只保證「聽到關鍵字不空白」。

### 9a. Rate limiter(token bucket)——認題:"N per second" / "burst" / "throttle"

30 秒:_"Token bucket — capacity for bursts, refill rate for steady state, and I refill **lazily** on each call instead of running a timer thread."_

```rust
struct Bucket { tokens: f64, cap: f64, rate: f64, last: Instant }
fn try_acquire(&mut self) -> bool {
    let now = Instant::now();
    self.tokens = (self.tokens + self.rate * (now - self.last).as_secs_f64()).min(self.cap);
    self.last = now;
    if self.tokens >= 1.0 { self.tokens -= 1.0; true } else { false }
}
```

O(1)。⚠ lazy refill = h 題定理再現(**不為 refill 開 thread**);併發版 = `Mutex` 包整顆就好(state 兩個 f64,contention 故事講不起來)。

### 9b. Merge K sorted streams(合併 K 路 log,telemetry 味)——認題:"K sorted" / "merge in order"

30 秒:_"A min-heap of one head per stream — pop the smallest, refill from the same stream."_

```rust
let mut heap = BinaryHeap::new();                     // Reverse<(key, src_idx)>
// pop → 輸出 → iters[src].next() 有貨就 push 回去
```

O(N log K)。⚠ `Reverse` + tie-break(h 題次鍵傷疤同款);某路耗盡就不補,別 unwrap。

### 9c. Top-K frequent(熱點統計)——認題:"most frequent" / "top K"

30 秒:_"Count with a HashMap, then keep a **min**-heap of size K — the root is the K-th place, anything smaller doesn't enter."_

O(N) 計數 + O(M log K) 掃描。⚠ 是 **min**-heap 不是 max(留最大 K 個,門檻是堆頂);全排序 O(M log M) 當下界講。

### 9d. Varint / bit packing(wire protocol 家族)——認題:"compact encoding" / "7 bits + continuation bit"

```rust
// encode:低 7 bits 一組,還有貨就插 0x80 continuation
while v >= 0x80 { out.push((v as u8) | 0x80); v >>= 7; }
out.push(v as u8);
// decode:shift 累加;u64 最多 10 bytes,超過 = malformed
```

⚠ e2 mask 家族:mask 要足位(`v as u8` 截斷天生對,手寫 `(1<<7)-1` 就要數對)。BE/LE 之爭不存在——varint 是 byte-oriented。
