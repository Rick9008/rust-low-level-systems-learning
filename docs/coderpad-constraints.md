# CoderPad Rust 環境限制(面試約束)

面試在 CoderPad 上進行。這份文件記錄已確認的環境限制,以及每條限制對練習方式的
實際影響。repo 的模組分級(README「學習路徑」)與 `rehearsals/` 的彩排規則都以此為準。

## 限制與影響

### 1. 單檔

整份解答活在一個編輯器 buffer 裡:沒有模組樹、沒有 `mod` 檔案分割、沒有
`tests/` 目錄。測試跟實作擠在同一個檔案。

**影響:**
- 練習時就用單檔結構:實作在上,`#[cfg(test)] mod tests` 在下。
  `rehearsals/` 的規則「自己的測試寫在 `src/<name>.rs` 底部」就是在模擬這件事。
- 不要依賴「拆檔整理思路」的習慣——面試時沒有這個選項。

### 2. Cargo 有,但 crate 清單固定:無 libc、無 tokio、無 crossbeam

環境跑的是 Cargo,但依賴是預先固定的一組,不能自己加。已確認**沒有**
libc、tokio、crossbeam。

**影響:**
- 一切以 std 為前提:併發只有 `std::thread` / `std::sync`(Mutex、Condvar、
  Arc、mpsc)/ `std::sync::atomic`。這正是本 repo「std-only」約束的來源。
- **epoll 一族(`epoll_sys`、`event_loop`、`tcp_echo`、`file_io_offload`)在
  面試環境做不了**:沒有 libc,單檔裡手寫 `unsafe extern "C"` syscall 綁定
  也不現實。它們是 deep-dive 學習材料,拿來回答概念題,不是 live coding 題。
- 沒有 loom / proptest:正確性只能靠自己當場寫的測試 + 腦內 dry-run。
  平時練習時 loom 幫你「證明」的那些 interleaving 直覺,面試時要內化成
  「我知道這裡為什麼對」的口頭論證。

### 3. Toolchain 可能偏舊 → 寫 edition 2021 相容的 code

不能假設環境是最新 stable。

**影響:**
- 語法與 std API 停在 edition 2021 相容範圍:不用 edition 2024 才有的語法,
  std API 優先用穩定已久的那批,別賭新 API 存在。
- `rehearsals/` crate 刻意設 `edition = "2021"`:在這裡寫得過編譯,
  到了 CoderPad 至少語法層不會爆。

### 4. 有 Run 按鈕 → "dry-run before you Run" 是字面意思

CoderPad 有 Run 按鈕,隨時可以編譯執行。誘惑是寫兩行按一次,用編譯器當思考的
拐杖——在計時面試裡這是時間黑洞,而且面試官看得到你每一次手忙腳亂的 Run。

**影響:**
- 寫完一段核心邏輯,先在紙上/註解裡把 boundary case 手 trace 一遍
  (空、單元素、滿、wrap、切斷點),**然後才按 Run**。
- 這正是本 repo 5 pillars 的 [Dry-Run] 那一環;`rehearsals/` 計時彩排時
  請把「先 dry-run 再 Run」當成硬規則執行。

## 一句話總結

CoderPad = 單檔 + std-only + 舊 toolchain + 有 Run 按鈕。
能考的是:鎖/條件變數、atomic、執行緒生命週期、ring/framing/index-based
資料結構——全部都在 README 優先級清單裡;epoll 考不了,留作深讀。
