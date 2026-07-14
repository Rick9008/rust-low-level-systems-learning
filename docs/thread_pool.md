# thread_pool 設計取捨

對應程式碼:`reference/src/thread_pool.rs`。相關:[bounded_queue](bounded_queue.md)(內部佇列同 idiom)、[file_io_offload](file_io_offload.md)(用池 offload 阻塞 IO)。

## Worker 迴圈的唯一難點:shutdown 不卡死

worker 的等待條件必須同時看兩件事:

```text
wait_while(|s| s.jobs.is_empty() && !s.stop)
```

「醒來先查 stop」的意思是:stop 必須參與 predicate。若只查 `jobs.is_empty()`,
Drop 置 stop 後 `notify_all`,worker 醒來看到佇列空、**睡回去**,`join` 永久卡死。
這是 thread pool 面試題的第一大坑。

## Shutdown 策略:drain vs abort

| 策略 | 行為 | 適用 |
|---|---|---|
| **drain(本實作)** | stop 後把佇列清空才退 | job 不可丟(寫檔、回應請求) |
| abort | stop 後立即退,pending 丟棄 | job 可重建(cache 預熱、推測性工作) |

drain 版的 predicate 天然支援:佇列非空時 `wait_while` 直接放行,worker 繼續拿 job;
只有「空 且 stop」才退出。Drop 回傳 = 所有已提交 job 完成,語意最好講。

## job panic:catch_unwind

沒有它:job panic → worker thread unwind 死亡 → 池容量悄悄 -1,無任何錯誤訊號,
最後一條 worker 死掉後 execute 進去的 job 永遠不執行。
`catch_unwind(AssertUnwindSafe(job))` 吞掉 panic;`AssertUnwindSafe` 成立的理由:
job 是 `FnOnce`,panic 後不會再被呼叫,無人能觀察到被撕一半的閉包狀態。
(production 會再把 panic 上報,例如計數器或重啟 worker。)

## 成本模型:為什麼要池

- `thread::spawn` ~10μs 級 + 預設 8MB stack 位址空間保留;每個小 job 都 spawn,
  成本可能超過 job 本身。
- 池把 spawn 攤平成一次性成本;代價是佇列的鎖競爭(worker 數多時可考慮
  work-stealing,見 rayon)。

## 邊界:execute-after-shutdown 為何不用處理

`ThreadPool` 沒有 `Clone`,shutdown 只發生在 `Drop(&mut self)`——
借用檢查保證那一刻不可能還有 `&self` 能呼叫 `execute`。API 形狀本身消滅了一類競態。

## Production 對照

- rayon:work-stealing、scope、parallel iterator。
- threadpool crate:與本實作幾乎同構。
- tokio 的 blocking pool:`spawn_blocking` 背後就是這個模式 + 動態擴縮。
