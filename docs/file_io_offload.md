# file_io_offload 設計取捨

對應程式碼:`reference/src/file_io_offload.rs`。上游:[thread_pool](thread_pool.md)、[executor](executor.md)。

## 為什麼 epoll 管不了 regular file

epoll 的世界觀是「fd 有就緒狀態」。socket 有(緩衝空/有資料);
regular file **沒有**——它「永遠 ready」,read(2) 直接去做磁碟 IO,
該等就等(O_NONBLOCK 對 regular file 無效)。
kernel 乾脆對 regular file 的 `epoll_ctl(ADD)` 回 **EPERM**。

所以 async 世界的檔案 IO 只有兩條路:

1. **offload(本實作)**:丟給專用 thread pool 阻塞,完成後喚醒等待者。
   tokio `spawn_blocking` / `tokio::fs` 全家都是這條。
2. **io_uring**:completion model,提交 IO 請求、kernel 完成後通知。
   真 async file IO,不佔用者執行緒。聲明不實作(它是另一份完整教材)。

## 完成通知的形狀:future,不是 callback

`spawn_blocking(pool, f) -> JoinFuture<T>`:
worker 完成 → 放結果 + wake;等待者 poll → 沒好就登記 waker。
`Mutex<(Option<Result>, Option<Waker>)>` 是「一次性交棒」的最小正確實作。
兩個時序都要對(測試都有):

- 等待者先到:poll 登記 waker → worker 完成時 wake。
- worker 先到:結果已放,第一次 poll 直接 Ready,waker 全程沒用。

這與 executor 的 park permit 又是同一課:**交棒的訊號要帶狀態**。

## panic 的去向

worker panic 若被 thread_pool 的 catch_unwind 吞掉,結果永遠不到,
等待者 hang——最壞的失敗模式(無聲)。
本實作在 job 內自己 catch_unwind,把 panic payload 當結果送回,
`JoinFuture::poll` 用 `resume_unwind` **在等待端重拋**:
panic 跟著在乎它的人走(tokio JoinError 的簡化版)。

## 成本模型

一次 offload:job 入隊(鎖)+ worker 喚醒 + 結果回傳(鎖)+ waker 喚醒
≈ 個位數 μs,對 ms 級的磁碟 IO 是零頭;對 μs 級的 page cache 命中讀,
overhead 可觀——io_uring 在小 IO 高併發下的優勢就在這。
pool 大小:磁碟 IO 是「等待為主」,可以開比核數多(tokio blocking pool
預設上限 512);CPU-bound 工作才鎖核數。

## Production 對照

tokio `spawn_blocking`/`tokio::fs`、tokio-uring、glommio(io_uring +
thread-per-core)。

## 互動教材

[artifacts/file_io_offload.html](artifacts/file_io_offload.html) ——
把檔案 fd 丟進 epoll,看它被回報成「永遠 readable」,再看 `read()` 照樣
把整條 event loop 凍在 page cache miss 上(其他連線的延遲計數器就在旁邊爬);
然後切到 offload,同樣的磁碟時間,但 loop 一個 tick 都沒停。
附:`Mutex<(result, waker)>` 三種交棒時序(含 lost wakeup)。
