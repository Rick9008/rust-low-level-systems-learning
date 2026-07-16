# hw_bridge 設計取捨 + 45 分鐘作戰順序

對應程式碼:`reference/src/hw_bridge/`(protocol → framer → handler → 雙 server → client)。
上游:[tcp_echo](tcp_echo.md)(IO 骨架)、[event_loop](event_loop.md)、[thread_pool](thread_pool.md)。

## 45 分鐘增量順序(面試實戰)

1. **Clarify(3 分鐘)**:命令集?(Ping/ReadSensor/SetFan)一問一答還是
   server 主動推?(一問一答)連線數量級?(數十)binary 還是 text?
   (嵌入式對端 → binary)錯誤要能區分「聽不懂這條」vs「連線壞了」。
2. **紙上定 wire format(5 分鐘)**:`[u32 len BE][u8 opcode][payload]`,
   **當場說死 len 含不含自己**(本協定:不含)。把 opcode 表寫下來。
3. **protocol.rs(10 分鐘)**:encode + `try_decode(buf) -> Option<(frame, consumed)>`。
   先寫測試:partial → None、兩個 frame 背靠背。
4. **naive 單連線端到端(10 分鐘)**:blocking accept 一條、read 迴圈、
   handler mock 掉(**Abstract the Noise:硬體回假值,聲明後往前走**)。
5. **補 framing(7 分鐘)**:`FrameReader::feed` + `next_frame` loop——
   處理半個 frame / 一次多個 frame。這是本題的評分重心。
6. **多 client:thread-per-conn(5 分鐘)**:accept 迴圈 spawn,handler
   進 `Arc<Mutex>`。**到這裡是一個能跑、正確的 server——先講完再優化**。
7. **有時間才:event loop 版**(通常是口頭 trade-off,見下表)。

## 核心考點:為什麼 framing 最容易錯

TCP 是 byte stream。`write(frame)` 三次,對面可能一次 `read()` 全收到,
也可能收到 1.5 個 frame。**message 邊界是你自己的協定要重建的**,
kernel 不管。三個 off-by-one 埋伏點:

1. len 含不含 len 欄位自己(兩種定義都存在,拿錯 = 永久錯位 4 bytes)
2. consumed 之後殘料的偏移(`buf[read_pos..]` 的起點)
3. 「差 1 byte 就完整」的等待條件(`buf.len() < 4 + len`)

reference 的測試策略:**窮舉每個切割點**(7-byte frame 的 1..6 全試)+
byte-by-byte 餵——這種 bug 靠幾個手挑案例抓不乾淨。

## binary vs text、length-prefix vs delimiter

| | 選了 | 沒選的理由 |
|---|---|---|
| **binary**(BE 整數) | ✓ | text(JSON):嵌入式端要 parser + 浮點;binary 定長欄位 O(1) 解析、頻寬小。text 的優勢(可讀、好 debug)用 wireshark dissector 補 |
| **length-prefix** | ✓ | delimiter(`\n`):payload 含 delimiter 要 escape(二進位資料必含);length-prefix 讀 4 bytes 就知道要等多少,zero-copy 切分 |
| delimiter 的優勢 | | 人可 telnet 手打、損毀後可 resync(掃下一個 `\n`);length-prefix 損毀 = 只能斷線 |

錯誤分層(本協定的設計決策):**framing 錯 = 斷線**(byte 流失去同步,
沒有 resync 點);**語意錯(unknown opcode / payload 長度不對)= 回 Error
frame,連線活著**(對端可能只是版本比你新)。

## thread-per-conn vs event loop

| | threaded | evented |
|---|---|---|
| 複雜度 | 直線邏輯、阻塞 IO | interest 狀態機 + 欠帳緩衝 + 回程信箱 |
| 每連線成本 | 1 thread(8MB 位址空間、~10μs switch) | ~數百 bytes 狀態 |
| 上限 | ~10² | ~10⁴⁺(C10K) |
| 硬體序列化 | `Arc<Mutex<Handler>>` | 單一 command worker 天然序列 |

**硬體控制器連線數就是數十,threaded 是對的答案**;evented 是「如果要接
一萬台」的答案。面試先寫 threaded,口頭把這張表講出來,比直接寫 evented
但寫不完更高分。

## evented 版的三段式 thread 切分

```text
[IO thread] epoll:accept + read/write + framing
     │ frame → job
[command worker ×1] 執行硬體命令(慢命令不卡 IO)
     │ (token, resp bytes) → outbox + eventfd wake
[IO thread] 醒來路由回應 → conn.out → flush
```

worker 恰好 1 條不是偷懶:協定沒有 request-id,client 靠 FIFO 對應回應,
多 worker 會亂序。**要多 worker,先改協定加 request-id**(client 端變
pending map)——「協定設計決定並發上限」的活教材。`in_flight` 計數器
保證 EOF 後在飛的回應送完才關線。

## sync client 的對應規則

一次一個 in-flight,第 n 個回應 = 第 n 個請求。`read_sensor` 驗證回應裡
echo 的 sensor_id——一行換到「回應錯位立即爆炸」而非靜默錯資料。
async/pipeline 版:request-id → `HashMap<id, waker/channel>`。

## handler-IO 對照組:handler 內部要做阻塞 IO 時的三種下場

`SlowHardware`(ReadSensor 走慢速匯流排,sleep 模擬)接上三種 server,
可執行證據在 `mod.rs::slow_handler_latency_contrast`:

| server | 慢命令發生時 | 何時是正解 |
|---|---|---|
| `server_evented_inline`(⚠️ 反面教材) | IO thread 凍住:**所有**連線的 read/write/accept 停擺整段 delay | 永遠不是。它存在是為了讓你看到病徵 |
| `server_evented`(offload,1 worker) | loop 不凍(IO 照跑),但延遲**跨連線傳染**——別人的命令在 worker 佇列陪排 | 下游只有一顆序列設備時(shard 沒意義,Mutex 照樣序列化) |
| `server_evented_sharded`(shard by conn) | 同連線保序(同 shard 單 worker FIFO)、跨連線隔離 | 每 shard 有**自己的下游通道**時 |
| `server_evented_spsc`(同 evented,換佇列) | 兩條 `Mutex` 佇列 → 兩條 SPSC ring + eventfd;handler 免鎖(worker 獨占) | SLA 是 p99.9 時——IO thread 不再被 worker 的鎖 preemption 拖累 |
| tokio(rehearsals 題 d) | async handler 天然不凍 loop——`.await` 就是讓位點 | pad 上的實戰答案 |

兩個設計句:
- **保序約束只有同連線才需要**——跨連線本來就沒有順序語意,所以平行單位是連線
  (`token % N`)。這是「協定沒有 request-id → FIFO 對應 → 並發上限被協定決定」
  這條因果鏈的出口。
- **shard 的前提是下游也能平行**。單顆硬體時 N 個 worker 還是在同一把 Mutex
  排隊——clarify 五問的 Q3(幾個 producer)要連著「下游長什麼樣」一起問。

## Production 對照

tokio + `tokio_util::codec`(`LengthDelimitedCodec` 就是 FrameReader 的
工業版)、prost/protobuf(schema 化的 payload)、gRPC(HTTP/2 之上的全家桶)。

## 互動教材

[artifacts/hw_bridge.html](artifacts/hw_bridge.html) — 逐 byte 餵進 FrameReader,
親眼看 `next_frame()` 一次一次回 `Ok(None)`(len 沒湊滿 4 byte 不行、湊滿了但 payload
還差 1 byte 仍然不行);黏包一次 feed 切出兩個 frame、malformed len 斷線 vs
未知 opcode 續命的對照(按下 malformed len,client 按鈕會真的全部灰掉);
threaded 6 threads vs evented 2 threads 的實測對比。
