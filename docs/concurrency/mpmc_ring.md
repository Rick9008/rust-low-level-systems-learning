# mpmc_ring 設計取捨

對應程式碼:`reference/src/concurrency/mpmc_ring/`(`mod.rs` 教學殼 + `core_impl.rs` 演算法)。
前置閱讀:[spsc_ring](spsc_ring.md)(本文假設你已懂自由跑計數器與 2 的冪 mask)。
互動教材:`html_p/lockfree-queue-family-handbook.html`(Vyukov 章 + 對撞 stepper)。

## 從 SPSC 到 MPMC 只有兩刀,但第二刀是本質的

**第一刀:佔位改 CAS。** SPSC 的 tail 單一寫者,load+store 就能推進;
多 producer 下兩人會讀到同一個 tail、寫同一個槽。所以寫槽之前必須先
「搶到號」——CAS(或 fetch_add)取號。這刀是機械的,誰都想得到。

**第二刀:發布訊號搬家。** SPSC 裡「bump tail」一個動作身兼佔位與發布
(資料先寫、tail 後動,consumer 看 tail 就知道能讀到哪)。佔位被迫先行後,
「號進了、資料還沒進」出現一道**縫**——tail 不再能當發布訊號。
Vyukov 的答案:每個槽位自帶一個 `seq`,對邏輯位置 pos 是三態狀態機:

| seq 值 | 狀態 | 誰在等它 |
|---|---|---|
| `pos` | 本圈輪空 | producer(dif==0 才 CAS) |
| `pos + 1` | 已發布 | consumer(dif==0 才 CAS) |
| `pos + cap` | 已釋放 | 下一圈的 producer(它的 pos' = pos+cap) |

happens-before 全部掛在 seq 的 Release→Acquire 上;head/tail 退化成
純取號機,**CAS 用 Relaxed 就夠**。SPSC 那兩條「Acquire 讀對方 index」的
同步邊整個消失——head 與 tail 甚至互不比較。這是讀 crossbeam/Vyukov
原始碼時最反直覺的一點,面試講出來就是超綱訊號。

## cap ≥ 2 不是慣例,是正確性

「已發布」= pos+1、「下一圈輪空」= pos+cap;cap=1 時兩者同值,
producer 分不出「滿」跟「可搶」,會直接覆寫未消費的資料。
原版 Vyukov assert cap≥2,本實作在 `new` 裡上調。這個退化是寫
hand-trace 測試時當場踩到的——dry-run 的價值展示。

## seq 自帶圈數 ⇒ ABA 免疫

CAS 的經典盲點是 ABA(值繞一圈回來,CAS 誤判沒人動過)。這裡 seq 每圈
+cap、單調遞增,舊圈的值永遠追不上新期望——與自由跑計數器同一招:
**用單調性買免疫,代價是想清楚溢位 wrap**(`wrapping_sub` 轉 isize,
|dif| ≤ cap 不會誤判)。

## 誠實邊界:lockless ≠ lock-free

producer 在「CAS 取號」與「seq 發布」之間被 deschedule,所有 consumer
就卡在那一格(dif<0 → 一直回 None)——沒有全系統進度保證,不滿足
lock-free 的正式定義。工程上通常無所謂(縫只有幾個指令寬),但面試
必須主動講:「這是 lockless,嚴格說不是 lock-free——要正式保證就上
Michael-Scott(unbounded、代價是 reclamation——教學版見 [mpmc_list](mpmc_list.md))」。

## 為什麼沒有 len()

MPMC 下 head/tail 都在動,任何 `tail-head` 都是「算出來那刻就過期」的
快照(見 ring_buffer 教材「len 之死」)。提供它只會誘導 caller 寫出
`if !q.is_empty() { q.pop() }` 這種 TOCTOU。要觀測深度,用 metrics
(採樣容忍過期)而不是 API。

## 退化表:為什麼沒有 mpsc_ring / spmc_ring 模組

| 佇列 | producer 端 | consumer 端 | 縫在哪 |
|---|---|---|---|
| spsc_ring | store | store | 無縫——index 本身就是發布訊號 |
| mpsc_ring | CAS + seq | store(讀 seq) | producer 側(佔位→發布) |
| spmc_ring | store | CAS + seq | consumer 側(佔位→**讀完**釋放) |
| mpmc_ring | CAS + seq | CAS + seq | 兩側都有 |

規則:**哪端是「多」,哪端就要 CAS 取號、就有縫、就需要 per-slot 訊號;
哪端是「單」,那端 index 保持單寫者、plain store 就夠。**
mpsc_ring = 本模組把 pop 的 CAS 換成 store,一行的參數化退化,不值一個模組。
SPMC 的隱藏坑值得口述:consumer CAS 搶到號 ≠ 讀完了——producer 只看
head 推進就覆寫會撕掉進行中的讀,所以「讀完」需要 per-slot 訊號
(seq 跳下一圈),縫對稱地搬到消費側。unbounded 的 MPSC 另有專屬解:
[mpsc_list](mpsc_list.md)(push 端 wait-free,tokio 的選擇)。

## 選型帳:什麼時候用它

| 情境 | 選擇 | 理由 |
|---|---|---|
| 端點各一 | spsc_ring | 問題退化,無 CAS 是效能上界 |
| 多 P 單 C、不能丟 | mpsc_list | push wait-free、unbounded |
| 多 P 多 C、容量要硬上限 | **mpmc_ring** | backpressure 內建(Err 歸還) |
| 吞吐要隨核數線性長 | 佇列之外找答案 | N×SPSC fan-in、per-core shard 或 [ws_deque](ws_deque.md)(per-worker)——單一 tail 的 cache line ping-pong 是物理上限 |
| 無競爭/低頻 | `Mutex<VecDeque>` | uncontended lock ~20ns,簡單贏 |

數字錨點(x86_64 量級):uncontended CAS ~20ns;兩核搶同一條 line 一次
所有權轉移 ~40–100ns;futex 陷核 ~1–2µs。「lock-free 一定比較快」是
面試陷阱——低競爭時 Mutex 常常更快,先量再換。
