# Culture fit 練習本(問題 + 要唸的英文 + 中文意思)

**這個檔只放三樣東西:面試官會問的問題、你要講的英文、那段英文的中文意思。**
沿革、追問防禦、為什麼這樣寫 → 全部搬到 [culture-fit-notes.md](culture-fit-notes.md)。
怎麼練 → [practice-method.md](practice-method.md)。

用途:**每一場的開頭自介** + 各場穿插的行為題 + recruiter debrief。
onsite **沒有獨立的 culture fit 場**,所以這些題會散在 coding 場和 deep dive 場裡冒出來。

用得到的場次:**8/10(一)10:00 deep dive(Ulysses Kao)** → **8/11(二)09:15 coding(Jan Lagarden)**
→ **8/11(二)10:00–10:15 recruiter debrief(Molly Huang)**。
反問三條(第 11 題)是 debrief 用的,也就是 **8/11 那天**,不是 8/10。

★ = 一定會被問、要練到順;其他是低頻,能講出意思就好。

| # | 問題 | ★ |
|---|---|---|
| 1 | Tell me about yourself. | ★ |
| 2 | Why Etched? Why leave your current company? | ★ |
| 3 | What are your thoughts on work-life balance? | ★ |
| 4 | How do you feel about working across timezones with a US team? | |
| 5 | Startup pace vs big company process? | |
| 6 | Tell me about a conflict with a teammate or manager. | ★ |
| 7 | Tell me about a failure. | ★ |
| 8 | Proudest project / biggest technical achievement. | |
| 9 | How do you handle ambiguous requirements? | |
| 10 | Tell me about a time you had to learn something quickly. | |
| 11 | 你要反問他們的三個問題 | ★ |

---

## 1. ★ Tell me about yourself.

**中文**:介紹你自己。(60–90 秒,收在「為什麼想去做硬體軟體協同設計的公司」)

> "I'm a systems and infrastructure engineer on Synology's mail platform — I've been there since twenty twenty-three, working in Rust, Go, and modern C++ on Linux.
>
> Three things fill my typical day. First, a two-node active-active high-availability layer — I built the state replication that keeps human approval decisions consistent when the network between the nodes partitions. Second, a C++ telemetry and logging daemon on an Asio event loop — many producers feeding one structured sink. And third, a Rust moderation daemon on Tokio doing real-time content inspection — it sits synchronously in the mail delivery path, so its latency is the platform's latency.
>
> My job, in one sentence: keep systems predictable when the network is unreliable, the load spikes, and a node dies. And 'predictable' includes fast — tail latency is part of the contract.
>
> Outside work, I build systems from scratch to understand them — a Redis-compatible key-value engine in Rust, and a deep dive into how async runtimes actually work.
>
> And that's why a company doing hardware-software co-design is exactly where I want to be — the interesting failures live at that boundary."

**中文意思**

1. 我是 Synology 郵件平台的系統與基礎架構工程師——2023 年到現在,在 Linux 上用 Rust、Go 和現代 C++。
2. 我的日常有三塊。第一,一個雙節點 active-active 的高可用層——我做的是狀態複製,讓人工審核的決定在兩個節點之間的網路分區時仍然保持一致。第二,一支跑在 Asio event loop 上的 C++ telemetry 與 logging daemon——很多 producer 餵一個結構化的 sink。第三,一支跑在 Tokio 上、做即時內容檢查的 Rust 審核 daemon——**它同步坐在郵件投遞路徑上,所以它的延遲就是平台的延遲。**
3. 我的工作用一句話講:**當網路不可靠、流量暴衝、有一台機器死掉的時候,讓系統仍然可預測。** 而「可預測」包含「快」——**尾端延遲也是合約的一部分。**
4. 工作之外,我用「從零做一遍」的方式理解系統——一個 Rust 寫的相容 Redis 的 KV 引擎,還有把 async runtime 到底怎麼運作挖了一遍。
5. 而這正是為什麼**一家在做硬體軟體協同設計的公司,就是我想去的地方**——**有意思的失效都住在那條邊界上。**

> 唸法:2023 唸 "twenty twenty-three"。
> 最後一句是 why-Etched 的鉤子,面試官十之八九順著它問下去——**正中下懷。**

---

## 2. ★ Why Etched? Why leave?

**中文**:為什麼想來 Etched?為什麼要離開現在的公司?(**不要講任何負面離職原因**)

> "For me this is a toward move, not an away move. I've learned a lot where I am — but the ceiling is that software stops at the kernel; the hardware underneath is always someone else's black box.
>
> Etched is making a bet I love: the chip sets the ceiling, and the systems software decides how close you get to it — schedulers, event loops, telemetry, the runtime between the model and the chip. That layer gets built from scratch, and it's performance-critical — which is exactly what I chase. My daily vocabulary — event loops, backpressure, binary protocols, latency budgets — transfers one to one. And the from-scratch part: today that's what I do on my own time; I want it to be the day job.
>
> So no — I'm not leaving something broken. I'm running toward the layer I've been trying to get closer to for years."

**中文意思**

1. 對我來說這是**「往哪裡去」的移動,不是「從哪裡逃」**。我在現在的位置學到很多——但天花板是:**軟體停在 kernel,底下的硬體永遠是別人的黑盒子。**
2. Etched 押的注是我很喜歡的一個:**晶片決定天花板,而系統軟體決定你能貼多近**——排程器、event loop、telemetry、模型與晶片之間那層 runtime。那一層是**從零開始造的**,而且**對效能極度敏感**——那正是我在追的東西。我每天在用的詞彙——event loop、背壓、二進位協定、延遲預算——**是一比一可以搬過去的**。至於「從零開始」這部分:今天那是我下班自己做的事,**我想讓它變成正職。**
3. 所以不是——**我不是在離開一個壞掉的東西。我是往我這幾年一直想貼近的那一層跑過去。**

> 「晶片定天花板、軟體定貼多近」是**產業通則**,不需要 Etched 的內部知識,被追問也站得住。
> 如果被追問離職的負面細節:**回到第一句重申 toward,不要展開任何抱怨。**

---

## 3. ★ Work-life balance

**中文**:你怎麼看工作與生活的平衡?(**陷阱題:答「我很重視平衡」和「我沒有生活」都扣分**)

> "Honest answer: I have my own rhythm, and I protect sleep — tired engineers write outages.
>
> But when something matters, I can sustain real intensity. William and I had two weeks to get our content-moderation feature to a hard demo deadline. We paced most of it deliberately — and then on the last Friday we paired straight through the night and went out for breakfast on Saturday morning. That happened once, at the end, and it was a decision — we could only afford it because the rest of the two weeks wasn't run that way. We made the demo, and that feature has been in production for over a year now.
>
> Long-term, I side with something Jon Gjengset — a Rust systems educator I learn a lot from — has argued: consistent, focused hours out-produce heroic hours over anything longer than a sprint. Sprints are fine when they matter, as long as they're sprints and not the steady state.
>
> I manage energy the way I manage capacity in a system: leave headroom, and know when to spend it."

**中文意思**

1. 誠實的答案:**我有自己的節奏,而且我保護睡眠——疲勞的工程師會寫出線上事故。**
2. 但當一件事真的重要,**我可以維持真正的高強度。** 我和 William 有兩週時間要把內容審核功能推到一個硬性的 demo 期限。**大部分時間我們是刻意控速的**——然後在最後那個星期五,我們一路 pair 到天亮,星期六早上一起去吃了早餐。**那件事只發生一次、在最後,而且那是一個決定**——我們撐得住,正是因為那兩週的其他時間不是那樣操的。**我們趕上了 demo,而那個功能已經上線一年多了。**
3. 長期來說,我站在 Jon Gjengset(一位我學到很多的 Rust 系統教育者)論證過的那一邊:**只要時間拉長到超過一次衝刺,穩定而專注的工時,產出會勝過英雄式的工時。** 衝刺在真的重要的時候沒問題——**只要它是衝刺,而不是常態。**
4. 我管理精力的方式跟我管理系統容量一樣:**留餘裕,並且知道什麼時候把它花掉。**

> **被追問「所以你會通宵?」**——這樣答:
> "Once, at the end of a two-week push, and it was a decision, not a habit. What I won't do is run that way as a baseline — that's how you get outages."
> 中文:一次,在兩週衝刺的最後,而且那是決定不是習慣。我不做的是把它當成常態——**那才是出事的做法。**

---

## 4. How do you feel about working across timezones with a US team?

**中文**:跟美國團隊跨時區合作你覺得如何?(主動給方案,不要只說「我可以」)

> "It works if the overlap is deliberate rather than accidental. Taipei morning is US West afternoon, so there's a natural window — I'd want that fixed and protected for anything that needs a real conversation. Everything else goes async, in writing.
>
> And honestly, async-first forces better design docs. I've seen that work."

**中文意思**

1. 這件事能成,前提是**重疊時段是刻意安排的、不是碰巧的**。台北的早上就是美西的下午,所以本來就有一個自然的窗口——**我會希望把它固定下來並保護好**,任何需要真正對話的事情都放進去。其他一切都走 async、用寫的。
2. 而且老實說,**async-first 會逼出更好的設計文件。我親眼看過這件事成立。**

---

## 5. Startup pace vs big company process?

**中文**:你偏好新創的節奏還是大公司的流程?為什麼?(**用實例,不要用形容詞**)

> "I'll answer with an example instead of an adjective. On the two-node H-A project there was no off-the-shelf answer for the conflict rules — I defined them, validated them, and owned the outcome. That's startup-shaped work happening inside a larger company, and it's the part I liked best.
>
> What I want from process is the part that catches mistakes — code review, design review — not the part that asks permission."

**中文意思**

1. 我用一個例子回答,而不是一個形容詞。在那個雙節點 HA 專案上,**衝突規則沒有現成答案——我自己定義、自己驗證、自己承擔結果。** 那就是**在一家大公司裡發生的、新創形狀的工作**,而那是我最喜歡的部分。
2. 我想要流程給我的,是**會攔住錯誤的那部分**——code review、design review——**不是要你去請求許可的那部分。**

---

## 6. ★ Conflict / disagreement

**中文**:講一次你跟同事或主管的衝突或意見不合。(收在「怎麼用反例收斂,不是誰說服誰」)

> "Sure — on the two-node H-A project. I'd drawn the architecture for the decision-replication layer and brought it to a review with my manager, our tech lead, and a junior teammate. The tech lead proposed something lighter: put the decision on a mail flag and let dsync — the mail store's own replication — carry it across. No new system to build.
>
> That's the right question to ask, so I took it seriously, and I tested it against a failure scenario instead of arguing preference. Flags replicate as state, and state replication settles conflicts by whoever wrote last. Partition: node A approves, the mail actually goes out; node B rejects. If the reject lands last, you've un-sent an email that is already in someone's inbox. And with exactly two nodes, there's no trusted clock to even say which one was later.
>
> That reframed the discussion. The gap was never data replication — the platform already had three of those. It was that nobody replicated what the application had *decided*. So we built the operation log, and derived the conflict rules from operation semantics: irreversible actions win. Six-plus months in production, zero inconsistency incidents.
>
> We didn't converge because someone argued better — a failure scenario made the answer obvious. That's how I like to disagree: bring the counterexample, not the volume."

**中文意思**

1. 有的——在那個雙節點 HA 專案上。**我把決策複製層的架構畫完之後,召集了我的主管、我們的 tech lead、還有一位資淺同事來 review。** tech lead 提了一個比較輕的做法:**把決定放在郵件的 flag 上,讓 dsync(郵件儲存本身的複製機制)幫我們帶過去。這樣就不必自己做一個新系統。**
2. **那是一個該問的問題,所以我認真看待它**,而且我不是去爭辯偏好,而是**拿一個失效情境去測它**。flag 是以「狀態」的形式被複製的,而狀態複製解決衝突的方式就是「誰最後寫」。分區的情況:節點 A 核准,信真的寄出去了;節點 B 退回。**如果那個退回最後才落地,你就「取消寄出」了一封已經在別人收件匣裡的信。** 而且就只有兩個節點,**根本沒有可信的時鐘可以說誰比較晚。**
3. **那個情境把討論重新框了一次。** 缺口從來不是資料複製——這個平台已經有三套了。**缺的是沒有人複製「應用程式決定了什麼」。** 所以我們做了那份操作日誌,並且**從操作語意推出衝突規則:不可逆的操作贏。** 上線六個多月,零不一致事故。
4. **我們收斂,不是因為誰比較會辯——是因為一個失效情境讓答案變得顯而易見。** 那就是我喜歡的不同意方式:**帶反例,不要帶音量。**

> **dsync 被追問就停在這一句,不要往下掰**:
> "Dovecot's mailbox replication tool; it syncs messages and their flags between servers."
> ⚠ 全段要給 tech lead credit("that's the right question to ask")。**這題考的是你怎麼不同意,不是「我糾正了我的 lead」**——語氣一歪就扣分。
> 趕時間可以砍兩處:"And with exactly two nodes…later."、"the platform already had three of those"。

---

## 7. ★ Failure

**中文**:講一次你的失敗。(**要有具體損失 + 機制性的改變**,不能只說「我學到要更小心」)

> "The failure that's genuinely mine: I shipped a cross-cluster operations interface without proper authentication and authorization. It was internal-only — so I treated the network boundary as the security boundary.
>
> It later surfaced as a critical vulnerability — CVSS ten point zero. I can't share the technical details because the CVE is still under embargo, but the design lesson is fully mine to own.
>
> I also designed the fix myself: defense in depth — mutual TLS at the transport layer, application-level authentication and authorization on every call, and localhost-scoped access control for the cross-cluster operations.
>
> Two things changed mechanically after that. Security review moved into the design phase instead of being a bolt-on. And my personal default flipped: every interface gets auth, and 'internal-only' never counts as a boundary again.
>
> The lesson wasn't 'be more careful'. The lesson was that internal-only is an assumption, not a boundary. Assumptions rot; boundaries hold."

**中文意思**

1. 真正屬於我的失敗:**我交出了一個跨叢集的操作介面,而它沒有適當的身分驗證與授權。** 它是內部限定的——**所以我把網路邊界當成了安全邊界。**
2. 它後來浮上來,成為一個嚴重漏洞——**CVSS 10.0**。技術細節我不能分享,因為那個 CVE 還在禁令期內,但**設計上的教訓完全是我要承擔的。**
3. **修法也是我自己設計的**:縱深防禦——傳輸層的雙向 TLS、每一次呼叫都要過應用層的驗證與授權、以及跨叢集操作限定 localhost 的存取控制。
4. 之後有兩件事在**機制上**改了。**安全審查搬進設計階段,不再是事後補上去的東西。** 而我個人的預設值翻轉了:**每一個介面都要有 auth,而「內部限定」再也不算一條邊界。**
5. 教訓不是「要更小心」。**教訓是:「內部限定」是一個假設,不是一條邊界。假設會腐爛;邊界會守住。**

> 唸法:CVSS 10.0 唸 "CVSS ten point zero";authn/authz 直接唸 "authentication and authorization"(唸縮寫反而卡嘴)。
> 被追 CVE 細節 → "still under embargo, so I'll stay at the design level."——**這句本身就是專業訊號。**

---

## 8. Proudest project

**中文**:你最自豪的專案 / 最大的技術成就。

> "The two-node H-A replication layer. And what I'm proud of isn't the final design — it's that every version was forced by a real failure rather than a whiteboard.
>
> Version one used shared NFS, and the file locks turned out unreliable after a link flap. Version two moved to local command queues and exposed a loss window when the queue drained. Version three backs up locally before dispatch and only clears it after the peer confirms.
>
> The design got its shape from the failures."

**中文意思**

1. 那個雙節點 HA 複製層。而**我自豪的不是最終設計——是每一個版本都被一次真實的失敗逼出來的,不是在白板上想出來的。**
2. v1 用共享 NFS,結果連線抖動之後檔案鎖變得不可靠。v2 改成本地命令佇列,暴露出佇列被清空時的遺失視窗。v3 在派送前先本地備份,而且只有對方確認之後才清掉。
3. **這個設計的形狀是那些失敗給的。**

---

## 9. How do you handle ambiguous requirements?

**中文**:你怎麼處理模糊的需求?(**直接引用他們自己面試官的回饋——這是最強的證據**)

> "The best evidence I have is your own interviewer's feedback. In the first round I got a DMA dispatch problem — a domain I had never touched. I clarified it from zero: what the hardware guarantees, what happens on cancellation, what the ordering contract is. The interviewer called the clarifying out as a strength.
>
> That's how I think about it: **ambiguity is a questioning problem, not a knowledge problem.**"

**中文意思**

1. 我手上最好的證據是**你們自己面試官的回饋**。第一輪我拿到一題 DMA 派送的問題——**一個我從來沒碰過的領域。** 我從零開始把它問清楚:硬體保證什麼、取消的時候會發生什麼、順序的合約是什麼。**那位面試官特別點名 clarify 是我的強項。**
2. 我就是這樣看這件事的:**模糊是一個「提問」的問題,不是「知識」的問題。**

---

## 10. Tell me about a time you had to learn something quickly.

**中文**:講一次你必須很快學會某件事。

> "I'd never trained a model. We needed a spam classifier, so I fine-tuned a multilingual BERT variant and stood up my own inference server — ONNX runtime, plus llama.cpp for part of it. It went to production at about ninety-seven percent precision and ninety-nine percent recall.
>
> The way I learn fast is to build the smallest end-to-end thing that actually runs, and then make it good — instead of reading until I feel ready."

**中文意思**

1. **我從來沒訓練過模型。** 我們需要一個垃圾信分類器,所以我 fine-tune 了一個多語言的 BERT 變體,並且**自己架起 inference server**——ONNX runtime,其中一部分用 llama.cpp。**上線時大約是 97% precision、99% recall。**
2. 我學得快的方法是:**先做出一個最小、但真的跑得起來的端到端版本,然後再把它做好——而不是一直讀到自己覺得準備好了。**

> 備選:為了修隔壁團隊的 Rust 服務,兩週內從「讀者」變成他們的 go-to reviewer。

---

## 11. ★ 你要反問他們的三個問題

**中文**:debrief 只有 15 分鐘,挑 2–3 個問,不要全唸。

**問團隊**

> "What's the boundary of the Taiwan team today — who owns the spec, who sets priority, and where's the interface with HQ?"

中文:台灣團隊現在的邊界在哪裡——誰擁有 spec、誰定優先順序、跟總部的介面在哪?

**問技術**

> "Where does software hurt the most right now — the runtime, the driver layer, or tooling?"

中文:現在軟體最痛的地方在哪一層——runtime、driver 層,還是工具鏈?

**問成長**

> "Six months in, what does 'this hire worked out great' look like for this seat?"

中文:進來六個月後,對這個位子來說「這個人招得很成功」長什麼樣子?

**問結果時程(debrief 一定要問)**

> "What are the next steps, and when should I expect to hear back?"

中文:接下來的流程是什麼,我大概什麼時候會收到消息?
