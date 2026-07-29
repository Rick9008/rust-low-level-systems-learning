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

## 我的稿(2026-07-29 起草;bullet 骨架 + 關鍵句逐字,晚上出聲唸)

### 1. ★ Tell me about yourself(60–90s)

- Systems / infrastructure engineer at **Synology's mail platform** since 2023 — Rust, Go, modern C++ on Linux.
- Three things I do daily: **a two-node Active-Active HA layer**(conflict resolution when the network partitions)、**a C++ telemetry/logging daemon**(Asio event loop, many producers → multiple sinks)、**Rust microservices on Tokio**(real-time content inspection, ~3,500 msg/s per node)。
- 一句人設逐字:"**My job, in one sentence: keep systems predictable when the network is unreliable, the load spikes, and a node dies.**"
- Outside work: build systems from scratch to understand them — a Redis-compatible KV engine in Rust, an async runtime deep-dive.
- 收尾鉤到 why-Etched:"That's why a company doing hardware-software co-design is exactly where I want to be — the interesting failures live at that boundary."

### 2. ★ Why Etched? Why leave?(不講負面)

- **Toward, not away**(骨架):現職學到很多,但成長天花板在「軟體只能貼到 kernel 為止」。
- "Etched is making the kind of bet where **systems software decides whether the silicon actually delivers** — schedulers, event loops, telemetry, the runtime between the model and the chip. That software layer is what I already do every day; I want the other side of the API to be real hardware."
- 逐字:"**I'm not leaving something broken — I'm running toward the layer I've been trying to get closer to for years.**"
- 證據句:daily work = event loops(Asio)、backpressure、binary protocols、latency budgets——"the vocabulary transfers one-to-one."

### 3. ★ Work-life balance(陷阱題:兩個極端都扣分)

- **誠實**:"I have my own rhythm — I protect sleep, because tired engineers write outages."
- **證據**(這一個月就是實例):"When something matters I can sustain real intensity: this past month I ran a full-time job plus a structured interview-prep schedule every night — and the way I made that sustainable was **scheduling and a hard sleep red-line, not willpower**."
- **邊界**:"Sustainable pace is what protects output quality; sprints are fine when they matter, as long as they're sprints and not the steady state."
- 一句收尾逐字:"**I manage energy like I manage capacity in a system: leave headroom, or the tail latency gets you.**"

### 4. How do you feel about working across timezones with a US team?(bullet 骨架)

- 主動給方案:Taipei morning = US West afternoon → **fixed overlap window for sync**;everything else async with written docs. "Async-first actually forces better design docs — I've seen that work."

### 5. Startup pace vs big company process?(bullet 骨架)

- 用實例不用形容詞:two-node HA 的衝突規則沒有現成答案 → 自己定義、自己驗證、自己扛結果——"that's startup-shaped work inside a bigger company. I liked that part best."

### 6. ★ Conflict / disagreement(SAR;收在「用資料或失敗場景收斂,不是誰說服誰」)

- **S**:split-brain 衝突解法,團隊(包括我)最初傾向 **Last-Write-Wins** with timestamps——最直觀。
- **A**:I pushed back **with a concrete failure scenario, not an opinion**:a partition where node A approves and the mail is *actually sent*, node B later rejects — "**you cannot recall a sent email by overwriting a row.**" 加上兩節點沒有可信時鐘。
- **R**:改成從**操作語意**推規則(irreversible actions win);上線 6+ 個月零不一致事故。
- 方法論收尾逐字:"**We didn't converge because someone argued better — we converged because a failure scenario made the answer obvious. That's how I like to disagree: bring the counterexample, not the volume.**"(⚠ 確認:當時實際的討論對象/場景,講之前把人物補真)

### 7. ★ Failure(要有具體損失 + 機制性改變)

- **候選 A(預設)——v2 遺失視窗**:"I shipped a dispatch design where an operation could be **silently lost** if the queue was drained or the consumer restarted before the peer persisted it. That's data loss in a mail-security product — the worst kind of quiet failure."(⚠ 確認:當時怎麼發現的——測試?線上?)
- 機制性改變:"Two changes came out of it: the design fix — **never discard a message before the peer confirms durability**(backup-before-dispatch)——and the process fix: **design reviews now start from crash points**: for every hand-off we ask 'what if the process dies right here?'"
- 收尾逐字:"**The lesson wasn't 'be more careful' — it was 'make the failure impossible by construction'.**"
- 候選 B(如果確認項 4 成立,CPU-spin 是自己的 code):0-byte read 沒當 EOF → 迴圈終止條件對照 syscall 契約 + 之後的 code review checklist。

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
