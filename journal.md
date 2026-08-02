# Journal — R2 備戰日誌(8/2 重啟)

> 輕敘事版,給未來的自己回看用;詳帳在 `scratch/cards_YYYY-MM-DD.md`、進度在 `PROGRESS.md`、逐日計畫在 `docs/interviews/README.md`。

## 2026-08-02(日)— 從「是不是面不過了」到全系列最大題的綠燈

下午打 sim m 全場(watchdog,六題之最:sol 191 行),65 分鐘超時 20,收錶時 review 出 8+2 個洞——最重的一個是 clarify 花了半場確認「block 不可重做」,實作卻會重派已完成的塊。當下情緒跌到谷底,問出「我是不是面不過了」。轉折點是兩個數據:sol 行數證明這題本來就不是 45 分鐘白紙題,以及三天前的 sim j 是錶內收掉的——寫不完不是我的穩定屬性,是題的尺寸。之後不開錶修帳,十個洞全部自己的手關掉,參考測試 3/3 綠,再把六欄 tuple 重構成具名 Alarm struct(Ord 四件套一課順帶入袋)。sol 對照的三課:deadline 與 owner 同居讓 stale 無法表示、先問 N 再選形狀、殭屍=生存證明。

晚上打完 RK 回來,深夜補打 l lite:開錯檔(rehearsals 空白版)霧了 20 分鐘,一度覺得「題目太抽象敘述又少」——破案後 35 分鐘 4/4 全綠,還自修了一個「兩條 ring 游標混帳」的真 bug。賽後把 barrier 收斂成「store-release 的硬體版」,先值後訊號第四次現身。

今天真正的收穫不是兩題,是三條 meta:①討論和落地之間的斷線才是 spec-heavy 題的失分點,state 表跳過去洞就沒有位置放;②英文 spec 是用聽的不是用讀的——三筆漏接實錄換來 read back/抄紙/編號逐答三對策;③情緒低谷要用數據校準,不是用意志力硬扛。

凌晨的產出(Claude 夜班):教材線兩頁(watchdog-deadline-design 🐕、hw-blocks-primer 🏗️ 全 SVG 圖解入門)+ 作戰本積木圖鑑節;algo 系開張(sim o boot_planner 全鏈 + AG-R/AG-T 兩卡)——PDF 說的「演算法穿硬體皮」正式有對策。明天:sim o 開機 → n lite → 洞複掃 → FP/TQ 卡。

（詳帳:`scratch/cards_2026-08-02.md`;8/1 帳:`scratch/cards_2026-08-01.md`;7/31 帳:`scratch/cards_2026-07-31.md`）
