# spsc_ring 設計取捨

對應程式碼:`reference/src/spsc_ring/`(`mod.rs` 教學殼 + `core_impl.rs` 演算法)。
前置閱讀:[ring_buffer](ring_buffer.md)(單執行緒版的 index 算術)。

## 從單執行緒 ring 到 SPSC:哪裡斷掉了

單執行緒版用 head+len。並發下 len 由 push/pop 兩方同時 +1/-1——這需要
read-modify-write(fetch_add),而且滿/空判定與 len 更新不是同一個原子步驟。
換**自由跑計數器**後:tail 只有 producer 寫、head 只有 consumer 寫,
每個變數單一寫者 ⇒ 一個 load + 一個 store 就夠,**整條 push/pop 路徑無 CAS**。
這是 SPSC 比 MPMC 快的本質:不是技巧,是問題本身退化了。

## 為什麼容量必須是 2 的冪

槽位 = `counter & mask`。計數器在 usize::MAX 溢位 wrap 到 0 時,
只有當 2^64 是 cap 的整數倍(⇔ cap 是 2 的冪),`& mask` 的序列才連續。
cap=3 時溢位點會從槽位 (MAX % 3) 跳到 0,靜默錯位——這是**正確性**問題,
不只是 `%` vs `&` 的效能問題。測試用 `channel_with_start` 後門把計數器
起點設在 usize::MAX-1,直接踩過溢位點驗證。

## Memory ordering:每一個都有名字

| 操作 | ordering | 配對與理由 |
|---|---|---|
| producer 讀自己的 tail | Relaxed | 單一寫者讀自己,無需同步 |
| producer 讀 head | **Acquire** | 配 consumer 的 head Release:確認槽位已被讀完才覆寫 |
| producer 寫槽位 → 存 tail | **Release** | 槽位寫入 happens-before tail 發布;讀到新 tail 的人必看到完整元素 |
| consumer 讀自己的 head | Relaxed | 同上單一寫者 |
| consumer 讀 tail | **Acquire** | 配 producer 的 tail Release |
| consumer 讀槽位 → 存 head | **Release** | 讀取 happens-before head 發布;producer 才能安全覆寫 |

把任一個 Release/Acquire 弱化成 Relaxed,loom 會在窮舉中找到
「consumer 讀到寫一半的槽位」的交錯並報 Causality violation——
本 repo 開發時實際注入驗證過,這不是理論。

## false sharing 與 `#[repr(align(64))]`

head 和 tail 若落在同一條 64B cache line:producer 每次 store tail,
MESI 協定把 consumer 核心上那條 line 打成 Invalid,consumer 讀 head 就 miss
——兩個「邏輯上無關」的變數在硬體層互踢,吞吐掉一個數量級。
`CachePadded`(align(64))讓兩個計數器各佔一條 line,~112B 換掉這個效應。

## 型別系統當同步工具

`Producer`/`Consumer` 不是 `Clone`,`push`/`pop` 拿 `&mut self`——
「單生產者單消費者」不是文件約定,是編譯期保證。想造出第二個 producer
只能 unsafe。這是 Rust 併發 API 設計的招牌手法(std 的 mpsc 同款)。

## loom 驗證架構(sync-shim)

`core_impl.rs` 只 `use crate::sync_shim as sync`;lib 給 std 型別,
`tests/loom_spsc.rs` 給 loom 型別 + `#[path]` include 同一份原始碼。
loom 驗的就是出貨的那份邏輯,而 production 路徑零 loom 依賴。
模型刻意小(2 元素、cap 1–2):loom 是狀態空間窮舉,小模型已覆蓋
所有「一步寫錯」的交錯;大模型只會指數爆炸,不會多抓 bug。

## Production 對照

rtrb(即本設計的工業級版,含 batch 讀寫)、ringbuf、
crossbeam::queue::ArrayQueue(MPMC,每操作都是 CAS,慢於 SPSC 專用)。

## 互動教材

[artifacts/spsc_ring.html](artifacts/spsc_ring.html) —— 環的 push/pop 與 usize 溢位逐步操作;
把 Release/Acquire 切成 Relaxed,親眼看那條 happens-before 邊消失、consumer 讀到未定義的值。
