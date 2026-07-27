# 檔案二 · 問題 run-sheet + 反問準備(週一 17:30,15–20 min)

> chat-Claude 擬稿,7/27 存檔入 repo。chat 後 2 分鐘內:三訊號打分+關鍵句原話寫在檔尾「## 實況」節(code-Claude 補的 capture 步)。

## 流程

**開場(1 min)**:自介一句(他讀過訊息,不重複)→「我最想搞清楚 role 實際的樣子,想直接進問題」。

## 四題

**Q1 — Role 形狀**
> 「Supercomputing SWE 的日常實際落在哪一塊比較多?provisioning、manufacturing test,還是 telemetry/observability?」

萃取:deep dive 押注方向 + 跟自己背景的重疊度。
**聽的耳朵**:JD 全文比 po 文更偏 firmware(BIOS/BMC、RoT、driver)。他若答「主要是 firmware integration」→ 日常偏整合驗證,可攜性收窄;答 logging / orchestration / 平台 → 寬端。JD 裡 "system level logging for large scale server deployments" = 你 Asio log daemon 的直接對應,聊到可以認領。

**Q2 — Build vs. support(含 library / 可攜性)**
> 「team 建的這些系統,有多少是做成 reusable 的 platform / library 讓後面的東西長在上面,多少是 case-by-case 解產線的題?」

萃取:建平台 vs. 救火隊。他答「我們在把 X 平台化」→ 追問 X 是什麼。
**聽的耳朵**:這題的答案同時回答「我有沒有 infra 貢獻空間」。他答到平台化方向 → 接一句「這正是我想做的類型」,表態一次就好。
**總判準(Q1+Q2 合起來測的東西)**:這份工作是逼你**造系統、追根因**(debug boot failure 追到 PCIe link training 那種深度 → 機器理解複利,好),還是把你變成整合流程裡的**一個工位**(調 vendor 參數、當翻譯窗口 → glue work,深度不複利)。firmware 本身不是壞字 —— integration without depth 才是。

**Q3 — Ownership 邊界**
> 「跟 San Jose 的分工怎麼切?台灣 team 是 own 完整系統,還是偏執行端?」

萃取:獨立 site 還是遠端手腳。(他 po 文自己寫 massive ownership,這是請他展開。)

**Q4 — 工程文化 + 誠實度測試(放最後)**
> 「您從 Google 帶了哪些工程實務過來?有沒有哪些是刻意不帶的?」

萃取:工程紀律水位(design review / code review / postmortem 有沒有在做)。「刻意不帶」的部分比「帶」更誠實 —— 講得出具體取捨 = 前三題可信;全是正面話 = 打七折。
(原版「跟預期不一樣的」備用:如果 Q4 他答得起勁、時間還有,這題當加碼。)

**收尾(1 min)**:謝時間 +「我明天面試,之後不論結果都會跟您 update」。

## 他反問時的準備

### 「你為什麼對 Etched 有興趣?」(45 秒,壓熟)
1. 我在 Synology 建的是高併發 streaming infra(mail 內容檢查 pipeline、log daemon)—— 看到 JD 的 telemetry「數十億硬體訊號」,是同一種系統換了 domain,想把這套能力用在更靠近硬體的地方。
2. 對 rack-scale 這種軟硬交界的系統有興趣:provisioning、hardware validation 是我沒碰過但想碰的邊界。
3. 小 team、ownership 廣 —— 您 po 文裡 decisions in days 那句,是我想要的工作方式。

### 「介紹一下你自己 / 現在在做什麼?」(30 秒)
在 Synology mail platform 四年,做兩類東西:一是 Rust/Tokio 的即時內容檢查 microservices(每節點約 3,500 封/分鐘);二是 Go/Rust 的雙節點 HA 系統(operation log、conflict resolution),production 六個月零事故。也把 Rust 引進了 team 的技術棧。

### 「你比較想做哪一塊?」
誠實版:telemetry/streaming 是我的即戰力,但我想藉這個 role 往硬體邊界走 —— provisioning、validation 這些是我想補的面。
(不要只答「都可以」—— 這題是在測你有沒有想過。)

## 三訊號判定(聽完整場後打分)

1. **SWE 工作本體**(Q1):✅ SWE 做軟體系統、firmware 另有 team(JD 分開列兩職缺)/ ❌ 全在 BIOS/BMC 整合
2. **造 vs. 修**(Q2):✅ 講得出正在從零建的系統,有名字 / ❌ 全是支援產線,說不出在建什麼
3. **深度空間**(Q4 / 整場質地):✅ 講問題會往下鑽到根因 / ❌ 全程管理層話術

規則:≥2 ✅ → 全力打完全程;≥2 ❌ → 降權但 TPS 照打(45 min 買後續資訊);混合 → 疑點帶進 deep dive round 驗證。
**注意:三個 ✅ ≠ 該去(offer 階段才比薪資/股權/Google 線);三個 ❌ 不影響週二 —— TPS 考的是你的 45 分鐘,不是 Etched 的 roadmap。**

## 紀律

1. 他講 → 你收 signal,不搶著接自己經歷;他主動問才給 30 秒版。
2. Q2、Q4 不追問第二層。chat 不是審訊。
3. **面試題型 / 考什麼 / 面試官風格,一個字不碰。**他主動給 → 聽,不引導。
4. 17:30 開始 → 18:00 前主動收尾,別讓對方先看錶。結束後晚上是你自己的:過一遍英文句庫、早睡 —— 隔天 8:45 是正場。

## 實況(7/27 當晚,Withers 口頭回報版)

- 結論:**聽完更想去**。近期主力=支援工廠測試程式+平台(出貨壓力),之後「回歸 SW」。
- 考題情報兩條(已入 taper 筆記+SCHEDULE 情報 #4):tests 執行順序(toposort,當晚已補練)/ DS 改 concurrent(主場)。
- 帶進 deep dive 的疑點:「回歸 SW」的具體時間表;工廠測試支援期多長。
