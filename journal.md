# Journal — R2 備戰日誌(8/2 重啟)

> 輕敘事版,給未來的自己回看用;詳帳在 `scratch/cards_YYYY-MM-DD.md`、進度在 `PROGRESS.md`、逐日計畫在 `docs/interviews/README.md`。

## 2026-08-02(日)— 從「是不是面不過了」到全系列最大題的綠燈

下午打 sim m 全場(watchdog,六題之最:sol 191 行),65 分鐘超時 20,收錶時 review 出 8+2 個洞——最重的一個是 clarify 花了半場確認「block 不可重做」,實作卻會重派已完成的塊。當下情緒跌到谷底,問出「我是不是面不過了」。轉折點是兩個數據:sol 行數證明這題本來就不是 45 分鐘白紙題,以及三天前的 sim j 是錶內收掉的——寫不完不是我的穩定屬性,是題的尺寸。之後不開錶修帳,十個洞全部自己的手關掉,參考測試 3/3 綠,再把六欄 tuple 重構成具名 Alarm struct(Ord 四件套一課順帶入袋)。sol 對照的三課:deadline 與 owner 同居讓 stale 無法表示、先問 N 再選形狀、殭屍=生存證明。

晚上打完 RK 回來,深夜補打 l lite:開錯檔(rehearsals 空白版)霧了 20 分鐘,一度覺得「題目太抽象敘述又少」——破案後 35 分鐘 4/4 全綠,還自修了一個「兩條 ring 游標混帳」的真 bug。賽後把 barrier 收斂成「store-release 的硬體版」,先值後訊號第四次現身。

今天真正的收穫不是兩題,是三條 meta:①討論和落地之間的斷線才是 spec-heavy 題的失分點,state 表跳過去洞就沒有位置放;②英文 spec 是用聽的不是用讀的——三筆漏接實錄換來 read back/抄紙/編號逐答三對策;③情緒低谷要用數據校準,不是用意志力硬扛。

凌晨的產出(Claude 夜班):教材線兩頁(watchdog-deadline-design 🐕、hw-blocks-primer 🏗️ 全 SVG 圖解入門)+ 作戰本積木圖鑑節;algo 系開張(sim o boot_planner 全鏈 + AG-R/AG-T 兩卡)——PDF 說的「演算法穿硬體皮」正式有對策。明天:sim o 開機 → n lite → 洞複掃 → FP/TQ 卡。

（詳帳:`scratch/cards_2026-08-02.md`;8/1 帳:`scratch/cards_2026-08-01.md`;7/31 帳:`scratch/cards_2026-07-31.md`）

## 2026-08-03(一)— 躁與 Kahn:從「我是不是不該準備了」寫到「很有成就感」

早上先把昨天挑出的 sol 刺收掉(tries 只進不出;我出的設計方向「兩層 map」對了一半,終點是搬進 ReqState 同居——跟我彩排版的設計殊途同歸)。l 複讀升級成深問課:自己推出 MMIO 佇列就是 SPSC、消費者是矽;追問「多人 submit 怎麼辦」摸到 NVMe per-core queue pair 的門;順手複驗了 watchdog「6 台不用 heap」——跟 sol 同款判斷,「先問 N 再選形狀」這課算癒合。

下午進 sim o 前導,出事:extract_cycle 同一段講三輪還是霧,一路跌到「LC 600 題是不是白刷了」「我是不是不該準備了」。轉折是一張對映表:sim o = LC 207+1136+2050 換皮、extract_cycle = LC 142 換皮——而我在最霧的時候零錯背出了 Floyd 兩階段。看不懂的從來不是演算法,是「用解法詞彙寫成的檔頭」加上飽和的腦。換成 LC 題面,秒懂;開始寫 code,躁退;寫完很有成就感。機制記下:躁 = 被動解碼沒有回饋;動手 = 已知模式+每步有叮聲。8/6 的逃生梯就是定界句——它的本質是把面試官的散文翻成自己的題面。新規矩:drill 檔讀法 = 測試→簽名→寫→檔頭最後;algo 首打槽 45m。

sim o 本體三輪拉鋸:v1 Dijkstra 慣性走火(OR 閘入場,cycle 測試吊死)+ filter().enumerate() 源頭認錯;v2 閘裝了但鬆弛關在閘裡,「同圖換個邊順序答案就變」被 repro 實錘;v3 自己走完紅測先行(先紅後綠),8/8 全綠,自我診斷收尾:「push 條件是 Kahn 不是 Dijkstra」。賽後追問挖到一句好的:parent 不能拿來找環——它只記「發射過的鬆弛」,而環是歷史沒發生的地方。

傍晚狀態回升,晚場自排(在家):sim o 複讀(stepper 這時才解鎖,正確的打開時機)→ n lite(招牌考點 indegree 入場閘,下午用真 bug 踩過,算複驗)→ 舊 code 複習+口說。i–k 複掃/骨架默寫/卡 FP/TQ → 8/4 空檔。

晚場帳(寫於 8/4 凌晨):n lite 照我臨場的要求改了制——不用前導卡,直接讀 harness 上半的英文 spec、打字 clarify,像真面試那樣。三個問題全問在刀口上:id 唯一性(追問出「唯一但不保證單調」,自己推出要發 seq)、wait_event 是不是 epoll 形狀、相依會不會指向已完成的 job——最後一問直接問進 Phase 2 的主雷,考官的評語是「你用 clarify 走進了陷阱的解法而不是陷阱」。state 表漏了整個 worker 側,被一句「walk me through: worker 2 done」補回來。drill 三十分鐘加十分鐘延長全綠;真洞一枚:waiting.remove 放在歸零判斷外面——「銷帳早於放行」,跟下午 sim o 的「鬆弛/入隊分家」是同一族,一天踩同族兩次。review 再抓兩洞:dispatch 一次只派一個(四台 worker 被串行化,而 submitted 順序這個 oracle 對平行度全盲)、dependents 只進不出(早上才結案的 sim m 刺當天回鍋)。

收尾默 async 兩皮,結果比想像慘也比想像值:首默 7 錯——tokio 的 use 路徑全忘、bind 的 `.await?` 被點名三次才加上、park/unpark 整組掉、future 被我 drop 掉重建(它本身就是那台狀態機)。兩輪自己修到 0,rustc 親驗。附帶收穫兩條記法:tokio 模組樹是 std 的鏡像(永遠不用背)、AsyncReadExt/WriteExt 是唯一死記(async 方法住在 Ext 上)。7 這個數字是今晚最值錢的產出——它出現在 8/3 半夜,就不會出現在 8/6 早上 09:15。

本日檔案:`drills/src/ds/boot_planner.rs`(填空+自寫紅測×2)|sol 刺三檔(7bce2f0)|`drills/src/concurrency/job_scheduler.rs`(n lite 5/5 綠)|`scratch/tcp_server.rs`+`scratch/executor.rs`(async 默寫底稿)|`scratch/cards_2026-08-03.md`|README/PROGRESS/rehearsals 對照表/SCHEDULE 同步。

（詳帳:`scratch/cards_2026-08-03.md`）
