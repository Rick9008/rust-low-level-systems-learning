# epoll_sys 設計取捨

對應程式碼:`reference/src/epoll_sys.rs`。下游:[event_loop](event_loop.md)。

## Readiness vs Completion:先站對地圖

- **epoll(readiness)**:kernel 告訴你「fd 可讀/可寫了」,IO 你自己做。
  適合 socket(緩衝區狀態 = 就緒狀態)。
- **io_uring(completion)**:你提交「讀這個 fd 到這個 buffer」,kernel 做完通知你。
  適合所有 fd(含 regular file),本 repo 聲明不實作。
- select/poll 是 O(n) 掃描的前輩;epoll 用 kernel 內部的紅黑樹 + 就緒鏈表,
  wait 是 O(就緒數) 而非 O(註冊數)——這是 C10K 的分水嶺。

## 綁定的工程決策

- **`unsafe extern "C"` 自綁而非 libc crate**:面試約束。只綁用到的 7 個
  syscall wrapper(glibc 符號);每個宣告上方註明語意 + errno。
- **errno 不自己綁**:`std::io::Error::last_os_error()` 是 std 的合法後門,
  少綁一個 `__errno_location`,少一個 unsafe 面。
- **`EpollEvent` 的 packed**:x86_64 上 kernel ABI 是
  `__attribute__((packed))`(12 bytes)。忘了 packed → struct 16 bytes →
  epoll_wait 寫入的事件陣列整批錯位,症狀是「token 亂掉」,離根因極遠。
  packed 的代價:**不能取欄位參照**(E0793,未對齊參照是 UB),
  只能整值/欄位按值複製——測試裡有示範。
- **EINTR 策略**:`epoll_wait` 被 signal 打斷 → wrapper 內重試
  (對 caller 而言什麼都沒發生);`close` 的 EINTR **絕不重試**
  (fd 狀態未定義,重試可能 close 到已被重用的 fd)。
- **RAII**:fd 洩漏在長行程 = 資源枯竭;double-close = 關到別人的 fd。
  Drop 裡 close、錯誤吞掉(Drop 不 panic)。

## eventfd:為什麼不用 pipe 做 self-wake

| | eventfd | pipe |
|---|---|---|
| fd 數 | 1 | 2 |
| 資料 | 8-byte 計數器 | byte stream |
| 多次通知 | 合併(計數累加) | 累積 bytes,要讀迴圈 |

計數器語意天生就是「wake 訊號」的形狀:notify 是原子 +1,
drain 一次拿走全部。搭配 EFD_NONBLOCK,drain 在計數 0 時 EAGAIN 不卡迴圈。

## LT vs ET(測試可直接觀察)

`boundary_level_reports_once_edge_reports_once` 測試把兩者的差異釘死:
資料不取走,LT 每次 wait 都報(狀態持續),ET 只報第一次(0→非0 的變化)。
推論:**ET 模式必須把 fd 讀/寫到 EAGAIN**,否則殘餘資料永遠等不到下一次通知。
LT 寬容(漏了下次再報),代價是高負載下重複喚醒。

## Production 對照

libc(綁定)、nix(safe wrapper)、mio(跨平台 readiness)、io-uring crate。
