# 2026-07-28 TPS Round 1(coding)——DMA dispatcher

**結果:過,feedback 正向。** 下一階段:coding ×2 + technical deep dive + culture fit talk(日期見 README 狀態欄)。

## 題目

DMA 訊號處理。系統有 6 個 `DmaEngine`(id 0–5)。實作一個 dispatcher:接收 `DmaRequest`,拆成 blocks 轉發給 engine,全部完成後回報。

給定 API(todo 介面,只能呼叫不能改;簽名憑記憶記錄):

```rust
struct DmaRequest { request_id, block_nums, block_start_pos }

fn get_dma_request() -> Option<DmaRequest>;                           // 拉新 request
fn send_dma_request_to_engine(engine_id, block_num, block_start_pos); // 派一塊給某台 engine
fn get_dma_result_done() -> Option<DmaEngineId>;                      // 哪台 engine 剛做完
fn wait_event();                                                      // 阻塞等事件
fn submit_dma_request_result_done(request_id);                        // 整個 request 完成回報
```

## 題型特徵(與 repo a–h 彩排的差異)

- **一大堆 provided API + 一個要實作的 fn**,不是從零手搓資料結構。
- **很長的英文 requirement**,spec 有洞——大量時間花在跟面試官溝通、認清 spec。
- **不能用 AI**,面試官是唯一 oracle。
- **event loop 裡兩邊 state 同時追**:request 端(還剩幾塊)+ engine 端(誰空著、誰在做)。計算 engine 本身有 state,要輪詢好了沒——設計重心是「state 放哪、誰持有」,不是演算法。

## 我的解法

- 兩個 loop,sequential 接 request(一次處理一個)。
- `engine_waiting_queue`:空閒 engine id 的 queue(初始 0–5)。
- 收到 request → 有空 engine 且還有 block 沒派,就 `send_dma_request_to_engine`。
- `wait_event()` → `get_dma_result_done()` 回收 engine 進 queue,繼續派。
- 完成判定:剩餘 block == 0 **且** `queue.len() == 6` → `submit_dma_request_result_done`。

## 面試官 feedback

- ✅ 不熟 DMA 也能釐清 domain knowledge、抓對要做的事、給出最佳解。
- ⚠ 時間不夠,code 上有漏洞。

## 重做方向(i 題的底稿)

sequential 版的天花板,pipeline 多 request 要補的三件事:

1. **per-request state**:`HashMap<RequestId, remaining_blocks>`(或 fd-dense 的 `Vec<Option<_>>`)。
2. **engine 佔用表**:`engine_id → (request_id, block)`,done event 才路由得回正確的 request。
3. **完成判定改 per-request counter 歸零**——`queue.len() == 6` 只在單 request 模式成立。

下輪必問的 clarify(這題 spec 的天然洞):done 會亂序嗎?engine 會 fail/hang 嗎(timeout/retry)?request 進來比 engine 消化快怎麼辦(backpressure/drop)?block 之間有順序相依嗎?`wait_event()` 醒來保證有事嗎(spurious wakeup)?
