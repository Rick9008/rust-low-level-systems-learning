# ring_buffer 設計取捨

對應程式碼:`reference/src/ring_buffer.rs`。相關:[spsc_ring](spsc_ring.md)(並發版,講 memory ordering;本篇講 index 算術)。

## 三種索引表示法

| 方案 | 滿/空判定 | 代價 |
|---|---|---|
| head+tail 留一格空 | `next(tail)==head` 滿、`head==tail` 空 | 浪費 1 slot;`next()` 到處 wrap |
| head+tail 自由跑(不 wrap 的計數器) | `tail-head==cap` 滿、`==0` 空 | 要求 cap 為 2 的冪(mask 取實體位)+ wrapping 算術;**但兩個索引各自單調遞增,單寫者──這是 SPSC 選它的原因** |
| **head+len(本實作)** | `len==cap` / `len==0` | 最直白;len 由 push/pop 兩方共寫,並發下不可用 |

單執行緒:head+len 最簡單正確。並發 SPSC:len 會被兩條執行緒同時寫,
必須換自由跑計數器——這條演進線在 spsc_ring 收尾。

## wrap:條件減法 vs `%` vs mask

- `%`:整數除法,x86 ~20-40 cycle,且對 2 的冪以外的 cap 也對——但慢。
- mask(`& (cap-1)`):1 cycle,但強制 cap 為 2 的冪。
- **條件減法**(`if i >= cap { i - cap }`):1 cmp + 1 sub,cap 任意;
  前提是 i < 2*cap(push/pop 的位移最多 +cap,成立)。

## `Vec<Option<T>>` vs `Vec<MaybeUninit<T>>`

Option 版:零 unsafe,每格多一個 discriminant(T 有 niche 如 `NonZero`、參照時免費)。
MaybeUninit 版:密實,但要自己維護「哪些格子已初始化」的不變量 + 手寫 Drop——
一整類 UB 風險換幾個 bytes。面試先寫 Option 版,被追問再升級,升級路徑在
spsc_ring(那裡因為要跨執行緒共享,不得不用 UnsafeCell/MaybeUninit)。

## 滿時的兩種策略

- `push_back` 拒絕:backpressure,資料不可丟(工作佇列)。
- `push_overwrite` 覆蓋最舊:新資料比舊資料值錢(telemetry、最近 N 筆 log)。
  API 回傳被擠掉的元素,caller 決定要不要處理。

## Production 對照

`VecDeque`(std,可增長的 ring)、`heapless::spsc`(嵌入式固定容量)、
`ringbuf` crate。
