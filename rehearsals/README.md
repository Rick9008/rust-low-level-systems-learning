# rehearsals —— 計時彩排(模擬 CoderPad 條件)

四題計時彩排。題目是面試 prompt 風格:只給場景、contract、規模,**不給任何做法提示**。
環境約束照 [`docs/coderpad-constraints.md`](../docs/coderpad-constraints.md)。
題 a–c 只准 std;題 d 用 tokio(pad 實測清單有)。

## 規則

1. **計時 45 分鐘一題。** 時間到就停筆,誠實記錄做到哪。
2. **模擬單檔:** 實作寫在 `src/<name>.rs`,**你自己的測試也寫在同檔底部的
   `#[cfg(test)] mod tests`**——CoderPad 就是全部擠一個 buffer。
3. **彩排時自己寫測試。** 先想 boundary、自己出測項。
   **先在紙上 dry-run,再 `cargo test`**(Run 按鈕紀律,字面意思)。
4. 你自己的測試轉綠之後,**才**跑參考測試對照:

   ```sh
   cargo test -p rehearsals --test <name>_test -- --include-ignored
   ```

   參考測試含刻意建構的 boundary case,預設 `#[ignore]`(workspace 保持綠)。
   對照重點不是過不過,是:**你的測試漏了哪一類邊界?** 漏掉的那類
   (wrap?空?重複操作?欄位切斷點?)下次要在動手前就想到。
5. 開始前不要偷看 `tests/`,也不要看 `reference/` 對應模組。寫完隨便你 diff。

---

## 題目 a:ring_drop_oldest

感測器以固定頻率產生讀數(`u32`),下游消費速度不穩定,偶爾停頓。
你要寫一個固定容量的 buffer 接在中間。

需求:
- 容量固定,恰好容納 `capacity` 筆(`capacity >= 1`)。
- **滿的時候不能 block、也不能拒收新讀數**:丟掉最舊的一筆,讓新讀數進來。
- 被丟掉的筆數要累計,監控系統會定期讀這個數字。
- 取出端是 FIFO。

**Part 1:** 單執行緒版(`SensorRing`)。
**Part 2:** 生產者與消費者各在自己的執行緒上(恰好一產一消),
把它做成執行緒安全的版本(`channel(capacity) -> (Producer, Consumer)`)。

API 簽名在 `src/ring_drop_oldest.rs`。

## 題目 b:pool_graceful_shutdown

服務啟動後要並發地對數百台設備做 health check;每個 check 是一個阻塞呼叫,
交給固定數量的 worker 執行緒消化。

需求:
- `new(workers)` 起固定 worker 數的 thread pool;`submit(job)` 丟任務進去。
- 服務收到終止訊號時呼叫 `shutdown()`,要求 **graceful**:
  - 所有**已被接受**(submit 回 `Ok`)的任務都必須執行完;
  - `shutdown()` 回傳時,上述任務保證已完成;
  - `shutdown()` 之後的 `submit` 一律拒絕(回 `Err(Rejected)`);
  - `shutdown()` 可能被呼叫多次(訊號處理常見),必須安全。

std-only(`std::thread` / `std::sync`)。API 簽名在 `src/pool_graceful_shutdown.rs`。

## 題目 c:frame_parser_heartbeat

裝置透過 TCP 傳送 frame,wire format:

```text
[u32 len(big-endian)][payload:len bytes]
```

`len` 是 payload 的 byte 數;`len == 0` 的 frame 是 **heartbeat**(沒有 payload)。

TCP 是 byte stream,沒有 message 邊界:一次 `read` 拿到的 bytes 可能只有
半個 frame,也可能一次夾好幾個 frame。

需求:寫一個 incremental parser:`feed(&[u8])` 吃進這次新到的 bytes,
回傳**這次新完成**的所有 frame(依 stream 順序)。heartbeat 也要如實回報。
假設 stream 格式正確(信任的 peer,不需處理 malformed)。

API 簽名在 `src/frame_parser_heartbeat.rs`。

## 題目 d:tokio_frame_server

(唯一用 crate 的一題:tokio。)

裝置閘道器:多台裝置同時透過 TCP 連上來,講題目 c 的協定——
`[u32 len(BE)][payload]`,`len == 0` 是 heartbeat。

需求:
- 用 tokio 寫 server:`serve(listener, idle_timeout)`,服務到 listener 出錯為止。
- 每條連線並發服務、互相獨立。
- 收到 data frame → 原封不動 echo 回去(同 wire format)。
- 收到 heartbeat → 不回應。
- 一條連線超過 `idle_timeout` 沒有任何 bytes 進來 → 關閉該連線。
  heartbeat 算流量,能讓閒置的裝置連線活著——這就是它存在的目的。
- TCP 照樣沒有 message 邊界(半個 / 多個 frame),題目 c 的功課這裡要重用。

API 簽名在 `src/tokio_frame_server.rs`。
