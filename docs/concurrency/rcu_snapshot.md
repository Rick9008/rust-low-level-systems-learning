# rcu_snapshot 設計取捨(讀多寫少的快照發布)

對應程式碼:`reference/src/concurrency/rcu_snapshot.rs`(單檔、零 unsafe)。
前置閱讀:[thread-safe-spectrum](thread-safe-spectrum.md)(它是光譜上
「讀多寫少 → 快照」那一站的實體)。定位:reference-only 讀物。

## 這一站解決的問題形狀

routing table / config / feature flags——以及上一輪討論的並發 trie/graph:
讀是熱路徑且要**一致的完整視圖**,寫少到可以承擔「整份重建」。
鎖階梯在這裡全部不對勁:`Mutex<T>` 讓讀者互相排隊;`RwLock<T>` 的讀者
要持鎖到讀完(長讀擋寫)、read-unlock 還是一次原子 RMW。

## 核心:`Mutex<Arc<T>>`,鎖裡只准碰指標

讀者 `load()` = 鎖住 → clone Arc → 放鎖(~20–40ns),之後在鎖外讀
不可變快照,想讀多久讀多久。寫者鎖外建新版,回鎖換指標。
RCU 三步的對應:**R**ead = 快照零同步;**C**opy = 鎖外 CoW;
**U**pdate = 換指標(對讀者一步原子)。

**RCU 最難的寬限期,被 Arc 免費解掉**:kernel RCU 要偵測所有 CPU 過
quiescent state 才敢回收舊版;這裡「最後一個讀者 drop Arc」就是寬限期
結束的精確時刻,一行代碼都不用寫。reclamation 問題(mpmc_list 的真 boss)
第三次被型別系統拆掉——前兩次:mpsc_list 的單 consumer、ws_deque 的裝箱。

## 寫者:樂觀重試(mini-STM)

`update(f)`:鎖外跑 `f(&cur)` 建新版,回鎖用 `Arc::ptr_eq` 驗證沒人插隊,
輸了拿最新版重算。契約:f 可能跑多次,必須無副作用。這個形狀值得記——
它就是 CAS 迴圈的高階版(比較的是「版本指標」而不是值)。

## 為什麼 std 沒有 AtomicArc(面試級考點)

「load 裸指標」和「引用計數 +1」是兩個操作,中間的縫裡最後一個持有者
可能已把物件釋放——你 +1 的是死記憶體。把兩步變一步的選項:
拿鎖(本版)、hazard pointer、epoch、arc-swap 的 debt list。
與 mpmc_list 的回收、ws_deque 的「偷看正被覆寫的槽」是同一個問題的三張臉:
**無鎖世界裡,「還有人在用嗎」本身就是一個要設計的並發問題。**

## 升級階梯與量級

| 階 | 讀者成本 | 買到 | 付出 |
|---|---|---|---|
| `RwLock<T>` | 持鎖整段讀 + 2 次 RMW | 最簡單 | 長讀擋寫、讀者互撞計數線 |
| **`Mutex<Arc<T>>`(本版)** | ~20–40ns 抓快照,之後零同步 | 讀不擋寫、永不撕裂、免費寬限期 | 全讀者共享一顆鎖+計數(核多會 ping-pong);寫 = clone 整份 |
| `arc-swap` crate | load 近乎 wait-free | 讀端熱點消失 | 依賴 + debt 機制的複雜度 |
| kernel RCU | 讀端**零原子操作** | 極致讀擴展 | quiescent 偵測、寫端寬限期等待,userland 難用 |

記憶體帳:讀者押著舊版就活著——同時存活版本數上限 = 並發讀者數 + 1;
寫熱或 T 巨大時,CoW 粒度要收窄(持久化結構/分段快照)。

## 選型一句話

讀:寫 ≥ 100:1 且讀要一致視圖 → 這一站;寫頻上來 → sharded_map;
單熱欄位 → 直接 atomic;要「讀到最新」的語意 → 快照根本不是你要的。
