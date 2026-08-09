# Culture fit 參考資料(沿革、追問防禦、為什麼這樣寫)

**這個檔不是拿來練的。** 練習用 [culture-fit-script.md](culture-fit-script.md)。

---

## 兩層規格(2026-07-31 定案)

- **★ 題 = 全英文逐字稿。** 7/31 實測發現:骨架版**唸不出來**——句子之間的中文接縫會卡住嘴。
- **其他題 = 能講出意思就好**,它們是低頻選配,臨場組句即可。

## 沿革

- **7/29 起草**:五題 ★ + 反問三條,素材 = 履歷 + Holdwin 簡報轉軸。
- **7/31 三輪(逐字化)**:★1 / ★2 / ★3 / ★6 / ★7 全數升級為全英文逐字稿。
- **7/31 深夜四輪**:★1 金句後補 p99 句;★2 換成「晶片定天花板、軟體定貼多近」;★3 證據段換成 Moderation demo 衝刺
  (原本的「面試準備」例子退役——**自我指涉 + 非工作產出**,兩個都是弱訊號)。
- **8/7**:★3 三細節補真 + 訊息改寫;★6 人物與場景補真;全檔重構成「問題 + 英文 + 中文意思」。

---

## ★1 Tell me about yourself

**結構**:三件日常(每件 = 名字 + 我做了什麼 + 一個規格詞)→ 一句話總結我的工作 → 工作之外 → why-Etched 鉤子。

- 「predictable includes fast — tail latency is part of the contract」這句是效能訊號,**不要為了縮短把它砍掉**。
- **8/7 改**:自介裡原本的「~3,500 msg/s per node」拿掉了。原因:那個數字的 scope 是單一階段的隔離量測
  (詳見 [deep-dive-notes.md](deep-dive-notes.md)),**60 秒的自介塞不進修飾語,裸講就是等著被追問到塌**。
  換成「同步坐在郵件投遞路徑上,所以它的延遲就是平台的延遲」——**訊號更強,而且沒有數字要守**。

## ★2 Why Etched

- 「晶片定天花板、軟體定貼多近」是**產業通則**(utilization gap;CUDA 護城河同理),不需要 Etched 內部知識,
  被追問也站得住。7/31 換掉原本的 "software decides whether the silicon delivers"——**那句斷言了他們的內部現實,你不確定**。
- **兩個天花板互相呼應**:第一段是「我的天花板是 kernel」,第二段是「晶片的天花板由軟體逼近」。這是刻意的。
- 被追問離職的負面細節:**回到第一句重申 toward,不展開任何抱怨。**

## ★3 Work-life balance

**⚠ 8/7 的改寫是換訊息,不是潤稿——這段一定要看懂再唸。**

原稿賣的是「hard sleep red-line + day ten as sharp as day one」(靠排程與睡眠紅線撐住,所以第十天跟第一天一樣清醒)。
但真實情節是**最後那個星期五和 William pair 通宵到星期六早餐**。這兩句放在一起,**面試官一問就對撞**。

改法:**通宵留著**(它是整段最有畫面、最可信的一句),但訊息改成
「兩週裡只有最後那一晚、是刻意的選擇、而且撐得住正因為前面沒那樣操」。
**強度訊號與節制訊號同時在,而且句句是真。**

收尾金句同步改成 "leave headroom, **and know when to spend it**"——因為新敘事就是「留餘裕 → 在關鍵處花掉」。
原本的 "or the tail latency gets you" 與通宵並存會刺耳,已退役。

**三細節補真(8/7)**:① 兩週;② feature 上線已一年多(進稿尾,回答「這個衝刺值不值得」);
③ 點名 William(你自己提的,留著 = 真實感)。

**⚠ Gjengset 引用的使用守則(7/31 加)**

1. **上場前自己把那篇 40-hours 文章重讀一遍**,確認你轉述的論點正確。(排 8/9 taper)
2. 被追問文章內容就**講論點、不掰細節**。
3. 如果對面的「拼命文化」訊號很強,把人名句縮短成
   **"I optimize for sustained throughput, not heroics"** ——論點不變,**少一個可以被挑戰的引用**。
4. 引用的價值:證明你對產能的想法是**讀過、想過的立場**,不是怕加班的托詞。

## ★6 Conflict

**人物與場景(8/7 補真)**:場合 = **你把架構畫完後,自己召集 manager / staff(tech lead)/ junior 的架構 review**;
提案人 = **staff 同事(tech lead)**,主張「用 mail flag 操作 dsync 幫我們同步,就不必自己做一個新系統」。

**為什麼原稿的 "last-write-wins with timestamps" 換成 dsync 版**

兩者不衝突——**靠 flag 讓 dsync 同步,本質就是狀態複製,而狀態複製的勝負規則就是「誰最後寫」。**
原稿講的是這件事的抽象名字,新稿講的是**當時真正被提出的具體方案**。技術結論一字未改
(7/29 你回填的理由本來就是「複製決定不了先後順序、也不知道誰的操作不可挽回」),只是把它接回真實現場。
**好處**:場合、人、提案都講得出來,追問不會空。

**追問防禦**

1. **dsync 一句話定義備著**:"Dovecot's mailbox replication tool; it syncs messages and their flags between servers."
   **被追問內部細節就停在這句,不掰。**
2. **政治安全**:全段給 tech lead credit("that's the right question to ask")——
   **這題考的是你怎麼不同意,不是「我糾正了我的 lead」**,語氣一歪就扣分。
3. **趕時間可砍兩處**:"And with exactly two nodes…later."、"the platform already had three of those"。
   砍完從 ~215 words(100 秒)回到 ~190 words(90 秒)。

## ★7 Failure

- 唸法:CVSS 10.0 唸 "CVSS ten point zero";authn / authz **直接唸 "authentication and authorization"**
  (唸縮寫反而卡嘴)。
- 被追 CVE 細節 → "still under embargo, so I'll stay at the design level."
  **這句本身就是專業訊號**,不是迴避。
- ⚠ 同 deep dive 的保密紅線:**停在「我漏了授權檢查、後果量級、我怎麼改流程」,不描述怎麼利用。**
- **備案 B**(如果想換一個非安全的 failure):v2 遺失視窗(設計層 failure)。
  但主戰場在 deep dive 專案一的演進段,所以這裡不建議換。
- **CPU-spin 不能用**——那是前人程式碼的 bug,只能當 debugging 故事,**不能認領成自己的 failure**。

## ★8 / ★9 / ★10 的定位

- **★8 Proudest**:直接用 deep dive 專案一的內容,**加值在「每個版本都被真實失敗逼出來」**這個角度。
- **★9 Ambiguous**:**引用他們自己面試官的回饋**,這是全題庫裡最強的證據形式——對方自己說過的話你不用證明。
- **★10 Learn quickly**:mmBERT 那個比 Rust reviewer 那個好,因為「我從來沒訓練過模型」的起點更低、對比更強。

## 反問三條的用法

debrief 只有 15 分鐘,**挑 2–3 個問,不要全唸**。
另外**一定要問結果時程**——那是 debrief 的實務目的,不問反而奇怪。

## ★7 追問彈藥補充(2026-08-09,Withers 口述補真)

修法不只是 internal-only 範圍限制,而是**按功能切分 API 面**:真正 node-local 的操作改 localhost 限定,
只有跨節點通訊/同步呼叫還暴露,且過 mTLS + 逐呼叫 authn/authz。被追 "what did the fix look like?" 用:
*"Part of the fix was splitting the interface by function: operations that are genuinely node-local became
localhost-only, and only the cross-node communication and synchronization calls stay exposed at all —
behind mTLS and per-call authentication and authorization."* 主稿五拍不動,這句是 ③ 的深度層。

## ★7 ①⑤ 事實修正(2026-08-09,Withers 抓到自相矛盾)

原句 "It was internal-only" 與 cross-cluster 打架(介面本來就跨節點走網路)。改為
"The traffic was node-to-node on a private network — nothing faced the internet — so I treated
the network perimeter as the security boundary";⑤ 金句同步改 "'It's on a private network' is an
assumption, not a boundary"。弧線一致:①私網裡混裝沒 auth → ③按功能切分+mTLS+per-call auth → ⑤週界是假設。

## ★2 改版(2026-08-09,Withers 要求講自己的話)

兩個真理由取代稿:①工作本身=高效能軟體+硬體重互動,是他想花接下來幾年深入的方向
(修法:不用「幫助我發展」當主詞,改「日常工作和成長是同一件事」——從拿取變投入);
②人生理由=整個職涯在台灣、沒進過矽谷新創,想測試自己在那種節奏和標準下的能耐
(修法:「想試一次」→「test myself at that pace and that bar」,去觀光客風險)。
舊句降級成盾牌:chip-ceiling 句answering「why Etched specifically」;toward/away 句擋「why leave」。
英文定稿在上場包 ★2 格。

**★2 再修(8/9)**:座位在 Etched 台灣擴點——「想進矽谷新創」改成「早期加入 SV 新創的台灣團隊、
參與把據點建起來,work at that pace and that bar」;對比改「Taiwanese companies」非地理。
與 ★11 反問第一題(台灣團隊邊界)互相呼應。

**Q9 動態分配(8/9)**:moderation AA=最強 ambiguity 故事,但已排 hardest problem 王牌 →
同場不重打(G&L「同 loop 別兩用」)。規則:AA 未出 → Q9 直接打 AA;AA 已出 → 回指一句+
R1 clarify 回饋收尾("Ambiguity is a questioning problem, not a knowledge problem")。誰先被問誰拿 AA。

## G&L 白板全面對表(2026-08-09 傍晚,上場包已同步)

1. 🔴 **★7 追問陷阱**:G&L failure 卡 = mTLS configured≠enforced/fail-open——在 Etched 場屬保密紅線 #1
   射程。被深追守 authz 層;安全深度彈藥=發現角度(外部研究員 responsible disclosure、
   「外部發現=內部測試缺口→修流程不只修 code」);教訓句可用不點名機制版
   ("I test that the bad connection gets rejected; I don't read the config and believe it")。
   備援 failure=milter 大重構 regression(integration test+checklist=安全網)。
2. **★6 補第二把**:gRPC vs JSON(被說服方向,"I weighted the priorities wrong"+commit 後抽象化 trait);
   dsync=說服人方向。兩向都備。
3. **新增三 chip**:Leadership=讀書會(降門檻+psychological safety→self-sustaining)、
   Mentoring=3 MR aftermath(成 Rust go-to)、Feedback=術語密度→講使用者面問題(寄信例)。
4. **Q10 技術判斷角度**:「分類任務不需 decoder 生成能力」+每階段數據推提案;
   選用收尾對 Etched 押韻(specialization beats generality)。
5. **Delivery 三備**:三連問/別「輸了照做」/別烈士。Hypothetical 公式:根因→處理→升級最後,接真實經驗。
