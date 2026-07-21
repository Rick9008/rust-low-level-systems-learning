# ws_deque 設計取捨(Chase–Lev work-stealing deque,教學版)

對應程式碼:`reference/src/concurrency/ws_deque/`(`mod.rs` 教學殼 + `core_impl.rs` 演算法)。
前置閱讀:[signal_pipeline](signal_pipeline.md)(SB litmus 與 SeqCst fence——同一帖藥)。
定位:**deep-dive 讀物**——讀懂能講,不排手搓。tokio/rayon 的 per-worker
run queue 就是這一格。

## 設計哲學:不是消滅競爭,是把競爭趕到冷路徑

全局共享佇列(mpmc_ring)每個 op 都在搶同一條 tail cache line。
work-stealing 反過來:每個 worker 有自己的 deque,owner 在 bottom 端
LIFO push/pop(**無 CAS 快路徑**),別人只有在自己沒活幹時才來 top 端
FIFO 偷。負載平衡時零競爭;失衡才付 CAS——把同步成本移到理論上少見的
路徑上。LIFO×FIFO 也不是隨便選:LIFO 吃 cache 熱度(剛 spawn 的 task
資料還在 L1),FIFO 偷走最舊、最可能已冷的任務,兩端天然錯開。

## SB litmus 就在正中央(本 repo 第二個非 SeqCst 不可的位置)

唯一的戰場是**最後一件**:owner pop 與 stealer 同時指向同一槽。
owner pop「先降 bottom、再讀 top」;steal「先讀 top、再讀 bottom」——
兩邊都是「先寫自己、再讀對方」,教科書 store-buffering 形狀。
Acquire/Release 擋不住(兩邊的 store 都可能停在 store buffer),
必須一對 `fence(SeqCst)`;決鬥本身用 top 上的 SeqCst CAS 裁決,
誰贏誰拿走。與 `signal_pipeline` 掛牌握手同一個 litmus、同一帖藥。

## loom 抓到的那一課:bottom 的 store 為什麼全升 Release

論文版(Lê et al. 2013)pop 降 bottom 用 Relaxed store,正確性靠雙
SC fence 的整體證明。本版第一稿照抄——**loom 當場打爆**:stealer 的
Acquire 讀到那筆 Relaxed 降值時,與 push 的 Release 鏈完全沒接上,
槽位寫入可以不可見 → 偷到 null。教學版的裁決:bottom 每筆 store 都
Release(x86 上免費),讓「讀到 bottom=b ⇒ 看得見 [t,b) 的槽位寫入」
成為局部可讀的不變量,loom 可證。這整段是「窮舉工具 > 紙上證明直覺」
的實錄,面試講出來就是親手驗過的證據。

## 教學版兩個簡化

1. **固定容量**(滿了 Err):工業版 buffer 會長,舊 buffer 的回收靠 epoch。
2. **值裝箱**(槽位是 `AtomicPtr<T>`):教科書版 inline 值的「先讀、
   輸了 CAS 再丟」在槽位跨圈重寫的極端交錯下是**正式的資料競爭**
   (論文承認、crossbeam 靠 epoch 處理)。裝箱讓偷看變成原子指標 load,
   UB 消失——代價是每 push 一次配置(~20–50ns)。
   「怎麼安全地偷看一個可能正被覆寫的槽」正是 reclamation 問題的另一張臉。

## `Steal::Retry ≠ Empty`

輸了決鬥必須重試,當作空會漏工作——與 `mpsc_list` 的 Inconsistent
同哲學:把「不確定」顯式交給 caller(runtime 知道該 yield 還是先幹別的)。

## 選型帳

| 需求 | 選擇 |
|---|---|
| 多 worker 各自產出、偶爾平衡 | **ws_deque**(per-worker)——tokio/rayon 的答案 |
| 所有 producer 餵同一群 consumer | [mpmc_ring](mpmc_ring.md)(全局共享)|
| 單一 worker、跨執行緒只有 wake | [mpsc_list](mpsc_list.md) + doorbell 就夠 |
