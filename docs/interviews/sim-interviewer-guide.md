# 面試官手冊(sim i–m)——⚠ 跑題中不准開

Claude(或任何 session)當面試官時照此運作:只回答被問到的 clarify(答案如下),不主動餵;Phase 1 驗收(邏輯對 + 能講出完成判定)才給 Phase 2。計時 45m 硬停。

---

## Sim i — DMA dispatcher v2

**隱藏 spec(被問才答)**:requests 會持續進來且**必須 pipeline**(sequential 做完一個再接一個 = Phase 1 可接受、Phase 2 明確不行);done event 只告訴你 engine id,**不含 block/request 資訊**(state 得自己記);done 不亂序(engine 級 FIFO);engine 不會 fail(那是 sim m);`wait_event()` 可能 spurious;block_start_pos 單位是 block 不是 byte。

**Phase 2(驗收後唸給他)**:"Now requests keep arriving while earlier ones are still in flight. A request must be reported as soon as **its** blocks are done — you may not serialize requests. Also, upstream now sends `cancel_request(request_id)`: cancelled requests must never be submitted as done, and their engines must be reclaimed."

**看點**:per-request 剩餘塊表 + engine→(request, block) 佔用表;`queue.len()==6` 完成判定必須消失;cancel 時「在engine 上的塊」怎麼收尾(等它 done 但不回報)。

## Sim j — Sensor interrupt pipeline

**隱藏 spec**:ISR 內不准 alloc/block/log;ring 滿 → **drop newest + 計數**(問了才給;沒問 = 看點沒抓到);wake 會合併(edge 語意);spurious wakeup 存在;FIFO 不 drain 完硬體會 overrun。

**Phase 2**:"Add a clean shutdown: on `stop()`, the worker must drain everything already in the ring, then exit — no lost samples, no hang. Also expose `dropped_count()` readable from a third thread."

**看點**:drain-then-sleep 的 lost-wakeup 序(check→sleep 縫);shutdown flag 與 sleep 的交互(先 flag 後 wake);dropped counter 的 Relaxed 夠不夠(夠,說得出為什麼)。

## Sim k — Per-core telemetry fan-in

**隱藏 spec**:producer 絕不 block(滿了 drop+計數);**單一 producer 內順序必須保留**,跨 producer 不要求;aggregator 醒來要 drain 所有 ring(輪詢每個 consumer);core 數開機定死不動態。

**Phase 2**:"One core is much hotter than the others. Make sure it cannot starve the cold cores' records — bound how much you take from one ring per round. Then: aggregator must flush at least every 10 ms even if woken constantly."

**看點**:round-robin + per-ring budget;park 前的 re-check(N 個 ring 全空才睡,又是 lost-wakeup);unpark 合併沒關係的理由講不講得出。

## Sim l — MMIO command queue

**隱藏 spec**:device 端不保證看見你 CPU 的寫入順序 → **填 descriptor 和寫 doorbell 之間必須 `barrier()`**(沒問/沒放 = 本題最大看點沒過);completion **Phase 1 保證按提交序**;ring 滿 = head==tail+cap,保留一格不必;doorbell 寫絕對 tail。

**Phase 2**:"Completions may now come back **out of order** — descriptors have a `tag` field you assign. Route each completion to the right command. And `submit` on a full ring must now return `Err(Full)` immediately — the caller handles backpressure."

**看點**:submit 序 fill→barrier→doorbell 一次寫對;讀 completion 先讀 CompTail 再讀 slot(方向反過來的 acquire);tag→command 在途表;full 判定的 off-by-one。

## Sim n — Priority job scheduler

**隱藏 spec(被問才答)**:同優先權 = **到達順序 FIFO**(沒問 = 平手語意沒抓到);done 只給 worker id;job 不可搶佔(派下去就跑到完);priority 0–255 越大越急;deps Phase 1 全空;job 量級 ~10⁴(heap 合理)。

**Phase 2(驗收後唸給他)**:"Jobs may now arrive with non-empty `deps`: a job may only be assigned after **all** its dependencies have completed. A dependency always refers to a job that has already arrived, there are no cycles, and note a dependency may already be complete by the time a job arrives."

**看點**:ready 結構選型(BinaryHeap O(log n) vs 每次線性掃,要講);**同權 FIFO 需要 seq 破平手**(BinaryHeap 不穩定,seq 在到達時發);`completed` 集合防「等一個永遠不會再來的完成事件」(dep 已完成的後到者);**priority 不能穿越 DAG**(p9 等 p5 的相依);加分:主動點出這就是 priority inversion、真系統用 priority inheritance。

## Sim m — Engine watchdog

**隱藏 spec**:block 操作**不 idempotent**(重做可能壞資料——他必須問!)→ 重派前要能證明舊 engine 真死或做 completion 去重;timeout 值 spec 不給,要他自己提「p99 塊延遲的數倍」並講理由;hung engine 之後可能吐出遲到的 done(**zombie done**)。

**Phase 2**:"A quarantined engine may later emit the done it owed — make sure a zombie done can't corrupt your state or complete the wrong block. Then add a retry budget: after 3 timeouts on the same block, fail the whole request upstream via `submit_dma_request_error(request_id)`."

**看點**:第三種 state(engine→deadline)+ `wait_event_timeout(最近 deadline − now)`;zombie done 的解 = 佔用表帶 generation/epoch(e2 同款);idempotency 問題不問就直接重派 = 扣大分,guide 提醒面試官當場追問「re-execute 安全嗎?」。

## Sim o — Boot-order planner(algo 系)

**隱藏 spec(被問才答)**:重複邊合法(indeg 按邊數對稱加減就自然正確);無依賴節點全進波 0;makespan 假設波內完全平行——「同時最多 K 台」被問到就答「好問題,K 上限讓它變 list scheduling(NP-hard 家族),今天聲明 K=∞,講得出『波內切 K 批 = makespan 上界』就滿分」;環回報只要**一個**環,不用全列;critical_path 多解任一條。

**看點**:一趟 Kahn 做三件事(分層/最長路徑 DP/環偵測);**「DAG 上最長路徑是 P、一般圖 NP-hard——因為無環才敢沿 topo 序 DP」這句話**;blast_radius 不含 failed 自身;波內排序=決定性輸出。扣分雷:對每節點跑 DFS 找最深鏈(O(V·(V+E)));cycle 只回 bool。
