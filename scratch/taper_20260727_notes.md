# 7/27 taper 收穫暫存(餵晚上 recall_checklist + hepta 上板)

## 內線情報 #4(7/27 coffee chat,software head 親口;收帳時併入 SCHEDULE 情報節)

- Role 實況:近期主力=支援工廠測試程式+平台(出貨壓力),之後「回歸 SW」——時間表是 deep-dive round 追問點;三訊號打分見 run-sheet 實況欄。
- **考題訊號兩條**:①「很多 test 決定執行順序」= **toposort/依賴排程**(舊裁決「graph 砍」失效→7/27 晚 40m 快寫補洞,題在 `scratch/toposort_practice_20260727.md`)②「資料結構改 multi-thread/concurrency-safe/lock-free」= ds_sync 階梯主場,口述資產已備(30 秒光譜+升級地圖)。

## 明早暖手複誦(7/28 08:00)

1. length-prefix 三行重默:`u32::from_be_bytes(...) as usize`(wire 型別解,再 as host)——今日唯一 ✗。
2. h 兩句英文:
   - _"Reschedule from the **old deadline**, not `now`, or every firing drifts."_
   - _"`wait_timeout` with a re-checked predicate — spurious and early wakeups are harmless by construction."_
3. Sender Drop idiom 方向:`if fetch_sub(1, Release) == 1 { 拿鎖放鎖; notify }`——**== 1 的人要關燈**(今天默寫改反過一次:寫成 ==1 return)。

## 口述金句(今天磨出來的,場上直接用)

- **抓自己 bug 三拍**:_"Wait — I think I have a bug here."_ → 點名場景(_"this breaks on the wrap-around case…"_)→ _"Let me fix that before moving on."_ 不道歉、不默改。
- **傷疤紀律句**:_"I've been bitten by this exact off-by-one before, so let me double-check the boundary."_
- **a 題 policy 句**:_"Drop-oldest means the producer itself consumes a slot on overflow — `head` gets two parties advancing it, the structure degenerates from SPSC to SPMC, forcing a CAS. **The policy decides the synchronization structure.**"_
- **lock-free 精確句**:_"Lock-free buys you a **worst-case guarantee**, not automatic average speed. A true SPSC ring wins both — no CAS at all. The moment CAS enters, average throughput depends on contention — **a parked waiter can beat a spinning CAS loop**."_
- 沒搶的 mutex 很便宜:futex fast path = 上鎖/解鎖各一個 atomic RMW,跟 lock-free 同量級。
- **executor #5 結案句**(口述債清空):_"My interview Delay re-arms on every poll — redundant wakes are harmless by the poll contract. Production registers instead: latest waker in a `Mutex<Option<Waker>>`, overwritten each poll because the task can migrate; one timer thread wakes it exactly once. Same contract, different price."_
- **trade-off 下上界配方**:沒走的路 ≥2 條永遠有貨——下界墊暴力解(帶數字否決:_"3,000 sleeping threads is 3,000 stacks and a scheduler tantrum"_)、上界放升級解(「規模再上去才值得付它的複雜度」)。

## Ordering 追問鏈(四層,一張卡)

clone=Relaxed(有 handle 才能 clone,count≥1,所有權論證)→ drop=Release(遺言:我做過的一切發佈在退出之前)→ 為何不全 AcqRel(N−1 個輸家不需要 Acquire;RMW 前不知道自己是不是 winner → 拆開付款)→ winner `fence(Acquire)`(drop 是**最大的一次讀取、唯一不拿鎖的讀取**——destructor 讀 ptr/len + free,靠 release sequence 一次收齊全部前人)。
**加碼**:在 bounded_channel 裡 counter 其實可 Relaxed——檢查都在 mutex 下、drop 先減後拿鎖放鎖,unlock 的 Release 把 Relaxed 遞減一起發佈(「鎖洗白 Relaxed」)。精確規則:**看 counter 是不是資料的唯一發佈通道**。場上句:_"Release here is technically redundant because the mutex publishes it, but I keep it so correctness doesn't depend on the lock placement."_

## timer 睡醒定理(三化身)

「睡在用舊狀態算出的 timeout 上,狀態被外人改了,就需要一條叫醒通道」:condvar 版 = wait_timeout+notify(schedule 插到最前面要 notify 睡的人)|epoll 版 = epoll_wait(timeout) + eventfd 門鈴(WAKE_TOKEN)|async 版 = tokio timer park/unpark。追問劇本:_"你 sleep until next deadline——我這時 schedule 一個更早的呢?"_

## 掃描記分(進晚上檢查表)

- 骨架默寫:1✓/5⚠/1✗(✗=length-prefix usize;⚠ 多為 compile 級手滑;fetch_sub 方向反一次)
- h 口述:⚠(heap/Big-O/升級路全對;掉 drift-free + wait_timeout 兩句招牌、傷疤句零、沒選的路只 1 條)
- a#1 dry-run:✓ 走 wrap(補課:滿載 push→head 前進→drop_cnt++→pop 那條傷疤路)
- d#1 dry-run:✓ 走 idle_timeout×半截 frame——自判「關連線是對的 policy」正確;磨尖=idle 語意兩種(逐 byte 重置 → slow-loris 養半 frame 不死;防法 max-frame-age 一句)。金句:_"a half frame is unprocessable by definition"_
- g#1 dry-run:✓ 走 recv drain(cnt=0+佇列有貨)——修版兩路統一從 pop_front 流出,Option 即答案。金句:_"Mutate under the lock; notify wherever you like."_(鎖外 notify 合法 vs Drop 側 store 沒進鎖的差別)
