# Thread Pool 完整版(execute + submit 回傳 + oneshot + panic 隔離)—— 2026-07-24 白天 Q&A 沉澱(壓縮版 7 卡)

> 源碼證物:`scratch/thread_pool2.rs`(完整版,編過+複核正確)| 射後不理版(rep#1 真 scope)複核紀錄見對話 | c#1:`rehearsals/src/frame_parser_heartbeat.rs`
> loom 親證:`reference/tests/loom_lost_wakeup.rs`(`lost_wakeup_when_no_lock_at_all` 用 Acquire/Release 照樣死)
> 相關舊卡:`hepta_20260723_pool_arc_ordering.md` 卡1(兩處方)、卡2(佇列選型)、卡3(Box/'static/Arc);`hepta_20260722_lockfree_day.md` 卡6(lost-wakeup)、卡5(fence)

> **已上板(2026-07-25 凌晨,壓縮 8→3 張,「Rust Low Level Notes」)**:卡1+3+4+4b →《Pool 完整版三軸》`49e33900`|卡2+5 →《Pool shutdown 兩題》`2885e430`|卡6+7 →《測試網與碼表(流程卡)》`41c51be3`。本檔保留全文為底稿;改卡要同步改這裡。

## 一句話骨架(今日主題)

**今天的根:pool 對 job 的型別是「瞎」的——泛型放錯層(struct vs 方法)就一路爆。** 外加三件配套:lost-wakeup 是「時機」不是「可見性」、回傳值走「側信箱」、panic 靠「三層 catch」。meta 教訓:Claude 給錯碼表(把進階版當 rep#1、還壓 10m),真面試 scope 是射後不理版。

---

## 卡 1:type erasure —— pool 對 job 型別是瞎的(核心)

- **戰報**:第一版寫 `ThreadPool<T, F>` / `Box<F<T>>` → 卡死 30 分。病根 = 想讓 pool「認得」job 的型別。
- **機制**:每個 closure 是**獨一無二的匿名型別**;`FnOnce` 是 trait 不是型別(`dyn FnOnce` unsized)。泛型 `<F>` 放 **struct** → 單態化成「某一種」→ `VecDeque<F>` 只裝得下那一種;但 pool 一生收**無數種** → 塞不下。
- **處方**:`type Job = Box<dyn FnOnce() + Send + 'static>`。泛型放**方法入口** OK(每次 call 各自單態化),但**進 queue 前 `Box::new` 擦掉**。口訣:**generic 進來,erase 之後才存。**
- 回傳 `T` = 第二根軸:走側信箱(卡3),pool 不碰 `T`。
- **成本**:每 job 一次 heap alloc(Box)+ 一次 vtable 間接呼叫;相對 job 工作量可忽略——這是「能收異質 job」該付的價。
- **面試句**:"The pool is type-blind — every job is erased to `Box<dyn FnOnce()+Send>`. Return values come back through a side channel, not through the pool."

## 卡 2:lost-wakeup ≠ 記憶體可見性(acq/rel 救不了)★ 今天最深

- **問句**:shutdown 用 `AtomicBool` + `Acquire`/`Release` 不就好了?
- **答**:不行——你把兩個問題混成一個。
- **分**:`acq/rel` = **可見性**(答「我讀到什麼值」);lost-wakeup = **時機/liveness**(答「notify 打中時,我掛上 condvar 等待佇列了沒」)。
- **縫在哪**:「查完 pred」→「park(掛佇列)」之間。notify 掉進縫 → 打**空佇列** → 丟掉。**讀到的值新鮮(false)也沒用**——問題不在讀到舊值,在 notify 打空。
- **只有鎖能關**:通知者必須握「等待者查條件用的那把鎖」去改條件,把「改條件」和「查條件+park」排成互斥,縫消失。
- **兩種合法擺法**:store 進鎖(教科書 / `stop` 進 `State`)、notify 進鎖(b#1 實採)。
- **親證**:loom `lost_wakeup_when_no_lock_at_all` 用的就是 Acquire/Release,照樣找到死鎖交錯。窮舉在證的正是這件事。
- **面試句**:"Acquire/Release answers *what value I read*; lost-wakeup asks *am I on the wait-queue when notify fires*. Only the mutex — held by the notifier while it mutates the condition — closes that gap."

## 卡 3:oneshot promise —— submit 的回傳走側信箱

- slot 型別:`Arc<(Mutex<Option<thread::Result<T>>>, Condvar)>`。
- **submit**:開 slot → clone 一份給裝箱閉包;閉包 = `catch_unwind(AssertUnwindSafe(f))` → 整包 `thread::Result` 塞 slot(**store 進鎖**)→ notify slot 的 condvar(**出鎖**);回傳 `JobHandle` 握另一份。
- **join**:`wait_while(|s| s.is_none())` → `take()` 回傳。
- 泛型 `<T, F>` **只在 submit 方法上**;裝箱前擦掉,pool 不碰 T/F。
- **join 側也守 lost-wakeup**:store 進鎖、notify 出鎖——同卡2 原則,舉一反三(今天自己搬對了)。
- **footgun**:`join(&self)` 能 join 兩次(第二次 slot 已 `take()` 成 `None` → 永等);spec 是 `join(self)`,by-value 讓型別擋掉重複 join。
- **面試句**:"submit boxes a wrapper that runs f, stashes the `thread::Result` into a shared slot, notifies; the handle holds the slot — T never touches the pool."

## 卡 4:panic 隔離三件套

- `catch_unwind(AssertUnwindSafe(job))`——`AssertUnwindSafe` **超容易漏**。
- **submit 閉包必須自己 catch**,否則 job panic → slot 永空 → `join` 掛死。
- slot 存 `thread::Result<T>`(= `Result<T, Box<dyn Any + Send>>`)才裝得下 panic → panic 傳得回 caller。
- **worker 外層也 catch**:擋 `execute` 那種射後不理、會 panic 的 job,保 worker 執行緒不死。
- job 跑前 `drop(guard)`:job 不在鎖內跑 → panic 不毒鎖(舊傷疤 ⑥)。
- **面試句**:"Every job runs under `catch_unwind`; a panicking job becomes an `Err` in the handle, never kills the worker or poisons the lock."

### 卡 4b:`catch_unwind(AssertUnwindSafe(|| {…}))` 語法解剖(單獨記)

- **`std::panic::catch_unwind(f)`**:跑 `f: FnOnce() -> R`,攔截 unwinding panic,回 **`thread::Result<R>`**(`Ok(R)` 沒炸 / `Err(payload)` 炸了)。= panic 的**防火牆邊界**。
- **`AssertUnwindSafe(x)`**:包裝器,**無條件**實作 `UnwindSafe`——「我保證這東西跨 unwind 邊界安全」。
- **為什麼一定要它**:`catch_unwind` 要求閉包 `UnwindSafe`(marker trait:panic 穿過去不會留下「被觀察到的壞不變式」)。閉包捕獲了 `&mut T` / `Box<dyn FnOnce>` / 內部可變的東西 → 編譯器**不敢**自動判定 → 報錯 _"may not be safely transferred across an unwind boundary"_ / _"does not implement `UnwindSafe`"_。`AssertUnwindSafe` = **逃生門**:手動斷言 OK。**它是 lint,不是硬保證。**
- **closure 形式**:`AssertUnwindSafe<F>` 自己代理實作 `FnOnce/FnMut/Fn`,所以 `catch_unwind(AssertUnwindSafe(|| job()))` 會呼叫裡面的閉包。若 `job` 已是 `FnOnce()` → `AssertUnwindSafe(job)` 直接包也行(不用再套一層 `||`)。
- **三個踩點**:
  1. **漏 `AssertUnwindSafe`** → E0277 "does not implement `UnwindSafe`"(最常見,你今天就漏過)。
  2. 回傳是 **`#[must_use]`** → 不接會 warning(pool 的 L84 那個)。`let _ = catch_unwind(…)` 或真的用它。
  3. **只攔 unwind**;`panic = "abort"` 編譯設定下攔不到(process 直接死)。預設是 unwind,一般沒事。
- **用在哪(邊界隔離,不是 try/catch)**:worker 執行緒隔離 job panic、**FFI 邊界(絕不讓 panic unwind 穿過 `extern "C"`,那是 UB)**、async task executor 攔 task panic。**當一般錯誤處理用 = anti-pattern。**
- **面試句**:"`catch_unwind` is a panic firewall at an isolation boundary — a worker, an FFI edge, a task — not general error handling. `UnwindSafe` is a lint; `AssertUnwindSafe` overrides it when you know no broken state can leak."

## 卡 5:submit-after-shutdown = policy(clarify 卡)

- **三分法**:① 拒絕 + 退回 job(`Result<(), F>`,mirror `mpsc::SendError`)② 默默丟 ③ panic / 當 caller bug。
- **關鍵區分**:graceful = **已排好的** job 全跑完;**stop 後才來的** submit 是**另一個問題**,答案是 policy 不是自動。
- 不 reject 的下場:worker 走光後 push 的 job = **orphan**,永遠沒人跑(notify 也打空)→ silently 丟。reject 把這洞關死(不只 UX,是正確性)。
- `execute` 回 `()` 沒法退 job → 只能「默默丟」或「panic」;想「拒絕+退回」就得回 `Result`,那就不叫 fire-and-forget 了。
- **clarify 句**:"On shutdown, should a late submit be rejected (job returned), silently dropped, or treated as a bug? I'll default to rejecting and handing the job back — nothing lost silently."

## 卡 6:測試網 —— 全 0 payload 是遮罩(c#1 + pool)

- c#1:紅測先行修 `may_compact` 雙洞(drain 不 rebase ptr → underflow;`..=4096` inclusive off-by-one)。正解 `drain(..self.ptr); self.ptr = 0`,兩洞一起死。
- **mutation 教訓**:植「排少一格」off-by-one,**全 0 payload 咬不住**(stale 的 0 跟正常的 0 分不出);換非 0(`[20; 4096]`)stale byte 汙染下個 len 前綴 → 才紅。
- 家族:**「我的測試只驗我想到的事」**(b#1 / lru / c#1 同源)。想到「別炸」,沒想到「別把 byte 排錯位」。
- pool:**沒寫測試 = 沒網**,mutation 沒東西咬——skeleton drill 合規,但要驗網得先有 smoke test。

## 卡 7:meta —— 碼表校正 + 難度分層(流程卡)

- **主傷疤癒合**:worker 三分支 De Morgan **這次沒翻**(睡 `empty && !stop`、退 `empty && stop`)。7/23 rep#1 的洞閉合。
- **新洞**:type erasure(第一次真正撞到——不是條件,是型別觀念)。
- **Claude 給錯碼表**:把 drill 的**進階版**(JobHandle/oneshot)當成 rep#1 spec,還壓 10m;rep#1 真 spec 是**射後不理版**(~40 行)。教訓:**spec 要對齊 rep 來源**。
- **難度分層**:完整版 ≈ **45 分題本身**(Jon 級也要 15–20 分全綠,10 分沒人)。面試 scope = **射後不理版(彩排 b)**;回傳值版 = **escalation**(finish 太快時面試官加碼——今天等於先存好加碼答案)。
- **45 分槓桿**:coding 自動化到 **~20 分**收掉,才保得住 clarify + boundary + dry-run。**b#1 死因 = core 溢時吃掉 boundary**——這是你要盯的整合風險,7/26 b#2 驗它。
