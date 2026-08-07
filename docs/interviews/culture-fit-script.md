# Culture fit 英文稿 + 模擬題庫

用途:每場開頭自介 + 各場 behavioral 提問 + 最後 15m recruiter debrief(onsite 沒有獨立 culture fit 場)。
練法:白天寫稿(每題 5–8 句就好,不背逐字稿,背骨架)→ 晚上出聲唸 → Claude 模擬追問。

## 題庫(★ = 必寫稿;其餘寫 bullet 骨架即可)

### 開場
1. ★ **Tell me about yourself.**(60–90s;收在「為什麼是 Etched 這種公司」)
2. ★ **Why Etched? Why leave your current company?**(對齊:hardware-software co-design、想貼著矽做系統;不要講負面離職原因)

### 工作型態(美國新創 × 台灣遠端,一定會探)
3. ★ **What are your thoughts on work-life balance?**
   - 陷阱題:答「我很重視 WLB」和「我沒有生活」都扣分。
   - 骨架:誠實(我有自己的節奏)+ 證據(衝刺期可以拉高強度——這一個月的面試準備就是實例:白天全職工作、晚上固定 2h 練到凌晨,靠的是排程不是意志力)+ 邊界(可持續才有品質,我用睡眠紅線保護輸出)。
4. **How do you feel about working across timezones with a US team?**(主動講 overlap 方案:台北早上 = 美西下午,會議窗口固定;其餘 async 文件溝通)
5. **Startup pace vs big company process — which do you prefer and why?**(用實例,不用形容詞)

### 行為題
6. ★ **Tell me about a conflict / disagreement with a teammate or manager.**(SAR;收在「怎麼用資料或實驗收斂,不是誰說服誰」)
7. ★ **Tell me about a failure.**(要有具體損失 + 學到的機制性改變,不是「我學到要更小心」)
8. **Proudest project / biggest technical achievement.**(可直接用 deep dive 專案一的 90 秒版)
9. **How do you handle ambiguous requirements?**(R1 現成例子:不熟 DMA 也能 clarify 出 spec——面試官點名稱讚,直接引用)
10. **Tell me about a time you had to learn something quickly.**

### 反問(準備 3 條,debrief 用)
11. 對 team:目前 Taiwan team 的角色邊界?跟 US HQ 的介面是什麼(誰定 spec、誰定priority)?
12. 對技術:軟體最痛的瓶頸現在在哪一層(runtime? driver? tooling)?
13. 對成長:六個月後怎麼定義這個位子做得好?

## 我的稿(2026-07-29 起草;**2026-07-31 三輪:★ 題升級全英文逐字稿**,唸的時候只唸引文區,中文全是「唸法註」不出聲)

> 兩層規格:**★ 題 = 逐字稿**(7/31 定案:骨架版實測唸不出來——句間的中文接縫會卡住嘴);
> 其餘題保持 bullet 骨架(它們是低頻選配,臨場組句即可)。

### 1. ★ Tell me about yourself(60–90s;~170 words ≈ 80s)

**逐字稿(直接唸)**

> "I'm a systems and infrastructure engineer on Synology's mail platform — I've been there since twenty twenty-three, working in Rust, Go, and modern C++ on Linux.
>
> Three things fill my typical day. First, a two-node active-active high-availability layer — I built the state replication that keeps human approval decisions consistent when the network between the nodes partitions. Second, a C++ telemetry and logging daemon on an Asio event loop — many producers feeding one structured sink. And third, Rust microservices on Tokio doing real-time content inspection, at about thirty-five hundred messages a second per node.
>
> My job, in one sentence: keep systems predictable when the network is unreliable, the load spikes, and a node dies. And 'predictable' includes fast — tail latency is part of the contract.
>
> Outside work, I build systems from scratch to understand them — a Redis-compatible key-value engine in Rust, and a deep dive into how async runtimes actually work.
>
> And that's why a company doing hardware-software co-design is exactly where I want to be — the interesting failures live at that boundary."

**唸法註(不唸)**:數字唸法——2023 = "twenty twenty-three"、3,500 = "thirty-five hundred"。
第二段是舊稿的「三件日常」條列展開成的句子:每件 = 名字 + 我做了什麼 + 一個規格詞(partitions / structured sink / 3,500 msg/s)。
最後一句就是 why-Etched 的鉤子,面試官十之八九順著它問——正中下懷。

### 2. ★ Why Etched? Why leave?(~140 words ≈ 70s;不講負面;7/31 深夜四輪改版)

**逐字稿(直接唸)**

> "For me this is a toward move, not an away move. I've learned a lot where I am — but the ceiling is that software stops at the kernel; the hardware underneath is always someone else's black box.
>
> Etched is making a bet I love: the chip sets the ceiling, and the systems software decides how close you get to it — schedulers, event loops, telemetry, the runtime between the model and the chip. That layer gets built from scratch, and it's performance-critical — which is exactly what I chase. My daily vocabulary — event loops, backpressure, binary protocols, latency budgets — transfers one to one. And the from-scratch part: today that's what I do on my own time; I want it to be the day job.
>
> So no — I'm not leaving something broken. I'm running toward the layer I've been trying to get closer to for years."

**唸法註(不唸)**:骨架 = toward-not-away(kernel 天花板)→ 晶片天花板/軟體貼多近 → from-scratch + 效能是我在追的 → 詞彙一比一 → 嗜好變正職 → 收尾金句。
「晶片定天花板、軟體定貼多近」是**產業通則**(utilization gap;CUDA 護城河同理),不需要 Etched 內部知識,被追問也站得住——7/31 換掉原本的 "software decides whether the silicon delivers"(斷言了他們內部現實,你不確定)。
兩個天花板互相呼應:第一段我的天花板是 kernel,第二段晶片的天花板由軟體逼近。
被追問「離職原因」負面細節時,回到第一句重申 toward,不展開任何抱怨。

### 3. ★ Work-life balance(陷阱題:兩個極端都扣分;8/7 補真後 ~175 words ≈ 80s)

**逐字稿(直接唸)**

> "Honest answer: I have my own rhythm, and I protect sleep — tired engineers write outages.
>
> But when something matters, I can sustain real intensity. William and I had two weeks to get our content-moderation feature to a hard demo deadline. We paced most of it deliberately — and then on the last Friday we paired straight through the night and went out for breakfast on Saturday morning. That happened once, at the end, and it was a decision — we could only afford it because the rest of the two weeks wasn't run that way. We made the demo, and that feature has been in production for over a year now.
>
> Long-term, I side with something Jon Gjengset — a Rust systems educator I learn a lot from — has argued: consistent, focused hours out-produce heroic hours over anything longer than a sprint. Sprints are fine when they matter, as long as they're sprints and not the steady state.
>
> I manage energy the way I manage capacity in a system: leave headroom, and know when to spend it."

**唸法註(不唸)**:結構 = 誠實 → 證據(Moderation 衝刺)→ 論點背書(Gjengset)→ 系統比喻收尾。
✅ **三細節已補真(8/7)**:① 兩週;② feature 上線已一年多(結局句進稿尾,回答「衝刺值不值得」);
③ 點名 William(你自己提的,留著=真實感)。
⚠ **8/7 訊息改寫(必讀,不是潤稿)**:原稿賣「hard sleep red-line + day ten as sharp as day one」,
與真實情節(最後週五通宵到週六早餐)**直接對撞**——面試官一問就破。改成「兩週只有最後一晚、
刻意選的、撐得住正因為前面沒那樣操」:強度訊號與節制訊號同時在,且句句是真。
收尾金句同步改("leave headroom, **and know when to spend it**"),因為新敘事就是「留餘裕→在關鍵處花掉」;
原本的 "or the tail latency gets you" 與通宵並存會刺耳,已退役。
**追問防禦**:問「所以你會通宵?」→「兩週衝刺的最後一晚,一次,是決定不是習慣。我不做的是把那個當常態——那才是出事的做法。」
⚠ **Gjengset 引用的使用守則(7/31 加)**:上場前自己把那篇 40-hours 文章重讀一遍,確認你轉述的論點正確;
被追問文章內容就講論點、不掰細節;如果對面的「拼命文化」訊號很強,把人名句縮短成
"I optimize for sustained throughput, not heroics" ——論點不變,少一個可以被挑戰的引用。
引用的價值:證明你對產能的想法是讀過、想過的立場,不是怕加班的托詞。

### 4. How do you feel about working across timezones with a US team?(bullet 骨架)

- 主動給方案:Taipei morning = US West afternoon → **fixed overlap window for sync**;everything else async with written docs. "Async-first actually forces better design docs — I've seen that work."

### 5. Startup pace vs big company process?(bullet 骨架)

- 用實例不用形容詞:two-node HA 的衝突規則沒有現成答案 → 自己定義、自己驗證、自己扛結果——"that's startup-shaped work inside a bigger company. I liked that part best."

### 6. ★ Conflict / disagreement(SAR;8/7 補真後 ~215 words ≈ 100s,砍兩處可回 ~190 words ≈ 90s)

**逐字稿(直接唸)**

> "Sure — on the two-node H-A project. I'd drawn the architecture for the decision-replication layer and brought it to a review with my manager, our tech lead, and a junior teammate. The tech lead proposed something lighter: put the decision on a mail flag and let dsync — the mail store's own replication — carry it across. No new system to build.
>
> That's the right question to ask, so I took it seriously, and I tested it against a failure scenario instead of arguing preference. Flags replicate as state, and state replication settles conflicts by whoever wrote last. Partition: node A approves, the mail actually goes out; node B rejects. If the reject lands last, you've un-sent an email that is already in someone's inbox. And with exactly two nodes, there's no trusted clock to even say which one was later.
>
> That reframed the discussion. The gap was never data replication — the platform already had three of those. It was that nobody replicated what the application had *decided*. So we built the operation log, and derived the conflict rules from operation semantics: irreversible actions win. Six-plus months in production, zero inconsistency incidents.
>
> We didn't converge because someone argued better — a failure scenario made the answer obvious. That's how I like to disagree: bring the counterexample, not the volume."

**唸法註(不唸)**:SAR 全在裡面(S=tech lead 提 dsync 捷徑、A=反例場景、R=語意規則+六個月零事故)。
✅ **人物與場景已補真(8/7)**:場合 = **你把架構畫完後,自己召集 manager / staff(tech lead)/ junior 的
架構 review**;提案人 = **staff 同事(tech lead)**,主張「用 mail flag 操作 dsync 幫我們同步,就不必自己
做一個新系統」。這比原稿的抽象版強:場合、人、提案都講得出來,追問不會空。
**為什麼原稿的 "last-write-wins with timestamps" 換成 dsync 版**:兩者不衝突——**靠 flag 讓 dsync 同步,
本質就是狀態複製,而狀態複製的勝負規則就是「誰最後寫」**。原稿講的是這件事的抽象名字,新稿講的是
當時真正被提出的具體方案。技術結論一字未改(7/29 你回填的理由本來就是「複製決定不了先後順序、
也不知道誰的操作不可挽回」),只是把它接回真實現場。
⚠ **追問防禦**:① dsync 一句話定義備著——"Dovecot's mailbox replication tool; it syncs messages and
their flags between servers."**被追問內部細節就停在這句,不掰**。② 政治安全:全段給 tech lead credit
("that's the right question to ask")——這題考的是你怎麼不同意,**不是「我糾正了我的 lead」**,語氣一歪就扣分。
③ 趕時間時可砍兩處:"And with exactly two nodes…later."、"the platform already had three of those"。

### 7. ★ Failure(✅ authz 故事——真正自己寫出的 bug;~155 words ≈ 75s)

**逐字稿(直接唸)**

> "The failure that's genuinely mine: I shipped a cross-cluster operations interface without proper authentication and authorization. It was internal-only — so I treated the network boundary as the security boundary.
>
> It later surfaced as a critical vulnerability — CVSS ten point zero. I can't share the technical details because the CVE is still under embargo, but the design lesson is fully mine to own.
>
> I also designed the fix myself: defense in depth — mutual TLS at the transport layer, application-level authentication and authorization on every call, and localhost-scoped access control for the cross-cluster operations.
>
> Two things changed mechanically after that. Security review moved into the design phase instead of being a bolt-on. And my personal default flipped: every interface gets auth, and 'internal-only' never counts as a boundary again.
>
> The lesson wasn't 'be more careful'. The lesson was that internal-only is an assumption, not a boundary. Assumptions rot; boundaries hold."

**唸法註(不唸)**:CVSS 10.0 唸 "CVSS ten point zero";authn/authz 口語直接說 "authentication and authorization"
(縮寫唸出來反而卡)。被追 CVE 細節 → "still under embargo, so I'll stay at the design level"——這句本身是專業訊號。
備案 B:v2 遺失視窗(設計層 failure,主戰放 deep dive 專案一演進段)。CPU-spin 不用——前人程式碼,只當 debugging 故事。

### 8. Proudest project(bullet 骨架)

- 直接用 deep dive 專案一的 90 秒版(deep-dive-projects.md);一句加值:"proud not of the final design, but that **each version was forced by a real failure** — v1 NFS locks, v2 loss window, v3 backup-before-dispatch."

### 9. Ambiguous requirements(bullet 骨架;R1 現成)

- "Your own interviewer's feedback is my best evidence: in round 1 I'd never touched DMA dispatch — I clarified the domain from zero, pinned the spec, and the interviewer called out the clarify as a strength. **Ambiguity is a questioning problem, not a knowledge problem.**"

### 10. Learn something quickly(bullet 骨架)

- 選一:mmBERT spam 分類器 + 自建 ONNX/llama.cpp inference server——from "never trained a classifier" to **97% precision / 99% recall in production**;或:為了修隔壁團隊的 Rust 服務,兩週內從讀者變 go-to reviewer。

### 11. 反問三條(debrief 15m 用;挑 2–3 問,不全唸)

- Team:"What's the boundary of the Taiwan team today — who owns the spec, who sets priority, and where's the interface with HQ?"
- 技術:"Where does software hurt the most right now — the runtime, the driver layer, or tooling?"
- 成長:"Six months in, what does 'this hire worked out great' look like for this seat?"
