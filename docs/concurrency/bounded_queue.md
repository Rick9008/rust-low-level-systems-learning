# bounded_queue 設計取捨

對應程式碼:`reference/src/bounded_queue.rs`。相關:[thread_pool](thread_pool.md)(同 idiom 的消費端)、[spsc_ring](spsc_ring.md)(無鎖版的對照)。

## 為什麼是 predicate wait,不是 if + wait

Condvar 有兩個不可依賴的行為:

1. **Spurious wakeup**:`wait` 可能無故返回(POSIX 明文允許,futex 實作上真實存在)。
2. **喚醒與搶鎖之間有窗口**:A 被 notify 後、重新拿到鎖之前,B 可能先進來把條件消費掉。

所以醒來必須**重查條件**,`wait_while`(= `while !pred { wait }`)是唯一正確形狀。
用 `if` 的版本在面試裡是一票否決級的 bug。

## 一個 condvar vs 兩個 condvar

| | 單 condvar(naive) | 雙 condvar(主體) |
|---|---|---|
| push 完成後 | `notify_all`(不知道誰在等什麼) | `notify_one(not_empty)` |
| 每次操作喚醒 | O(waiters) | O(1) |
| 正確性 | 相同 | 相同 |

單 condvar 版**不能**用 `notify_one`:可能喚到等空位的 producer,它重查條件睡回去,
該醒的 consumer 沒醒(lost wakeup)。`notify_all` 修正這點,代價是 thundering herd。
把「等資料」與「等空位」拆成兩個 condvar 之後,notify 目標明確,`notify_one` 就安全了。

## close 語意的三個決策點

1. **pop 遇到 close**:先 drain 再回 `None`。佇列裡的資料是有效工作,close 只該擋新資料。
2. **push 遇到 close**:立即 `Err(PushError(item))` **歸還元素**——caller 保有所有權,
   可以記 log 或走 fallback。直接吞掉是資料遺失。
3. **close 的通知**:兩邊都 `notify_all`。所有等待者都必須觀察到 closed 離場;
   `notify_one` 只放行一個,其餘永久卡死。

## 為何先解鎖再 notify

`notify` 時仍持鎖,被喚醒的執行緒立刻嘗試拿鎖又拿不到,多一次 context switch
(hurry-up-and-wait)。先 `drop(guard)` 再 notify 避開。注意:順序反過來**不影響正確性**
(喚醒者醒來還是會重查條件),純粹是效能取捨。

## 容量預配

`VecDeque::with_capacity(cap)`:佇列生命週期內零 realloc。動態擴容是 amortized O(1),
但高吞吐下單次 realloc 的延遲尖峰會出現在持鎖區間內——空間 O(cap) 換可預測延遲。

## Production 對照

- `std::sync::mpsc`:std 內建,但 MPSC 且 API 形狀不同。
- `crossbeam-channel`:lock-free MPMC + select,production 首選。
- tokio `mpsc`:async 版,滿時 `await` 而非阻塞 thread。

## 互動教材

[artifacts/bounded_queue.html](artifacts/bounded_queue.html) —— 可單步的 Mutex + Condvar 模擬器:
多 producer / 多 consumer 搶同一把鎖,`if` 與 `while` 兩種 wait 形狀可切換。
親眼看被 notify 的 consumer 在搶到鎖之前被另一個 consumer 插隊偷走元素,
`if` 版在空佇列上 `pop_front()`;還有 spurious wakeup 按鈕,以及 `close()` 用
`notify_all` 與只 `notify_one` 的對照(後者留下永遠醒不來的等待者)。
