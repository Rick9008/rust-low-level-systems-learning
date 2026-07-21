# tcp_echo 設計取捨

對應程式碼:`reference/src/io/tcp_echo.rs`。上游:[event_loop](event_loop.md);姊妹篇:[hw_bridge](hw_bridge.md)(在同樣的 IO 骨架上疊協定)。

## 這個模組真正在教的三件事

### 1. write 塞住怎麼辦(EPOLLOUT 的正確用法)

`write(2)` 只保證「寫進 kernel 送緩衝」,緩衝滿(對端收得慢)就 WouldBlock。
錯誤答案:迴圈重試寫到完——阻塞了整個 event loop,所有連線陪葬。
正確答案(本實作):

```text
寫多少算多少 → 剩餘進 per-conn 欠帳緩衝(VecDeque<u8>)
→ interest 加上 WRITABLE → 可寫事件來了再 flush → 清空後拆掉 WRITABLE
```

### 2. WRITABLE 是暫時的 interest,不是常駐的

LT 模式下 socket 幾乎永遠可寫 ⇒ 常駐 WRITABLE = 每輪 poll 都有事件
= busy loop 燒滿 CPU。interest 是隨連線狀態走的狀態機:

```text
無欠帳:READABLE
有欠帳:READABLE | WRITABLE
對端 EOF 且有欠帳:WRITABLE(flush 完就關)
```

`registered` 欄位記錄當前註冊,變了才 reregister(省 syscall)。

### 3. 連線生命週期的完整路徑

accept(迴圈到 WouldBlock,一次事件可能積壓多個連線)→ 讀(到
WouldBlock/EOF)→ 回寫(快路徑直接寫,慢路徑掛帳)→ 半關閉
(EOF 後仍 flush 欠帳)→ 清理(deregister + drop = close)。
每條路徑測試都踩過,包含 1MB 灌流逼出部分寫。

## 誠實的簡化(面試時要能講出來)

- **欠帳緩衝無上限**:慢消費者會吃爆記憶體。production 設高水位,
  超過就暫停讀該連線(拿掉 READABLE)——backpressure 沿 TCP 流回發送端。
- **LT 而非 ET**:LT 漏處理下次還報,好寫;ET 假醒少但必須讀寫到 EAGAIN。
  本實作讀寫本來就到 WouldBlock,改 ET 只差註冊旗標——但 accept 漏收
  在 ET 下是永久 stall,LT 下只是慢一輪。面試先 LT。
- 單執行緒:多核擴展要 SO_REUSEPORT 多 loop 或 accept 後派發(hw_bridge
  的 evented server 討論)。

## 成本模型

每輪事件處理:O(事件數 × 每連線 IO 量);記憶體 O(連線數 × 欠帳)。
單 loop 單核可服務的連線數瓶頸通常在「每事件的處理時間」,
echo 幾乎為 0 ⇒ 瓶頸在 syscall 與記憶體頻寬。

## Production 對照

tokio 的 `TcpStream`(readiness 藏進 async/await,欠帳緩衝 = 你的 write
future 卡住)、mio 官方範例、nginx(事件模型同構,C 實作)。

## 互動教材

[artifacts/tcp_echo.html](artifacts/tcp_echo.html) —— 可點的寫入 backpressure 狀態機:
灌 8 KiB 進去看 `write()` 撞上滿的送緩衝,親手比較四種處理方式(忙等 / 丟掉 /
阻塞 / 緩衝 + EPOLLOUT),看 interest 在 `READABLE` 與 `READABLE|WRITABLE` 之間
隨欠帳生滅,並把「清空後不拆 EPOLLOUT」的 100% CPU 空轉 bug 按出來。
