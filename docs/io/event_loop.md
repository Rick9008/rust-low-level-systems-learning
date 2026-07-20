# event_loop 設計取捨

對應程式碼:`reference/src/event_loop.rs`。上游:[epoll_sys](epoll_sys.md);下游:[tcp_echo](tcp_echo.md)、hw_bridge 的 evented server。

## 為什麼是「薄 poller」而不是「胖 framework」

兩種 event loop API 形狀:

1. **回呼註冊制**(libuv 風):loop 持有 handler,`register(fd, callback)`。
   Rust 下回呼要 `Box<dyn FnMut>` + 從回呼裡改 loop(再註冊/停機)會撞借用,
   需要 action queue 迂迴。
2. **poll 制(本實作,mio 的形狀)**:loop 只管
   register/poll/translate,事件帶 token 回來,caller 自己 match dispatch。
   狀態(連線表)在 caller 手上,借用天然分離。

面試裡 2 明顯好寫好講;1 的複雜度花在框架性上,不在考點上。

## 借用分離的兩個設計點

- **`Events` 由 caller 持有**:`poll(&mut self, &mut Events)` 之後,
  迭代 events(借用 Events)期間還能 `el.register(...)`(借用 EventLoop)。
  若 poll 回傳 `&[Event]`(借 self),dispatch 中就動不了 loop——mio 同款解法。
- **token 而非參照**:事件帶 u64 token,caller 用 `HashMap<u64, Conn>` 找回
  狀態。kernel 幫你保管一個 u64(epoll_data),Rust 這邊零借用糾葛。

## eventfd self-wake:跨執行緒喚醒的唯一正解

睡在 `epoll_wait` 的執行緒,只聽得見 fd。要它醒(注入任務/停機),
就給它一個「它有在聽的 fd」——eventfd。
`WakeHandle` 是 `Arc<EventFd>` 的 clone:迴圈先死,handle 的 fd 仍有效
(Arc 保活),不會寫進已重用的 fd 號碼。

**wake 先於 poll 不丟**:eventfd 計數已 +1,下一次 wait 立即返回——
與 executor 的 park permit、condvar 的 predicate 是同一課的第三種面貌:
喚醒訊號必須「帶狀態」,裸訊號(對已醒者喊話)必然丟。

## 事件緩衝大小

固定 64:一輪拿不完,下一輪 epoll 繼續報(LT;ET 也會因還有未消費的
邊沿事件…不,ET 不會——所以 wake fd 和所有內部 fd 都走 LT)。
64 是 syscall 次數與延遲的折衷;tokio/mio 預設同量級(1024 上限常見)。

## 沒做的(面試時聲明 stub)

timer(需要 timerfd 或 wait timeout + 時間堆)、
任務佇列(wake + `Mutex<VecDeque<Task>>` 即可疊上)、
多 loop 分片(SO_REUSEPORT / accept 後 round-robin 派發)。

## Production 對照

mio(本模組的工業版:抽象 epoll/kqueue/IOCP)、tokio reactor、libuv。

## 互動教材

[artifacts/event_loop.html](artifacts/event_loop.html) —— LT / ET 並排模擬器:
同一個「讀 32 bytes 就 return」的 handler,LT 靠重複通知續命,ET 則在下一次
`epoll_wait` 回 0 事件、68 bytes 永遠擱淺在 kernel 收信緩衝裡;另含 token → conn 的
O(1) dispatch、`EPOLLIN|EPOLLOUT|EPOLLET` 位元遮罩解碼器,以及沒有 eventfd 時
`shutdown()` 叫不醒睡在 `epoll_wait` 裡的執行緒。
