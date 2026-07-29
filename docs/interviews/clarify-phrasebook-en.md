# Clarify 英文句庫(問對問題 + 聽不懂時的修復句)

用法:不背整句,背**粗體骨架**;每晚出聲場挑一類唸 3 遍。跑 sim 時 clarify 必須用英文打字,句子從這裡拿。

## 0. 開場定界(拿到題先講,≤30 秒)

- "Before I code, let me make sure I understand the setup: **we have [N resources], requests arrive via [API], and my job is to [goal]. Is that right?**"
- "I'll ask a few questions first, then walk you through my plan before writing code."

## 1. 並行模型(最高價值,R1 那題的靈魂)

- "**Do requests need to be handled concurrently, or is it acceptable to fully process one request before pulling the next?**"(← 接 request 跟處理 request 要不要同步進行)
- "Can a second request arrive while the first is still in flight — and if so, is it expected to make progress, or may it wait?"
- "Is this single-threaded event-loop style, or am I expected to spawn threads?"

## 2. API 語意(每個給的 API 至少掃一遍)

- "**Does `get_X()` returning `None` mean 'nothing right now' or 'stream closed'?**"
- "Does `wait_event()` guarantee something is ready when it returns, or can it wake spuriously?"
- "When `get_dma_result_done()` gives me an engine id, does it tell me **which block or request** that was? Or do I track that myself?"
- "Is this API safe to call from [ISR context / another thread], or only from the main loop?"
- "Is `block_start_pos` in **bytes or in blocks**?"(單位題,每題必掃)

## 3. 順序(ordering)

- "Do completions come back **in the order I submitted them**, or can they be out of order?"
- "Is there any ordering requirement **between requests**? **Within one request's blocks**?"

## 4. 滿了/掉了(backpressure vs drop)

- "**What should happen when the queue is full — block the producer, or drop?** If we drop, drop the oldest or the newest?"
- "Can the upstream be pushed back on, or does data keep coming regardless?"(硬體 = 推不回去)
- "Do we need to count or report drops?"

## 5. 失敗模式

- "**Can an engine fail or hang?** What should happen to its in-flight work?"
- "Is re-executing a block **safe (idempotent)**, or could a retry corrupt data?"
- "If something goes irrecoverably wrong, who do I report it to — is there an error path upstream?"

## 6. 規模與 SLA(數字要反過來消滅設計選項)

- "**Roughly how many requests per second** should this sustain? That tells me whether a simple mutex is fine or we need lock-free."
- "Is the latency target about the **average or the tail** — do we care about p99?"
- "How many [sensors/cores/engines] at most? That decides a Vec of slots vs a HashMap."

## 7. 複述收口(clarify 結束、動手前)

- "**Let me restate to make sure I've got it:** [3 句摘要]. Did I miss anything?"
- "My plan: [state 表 + loop 骨架一句話]. **Does that match what you expect before I start coding?**"

## 8. 聽力修復句(聽不懂/不確定時,絕不裝懂)

- "Sorry, **could you say that again**? The audio cut out a bit."
- "Could you say that **more slowly / in other words**?"
- "**Just to confirm, you said** [my paraphrase] — right?"
- "I'm not familiar with the term **[X]** — could you describe what it does?"
- "**When you say [X], do you mean [A] or [B]?**"(把模糊詞逼成二選一,最好用的一句)

## 9. 邊寫邊講(narrate 潤滑句)

- "Let me think out loud for a second."
- "I'll start with the simple version and tighten it after it works."
- "I know this line is subtle — let me explain why the order matters here."
- "I'm running short on time, so let me describe what's left and where the holes are."(R1 的教訓:時間不夠時主動把洞講出來,比被抓到強)
