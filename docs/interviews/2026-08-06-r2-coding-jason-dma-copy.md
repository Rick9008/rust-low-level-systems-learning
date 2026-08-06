# 2026-08-06 R2 coding #2(Jason Catlin)——dma_copy over segments

**結果:順利,題目偏簡單,45 分鐘內含 follow-up 收完。** 原定同天連跑的 Jan Lagarden 場(09:15)因面試官臨時有事**改期 8/11(二)09:15**;recruiter debrief 隨之移到 8/11 10:00–10:15(Molly Huang)。當天實際只考一場。

## 題目

給定 primitive:

```rust
fn dma_copy(dest_addr: u64, source_addr: u64, size: usize) -> bool // true = success, false = fail
```

加上一組 segments(每段有 start addr + byte 數)。稍微 clarify 完就是寫一個 loop 把整段拷貝拆成合法的逐段呼叫。**Domain 又是 DMA**——R1(DMA dispatcher)與 sim i 的直系親戚,認題零成本;複雜度遠低於 sim 系(單一 fn + 迴圈,無 event loop、無多重 state)。

## Follow-up(唯一一題)

Q:有時候 `dma_copy` fail 可能是硬體壞掉,怎麼辦?

答(現場):比照 API gateway 的做法——**retry + exponential backoff**,並記錄 fail 次數設上限(避免對永遠不會成功的裝置 busy loop),超限後 **notify IT manager** 升級處理。

## 洞清單(場後自評,餵 8/11 Jan 場)

1. **Idempotency 沒點名**——sim m 的正課「idempotency 決定敢不敢 retry」這句沒端出來:retry 前應先問一句 `dma_copy` 失敗時是否可能已部分寫入、重打是否安全。答案的 ops 層(backoff/上限/升級)是對的,但少了這句 senior 裁決句。8/11 若再遇 failure-handling follow-up,先講 idempotency 再講 backoff。
2. **升級動作可以更貼硬體公司語境**——「notify IT manager」換成「mark the device unhealthy、把它移出 rotation、發 alert/telemetry」,同一個意思但詞彙在題目 domain 內。
3. 題目簡單=訊號:Jason 場可能是 screen 性質;**Jan 場(改期後單獨一場)按 R1/sim 經驗準備 spec-heavy 題型**,不因今天簡單而降備。

## 行程異動(2026-08-06 官方信確認,需 Reply All 回覆)

| 場次 | 時間 | 連結 |
|---|---|---|
| Technical Deep Dive — Ulysses Kao | **8/10(一)10:00–10:45** | meet.google.com/kbq-myia-tvk |
| Coding — Jan Lagarden | **8/11(二)09:15–10:00** | CoderPad app.coderpad.io/9FF772PP + meet.google.com/off-xvwe-qek |
| Recruiter Debrief — Molly Huang | **8/11(二)10:00–10:15** | 同上 meet |

待辦:①信要 **Reply All 確認時間**;② 另一封 **NDA 電子簽名**信寄到後面試前簽掉。
