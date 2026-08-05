# 8/5 晚 mock 面試官包——Dependency-aware Job Scheduler(C++17)

> **選型裁決**:三選項中選「有 dependency 的 thread pool job 題」——thread pool 是它的
> Part A,graph(拓撲序/Kahn)是 dependency 的內核,一題吃掉全部三個選項。
> 階梯式出題:弱的 candidate 收在 Part A 也有完整交付;強的 45 分鐘剛好打完 Part B。
> 參考解:`mock_cpp_0805_sol.cpp`(已編譯驗證:g++ -std=c++17 全綠 ×3 + TSan clean)。
> ⚠ 這是全新出的題,不是本 repo sim n 的 hidden spec——可放心出給外人。

---

## §1 題目卡(唸給 candidate;英文)

**Part A(開場就給)**

> Design a job scheduler class in C++. It owns N worker threads.
> - `JobId submit(std::function<void()> fn)` — enqueue a job, return its id.
> - `void shutdown()` — all jobs submitted so far must finish; then workers stop and join.
>   Submitting after shutdown is an error.
> Standard library only. Correctness first, then we'll extend it.

**Part B(Part A 能跑之後才給)**

> Now extend `submit` to take dependencies:
> `JobId submit(std::function<void()> fn, const std::vector<JobId>& deps)`.
> A job may only run after **all** its deps have finished. Deps are ids returned by
> earlier `submit` calls. Everything else (shutdown semantics) stays the same.

不主動講的隱藏事實(等 clarify;問到 = 加分):見 §2。

---

## §2 Clarify 答案鍵(好 candidate 該問的,和你的標準答)

| # | 該問的問題 | 你的答案 | 沒問的後果 |
|---|-----------|---------|-----------|
| 1 | deps 會不會成環? | 「deps 只能是**先前回傳過的 id**——想想這意味著什麼」(答:建構上必為 DAG,不用 cycle detect) | 白寫 Kahn/DFS 環檢測,燒 10 分鐘 |
| 2 | dep 在 submit 時**已經完成了**怎麼辦? | 「必須照常跑」——這是本題最大的坑(§4 bug#1) | 卡死:remaining 永遠不歸零 |
| 3 | submit 會被多執行緒並發呼叫嗎? | v1 單一 producer 即可;但持鎖寫法天然 thread-safe,答哪個都行 | 無,設計選擇 |
| 4 | shutdown 是 drain 還是 abort? | **drain**:已 submit 的全跑完(含靠依賴鏈解鎖的) | 語意錯:ready 空就走人,鏈尾沒跑 |
| 5 | job 的 fn 會不會丟例外? | Part A/B 假設不丟;丟了怎麼辦是 follow-up(§6) | 無 |
| 6 | 假的 dep id(從沒 submit 過)? | 丟例外或 UB 皆可,講清楚就好 | 無 |

**開場 3 分鐘看什麼**:有沒有先列 state 表(每個 job 的狀態:waiting/ready/running/done + 需要哪些共享結構)再動手。直接開打碼的,Part B 九成會亂。

---

## §3 45 分鐘時間線(你手上的錶)

| 時間 | 應該在哪 | 落後時的推法 |
|------|---------|-------------|
| 0–5 | clarify + 口頭設計 | 「先講你的資料結構再寫」 |
| 5–20 | Part A 能跑(pool + condvar + drain shutdown) | 20 分還沒 condvar wait 對:給提示 L1 |
| 20–25 | 給 Part B,聽設計:remaining 計數 + dependents 表 | 只想到「輪詢檢查 deps」:提示 L2 |
| 25–40 | Part B 寫完 + 自己 dry-run 菱形 A→(B,C)→D | |
| 40–45 | 追問 §4 bug#1(late dep)+ 一個 follow-up | 沒時間就只追 bug#1 |

**提示階梯**(卡住才給,一次一級):
- **L1**(condvar):"What does a worker do when the queue is empty? What wakes it up?"
- **L2**(deps 結構):"Per job, what's the minimum you must remember to know it's runnable?"(要引出:剩餘 dep 計數 + 反向 dependents 表 = Kahn in-degree)
- **L3**(放行點):"When exactly does a blocked job become ready? Who makes that happen?"(答:在**完成者的 finish 路徑**裡遞減,不是等待者自己查)

---

## §4 經典 bug watch list(你的批改火力;全部是你自己踩過的)

1. **Late dep 卡死**(最高頻):remaining 初始化成 `deps.size()`,靠「完成事件」遞減——dep 在 submit 前已完成 ⇒ 事件永不再來 ⇒ 永久 waiting。**追問法**:「submit B 依賴 A,但 A 十秒前就跑完了,走一遍你的碼。」正解:submit 時查已完成集合,只對「還活著的 dep」計數。
2. **銷帳與放行分家**:「標記完成」和「遞減 dependents」不在同一臨界區 ⇒ 中間窗有人 submit 新依賴 ⇒ 丟失或重複放行。正解:finish() 全程持鎖,一個臨界區內做完:遞減→放行→erase。
3. **持鎖跑 user fn**:worker 拿著 mutex 呼叫 `fn()` ⇒ 單線程化;fn 裡再 submit ⇒ 自死鎖。追問:「fn 裡面呼叫 submit 會怎樣?」
4. **shutdown 語意錯**:worker 的離開條件寫 `stop && ready.empty()` ⇒ 還有 job 在跑、其 dependents 尚未放行,worker 卻先走光。正解:離開條件 = `stop && 未完成集合為空`(running 也算未完成)。
5. **先 notify 後立 flag / 不持鎖立 flag**:與 `wait` 判定交錯 ⇒ 丟失喚醒。口訣:先 flag(持鎖)後 notify。
6. **dependents 表只進不出**:job 完成不 erase ⇒ live map 無限長。輕追問即可(這是你 8/4 的 entry→get 課):「這個 map 什麼時候變小?」
7. **`if` 代替 `while`/predicate wait**:spurious wakeup 直接漏接。看到 `cv.wait(lk)` 裸呼叫就記一筆。

---

## §5 評分表(五支柱,每項 0–2 分)

| 支柱 | 2 分長相 | 0 分長相 |
|------|---------|---------|
| Clarify | 自發問到 §2 的 #2、#4 | 一個沒問直接寫 |
| Abstract | 先講 state 表/資料結構才動手 | 邊寫邊想,結構長出來的 |
| Iterate | Part A 先通再上 B;每步可編譯 | 一次全寫,最後才編譯 |
| Trade-offs | 能講「為什麼計數不用輪詢」「map 何時縮」 | 說不出替代方案 |
| Dry-Run | 自發用菱形依賴走一遍自己的碼 | 寫完就宣稱對 |

及格線:Part A 正確 + Part B 設計對(碼沒寫完 OK)+ bug#1 追問下能自己修。

---

## §6 Follow-ups(快手加菜,挑一個)

1. **失敗傳染**:fn 丟例外 ⇒ 該 job 標 failed,所有傳遞依賴它的 job 取消(不跑但要銷帳,否則 shutdown 掛死)。考:取消也得走放行管線。
2. **回傳 future**:`submit` 改回傳 `std::future<T>`(packaged_task);考 API 手感。
3. **completed 集合無限長怎麼辦**(參考解的已知 trade-off):選項=不准依賴已完成 job / 世代回收 / 容忍(每 id 8B,量級帳算給你聽)。這正是你 8/4 的 160KB bound 課,反問時你有完整彈藥。

---

## §7 參考解對照(`mock_cpp_0805_sol.cpp`)

核心結構 30 秒版:每 job 一個 `Node{fn, remaining, dependents}`;`nodes_` 只放未完成
(running 也在內,所以 `nodes_.empty()` 就是 drain 完成條件);`completed_` 集合接住
late dep;`finish()` 持鎖一口氣:遞減 dependents→放行歸零者→erase 自己→必要時
notify_all。四個 smoke test 對應 §4 的 bug #1/#3/#4(菱形序、late-dep、平行度、drain 長鏈)。
