# Lock-free 追問日 + b#1 四洞(2026-07-22 Q&A 沉澱)

> 完整圖解頁:`docs/artifacts/qa_lockfree_followups.html`(晨間九題)、
> `docs/artifacts/signal_pipeline.html` ②(fence 全套:真 code、x86 表、timeline、六題 FAQ)
> loom 證物:`reference/tests/loom_lost_wakeup.rs`(三種通知擺法的裁決)

## 一句話骨架(今日主題)

**「綠」有三層假象:綠測試只證明「跑過的那幾種排程」;修過 ≠ 有網(mutation 才驗得出);被 `let _ = join()` 吞掉的 panic,任何測試都看不見。**

---

## 卡 1:MPSC/MPMC 光譜(30 秒口述)

| | push 端 index | pop 端 index | per-slot seq |
|---|---|---|---|
| SPSC ring | 單寫者:atomic store | 單寫者:atomic store | 不用(bump 兼佔位+發布) |
| MPSC ring | 多寫者:**CAS** | 單寫者:**非原子私有欄位** | 要 |
| MPMC ring | 多寫者:CAS | 多寫者:CAS | 要 |

- 規則:**哪端是單,那端 index 保持單寫者,同步整格降級;push 端變多,seq 逃不掉。**
- 兩軸判準:寫者數 → CAS vs store;**他端讀者數 → atomic vs 普通欄位**(單寫者 ≠ 私有!SPSC 的 head 有對面讀者,必須 atomic)
- MPSC list(swap)vs ring(CAS):unbounded+wait-free push+每筆配置+Inconsistent 縫 vs bounded+零配置+CAS 重試

## 卡 2:seq = 槽位的時鐘

- 三態:`seq==pos` 空可搶 / `pos+1` 可讀 / `pos+cap` 等下一圈——**一個數字編碼狀態+圈數**
- 為什麼不能歸零:下一圈客人票號是 pos+cap,歸零會算出 dif=−cap → 永遠「滿」
- `dif<0` 判滿的單調性證明:seq 只增;票號 pos 被領走的前提是有人看過 seq==pos;我讀到更小值 ⇒ 票沒被領 ⇒ **快照必新鮮**
- 計數器不繞 ring(`& mask` 投影),只繞 2⁶⁴(wrapping 差值,serial number arithmetic)

## 卡 3:時代數統一(一招三用)

| 結構 | 位置被重用 | 時代數防什麼 |
|---|---|---|
| mpmc_ring `seq` | 槽位每圈 | 舊圈的 P/C 誤動槽位 |
| arena `gen`(pack 進 CAS 字) | free 鏈節點 | **ABA**:舊快照 CAS 誤成功 |
| fd_registry `generation` | kernel 重發 fd | **stale token** 打到新連線 |

**位置會重用 → 光比位置不夠 → 配時代數;舊時代的人做什麼都被拒。**

## 卡 4:swap vs CAS vs fence

- **swap 必成功**(無條件交換,回舊值)→ wait-free;**CAS 可失敗**(「還是 expected 才換」)→ 重試迴圈 = lock-free
- 原語跟佔位語意走:list 掛鏈尾無條件 → swap;ring 領票有前提 → CAS
- **CAS 決定「誰贏」(仲裁一個變數);fence 決定「誰先被看見」(排序前後所有操作)**
- M-S 的 help:懸空步(推路標)資訊公開人人可代勞 → 正式 lock-free;Vyukov 懸空步(寫 payload)無人能代勞 → lockless。**分水嶺:懸空的工作別人幫不幫得了**

## 卡 5:fence(SB litmus)

- 標籤只管掛標籤那筆;fence 管前後**所有**操作 + SeqCst fence 有全機全序
- x86 對照:`load(SeqCst)` = 普通 `mov`(**不沖 buffer,標籤不是魔法**);`store(SeqCst)` = `xchg`;`fence(SeqCst)` = `mfence`
- SB 形狀:兩邊都「先寫自己、再讀對方」;store 悶在各自 store buffer → 兩筆 load 都讀到舊值 → 互相錯過
- 兩邊插 fence ⇒ fence 有全序必有先後 ⇒ **至少一邊看見另一邊**;柵欄不是推播,是禁止「雙盲」這種結局
- 重排表:acq 殺讀側、rel 殺寫側、**Store→Load 只有 SeqCst 殺得掉**
- 危險不對稱:掛牌(true)攸關 liveness → SeqCst+fence;摘牌(false)最多多叫一聲 → Release 是紀律不是必要

## 卡 6:condvar lost-wakeup(b#1 洞⑤)

- 窗:「檢查完 predicate、正要睡、還沒睡著」;notify 只叫得醒**已睡著**的人
- 關窗兩解(loom 三變體裁決,`loom_lost_wakeup.rs`):
  - **store 進鎖**(教科書):waiter 檢查期間,牌都寫不了
  - **notify 進鎖**(b#1 實採):牌隨時可寫,但**鈴要排隊**;醒來重拿鎖時 mutex 順便發布 store
  - 兩者皆綠;全不拿鎖 → loom `deadlock; threads=[Blocked, Blocked]`
- 今天 hang 的教訓:**condvar 醒來的人要重新拿回 mutex 才走得出 wait**——拿著鎖 join = 互等死鎖
- 綠測試殺不死這 bug(奈秒窗);武器只有紙上分析或 loom

## 卡 7:b#1 四洞總帳(pool_graceful_shutdown)

1. **洞④** pop unwrap panic:被 shutdown 叫醒時佇列可能空,盲 `unwrap` → panic 抱鎖死 → 毒鎖連環爆;**全被 `let _ = join()` 吞掉**(oracle 綠著)。修:`None => continue` + `join().expect()` 讓屍體浮上來
2. **洞⑤** lost-wakeup:store+notify 全程不拿 jobs 鎖。修:notify 進鎖(loom 證綠)
3. **洞⑥** 鎖圈住 job 執行:worker 拿著 guard 跑 job → 整個 pool 串行。證據:40 job×10ms÷4 worker 應 0.1s,實測 0.4s。修:pop 完 `drop(guard)` 再跑,**時間 0.40→0.10s**
4. e2 複核同款教訓:**修過 ≠ 有網**——7/21「紅測未先行」的債,mutation(改壞→看紅)才驗得出;e2 兩洞當時零網,今日補紅測×2

## 卡 8:spin vs park vs batch(吞吐的兩本帳)

- 表格那格「~µs、每次喚醒 syscall」是**單次帳**;「吞吐型」是**每秒總產出帳**——忙碌系統喚醒是稀有事件,µs 攤到 ~0;spin 的核整顆燒掉
- 判準:**工作粒度 vs 喚醒成本的比值**。粒度 100µs vs 2µs → park;粒度 100ns(封包)→ 2µs 是工作 20 倍 → spin(DPDK busy-poll:專屬核無限迴圈問網卡,不睡不 syscall)或 batch
- batch 兩端各攢一半:**producer 攢通知**(空→非空才 notify;eventfd counter 合併)+ **consumer 攢處理**(醒一次吃到空)
- park 要 SeqCst 的真因:不是多寫者,是**兩邊都「先寫自己、再讀對方」**(SB!)——condvar 靠 mutex 擋,park-token 靠 SeqCst fence 擋。同病兩藥

## 今日產出帳(白天)

- lru 兩洞修(紅測×2 + 單獨 unlink 直測×2 + mutation 複驗)commit a77aa44
- spsc 空白 #2:**10 分寫完(限 20)、首編 4 錯全手滑、概念零傷**(#1:35 錯超時);三處穿線規則:包裝型別 = 宣告/建構/存取三處都要出現
- e2#1 複核:兩洞皆無網 → 補紅測×2 先紅後綠
- b#1:自寫紅測×3(mutation 驗咬人)+ 四洞全修(0.10s)
- 沉澱:qa_lockfree_followups.html(九題+stepper 導覽)、signal_pipeline.html 大改(gruvbox-material+fence 全套)、loom_lost_wakeup.rs、telemetry_aggregator drill
- SCHEDULE v9:白天=打字場 4h+/晚上 23:30–02:00=出聲場;卡片實估 30–45m;02:00 熄燈(7/26 起提前)
