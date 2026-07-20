# fd_registry 設計取捨

對應程式碼:`reference/src/fd_registry.rs`。相關:
[arena_lockfree](../concurrency/arena_lockfree.md)(generation 防 ABA 的 lock-free 版)、
[event_loop](event_loop.md)(interest table 的使用場景)、
[cost-model](../cost-model.md) 第三節(poll vs epoll 的 Big-O 故事)。

## 這題在考什麼

JD 點名的 "event registry" 是 practical data modeling 題:kernel 每次只還你
一個整數(`epoll_event.data` 的 u64),你要 O(1) 找回 handler——而且 fd 會回收。
反射答案 HashMap 是錯的方向:它的問題不只常數,而是 **fd 重用 bug 完全沒解**。

## HashMap vs Vec slots vs slab/slotmap

| 結構 | lookup | fd 重用 bug | 空間 |
|---|---|---|---|
| `HashMap<fd, T>` | hash + probe(常數大、cache 差) | 沒解 | O(live) |
| `Vec<Option<T>>` by fd | 一次 array load | 沒解 | O(max_fd) |
| **+ generation(本實作)** | array load + 一次比較 | stale → `None` | O(max_fd) + 4B/slot |
| slab | array load | key 重用同樣沒解 | O(max_fd) |
| slotmap | array load + gen 比較 | 有解——但 key 是容器發的,fd 是 kernel 發的,對不上 |

fd 小、密集(RLIMIT_NOFILE 級別)⇒ O(max_fd) 空間可接受;
array load ~1ns vs hash 數十 ns(數字見 cost-model)。
本實作的定位:**caller 指定 index 的 generational slot map**——
slab 的形狀 + slotmap 的世代驗證,而 key 直接用 kernel 發的 fd。

## generation 在防什麼

時序:`close(5)` → kernel 回收號碼 → `accept` 回 5(新連線)→
event queue 裡還躺著舊 5 的 readiness event → dispatch 查表拿到**新連線**的
handler → 資料錯亂。低流量測試抓不到,高 churn 的 production 半夜爆。

解法一個欄位:`gens[fd]` 在 unregister 時 +1;舊 token 的 gen 對不上,
`get` 回 `None`,過期事件自然被丟棄。
與 [arena_lockfree](../concurrency/arena_lockfree.md) 同構:那邊 generation 防 CAS 的 ABA,
這邊防 stale dispatch——同一個「index 會被回收,持有者要驗明正身」問題。

## token = (gen << 32) | fd

`epoll_event.data` 給你 64 bits:恰好 gen(u32)+ fd(u32) 打包塞進去,
kernel 免費幫你攜帶完整身份,事件回來一個 u64 就能判過期。
面試主動講這句 + stale-event bug,是「建過而不是讀過」的訊號。

## 誠實邊界

- gen 是 u32、wrapping:同一 fd 經 2^32 次 register/unregister 後理論上
  false-match。真要防:64-bit gen(token 就裝不下 fd 了,得換表示)或
  下限檢查——本實作選擇宣告邊界,不加成本。
- `register` 撞活著的 slot 直接 panic:kernel 不重發活著的 fd,
  double-register 是 caller bug,靜默覆蓋只是把 bug 往後推。
- 單執行緒(event loop 內部結構)。要跨執行緒:鎖起來,或走
  arena_lockfree 的 lock-free free-list 路線。

## Production 對照

- **slab**:mio / tokio 生態的 token→state 標準底層(無 generation,
  靠使用紀律避開重用視窗)。
- **slotmap**:generational key 的通用版(key 容器自發)。
- mini-runtime(本 repo Phase 5)將以 `FdRegistry<Waker>` 作 reactor 的
  interest table——同一個結構,value 換成 waker。

## 面試對映

第一個 clarify:**fd 密集還是稀疏?會回收嗎?**
(稀疏 → 退回 HashMap 並講明 trade-off;會回收 → generation 上場。)
彩排題卡:`rehearsals/README.md` 題目 e2。
