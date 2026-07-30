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

### 3. ★ Work-life balance(陷阱題:兩個極端都扣分;~135 words ≈ 65s)

**逐字稿(直接唸)**

> "Honest answer: I have my own rhythm, and I protect sleep — tired engineers write outages.
>
> But when something matters, I can sustain real intensity. Concrete example: my teammate and I were driving our content-moderation feature to a hard demo deadline — a stretch of focused, genuinely high-intensity development. What made it sustainable was scheduling and a hard sleep red-line, not willpower — that's what kept day ten as sharp as day one. And we made the demo.
>
> Long-term, I side with something Jon Gjengset — a Rust systems educator I learn a lot from — has argued: consistent, focused hours out-produce heroic hours over anything longer than a sprint. Sprints are fine when they matter, as long as they're sprints and not the steady state.
>
> I manage energy the way I manage capacity in a system: leave headroom, or the tail latency gets you."

**唸法註(不唸)**:結構 = 誠實 → 證據(Moderation 衝刺)→ 論點背書(Gjengset)→ 系統比喻收尾。
⚠ **證據段三細節待補真(7/31 深夜換稿:原「面試準備」例退役——自我指涉+非工作產出)**:
① 衝刺時長("a stretch" 換真數字,如 "two weeks");② 結局補一句(demo 之後 feature 上線?);
③ 是否點名隊友("my teammate William and I" 比匿名更真,加分不是風險——你選)。
⚠ **Gjengset 引用的使用守則(7/31 加)**:上場前自己把那篇 40-hours 文章重讀一遍,確認你轉述的論點正確;
被追問文章內容就講論點、不掰細節;如果對面的「拼命文化」訊號很強,把人名句縮短成
"I optimize for sustained throughput, not heroics" ——論點不變,少一個可以被挑戰的引用。
引用的價值:證明你對產能的想法是讀過、想過的立場,不是怕加班的托詞。

### 4. How do you feel about working across timezones with a US team?(bullet 骨架)

- 主動給方案:Taipei morning = US West afternoon → **fixed overlap window for sync**;everything else async with written docs. "Async-first actually forces better design docs — I've seen that work."

### 5. Startup pace vs big company process?(bullet 骨架)

- 用實例不用形容詞:two-node HA 的衝突規則沒有現成答案 → 自己定義、自己驗證、自己扛結果——"that's startup-shaped work inside a bigger company. I liked that part best."

### 6. ★ Conflict / disagreement(SAR;~160 words ≈ 75s)

**逐字稿(直接唸)**

> "Sure — this happened on the two-node H-A project. When we designed the split-brain conflict resolution, the team's first instinct — mine included — was last-write-wins with timestamps. It's the intuitive answer.
>
> I pushed back, but with a concrete failure scenario rather than an opinion. Picture a partition where node A approves a message and the mail actually goes out — and node B later rejects it. You cannot recall a sent email by overwriting a row. And with exactly two nodes, there's no trusted clock to even say which write was 'later'.
>
> That scenario reframed the discussion. We ended up deriving the rules from operation semantics — irreversible actions win — and that design has now run in production for over six months with zero inconsistency incidents.
>
> We didn't converge because someone argued better — we converged because a failure scenario made the answer obvious. That's how I like to disagree: bring the counterexample, not the volume."

**唸法註(不唸)**:SAR 全在裡面(S=LWW 直覺、A=反例場景、R=語意規則+六個月零事故)。
⚠ **仍待補真**:當時實際的討論對象與場景(誰主張 LWW、在什麼會議)——上場前把人物填進第一段,
一句就夠:"my tech lead and I both started from..."。

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
