# 把 X 變 thread-safe:決策光譜

來源:`html_p/p5-thread-safe-spectrum.html`(互動版含 stepper 與 tradeoff 卡,
追問鏈與 self-quiz 也在那)。這份是濃縮 + repo 模組對映,7/25 口述底稿。

**核心心法:停在「能滿足需求的最高站」。** 往下走每一站都是拿通用性/實作
簡單度換更少 blocking——沒有需求逼你下去,就別下去。面試被問「怎麼把它變
thread-safe」,沿光譜由簡到繁走一遍就是最強答案;一開口就跳 lock-free
顯得沒判斷。

## 光譜七站(每站 = 上一站的失敗模式逼出來的修法)

| 站 | 工具 | 修什麼 | 新代價 / 陷阱 | repo 對應 |
|---|---|---|---|---|
| 0 | `Mutex<T>` 一把大鎖 | 起點:永遠正確、零負擔 | 讀不能並行;臨界區長 = 全員串行 | `bounded_queue`、`thread_pool` 內部 |
| 1 | sharded lock | 單點 contention → 切 N 片 | 跨 shard 要**按固定索引順序**鎖多把,否則 AB/BA 死鎖;熱點集中單 key 時無效 | `sharded_map` |
| 2 | `RwLock` | 讀仍互斥 → 多讀並行 | 寫擋全部讀;fairness 契約 unspecified(writer starvation 要講得出);讀端仍對鎖字做 atomic RMW | (無專模組;`async_sync` 的 docs 有 bound 對照) |
| 3 | RCU / ArcSwap 模式 | writer 擋 reader → 離線建新版、原子換 `Arc` 指標 | 每寫整份重建 + alloc;舊版等最後一個 reader drop。**std-only 誠實邊界**:真 lock-free 讀要 hazard-pointer 級機制(arc-swap crate);std 只能 `RwLock<Arc<T>>` 逼近(鎖只包指標 swap 的 O(1) 臨界區) | (概念站,無模組) |
| 4 | seqlock | ArcSwap 每寫一次 alloc → seq 奇偶 + retry,零配置快照 | 資料要小且 trivially copyable;**Rust 特有坑**:讀寫並發對非原子資料是 data race(UB)——payload 得存 atomic 才 sound | (概念站;`fd_registry` 的 token 打包是遠親) |
| 5 | lock-free(CAS) | 想無鎖做結構性修改 | 系統級 progress、單 thread 無保證;指標型結構的真難題是 **reclamation**(ABA / use-after-free) | `arena_lockfree`(generation 防 ABA) |
| 6 | wait-free(SPSC) | CAS 重試沒有有界步數 → 限定 1:1,每 index 單寫者,免 CAS | 只支援 1:1;多方要 Vyukov bounded MPMC(per-slot seq + CAS) | `spsc_ring`、`signal_pipeline`(扇入 = 多個 1:1,不是 MPMC) |

## 資料結構 → 預設站位(面試 cheat sheet)

- 並發 map:站 1(sharded)起手,熱點單 key 再議
- config / routing table(read-mostly、整批替換):站 3
- 小型 POD 快照(座標、統計對):站 4
- 計數器 / running-max:站 5 的純 CAS(無指標 → 無 reclamation 問題)
- telemetry 一產一消:站 6——這就是 JD 的答案位

## 兩個關鍵分水嶺

1. **讀寫比**:讀遠多於寫才值得離開站 0(→2→3);寫不少就留在 0/1。
2. **上游推不推得回去**(接 clarify Q1):blocking 站(0–2)天然 backpressure;
   lock-free 站(5–6)滿了只能 drop / 重試——full policy 要自己補。

## edition 2024 的一顆地雷(pad 上會踩)

`if let Some(x) = m.lock().unwrap()...` 的 guard:**2021 在 else 區塊裡仍持鎖**
(再 lock 就自我死鎖),**2024 已 drop**。pad 是 2024——但如果你用 2021 的直覺
「else 裡不能再 lock」反而是白限制;反過來在舊 codebase 用 2024 直覺會死鎖。
臨場最穩的寫法是不賭 drop scope:先 `let guard = ...;` 顯式作用域。
