# spin_lock —— 為什麼是 TTAS + RAII guard

對應 `reference/src/concurrency/spin_lock.rs`(drills/challenges 有練習版)。

## 為什麼存在(何時選 spin,何時選 mutex)

臨界區奈秒級、或身處**不可睡眠的 context**(ISR、已持有另一把 spinlock)時,
`Mutex` 的 park/unpark 是 syscall(µs 級)——比臨界區本身貴一千倍,忙等反而便宜。
反過來,臨界區可能拉長(IO、alloc、可搶佔)就必須睡:自旋等一個被 OS deschedule
的持鎖者,是拿整顆核心陪葬。production 的折衷是 `parking_lot` 式混合:自旋幾輪
沒拿到就 park。cost model 錨:spin 一圈 ~ns,park/unpark ~µs,三個數量級。

## TAS vs TTAS:貴的不是等,是等的方式

- **TAS**(test-and-set 硬撞):每次失敗都是一次 RMW。RMW 需要 cache line
  **獨占權**(MESI 的 M/E 狀態),N 個等待者輪流把這條 line 搶來搶去,
  匯流排流量 O(等待者數),持鎖者想放鎖還得跟他們搶。
- **TTAS**:外圈 `swap` 搶鎖;搶不到進內圈**純 load** 等待——讀取讓 line 停在
  Shared,大家各讀各的 L1,匯流排安靜;釋放時 line 失效,等待者才回外圈再搶。
  內圈配 `std::hint::spin_loop()`:告訴 CPU 這是忙等(省電、讓出 SMT 資源)。

## Ordering:一對邊,兩個理由

`lock` 的 **Acquire** 配 `Drop` 的 **Release**——前一個持鎖者臨界區內的全部寫入,
happens-before 下一個持鎖者的臨界區。放鎖用 Relaxed 的話,x86 上多半僥倖能動
(strong memory model),ARM 上就是真 bug:下一個持鎖者讀到殘影。
`try_lock` 失敗側 Relaxed:沒拿到鎖就沒有臨界區,不需要任何可見性。

## RAII guard:解鎖不可能忘記、不可能做兩次

guard 活著 = 持鎖;離開作用域或 panic unwind = `Drop` 放鎖。
兩個型別層決定:

- `unsafe impl<T: Send> Sync for SpinLock<T>`——bound 是 **Send 不是 Sync**:
  鎖保證 T 永遠不被兩執行緒同時觸碰,需要的只是「T 的獨占存取可以移到別的
  執行緒上」。(對照:`RwLock` 允許多讀者並存,所以那邊才需要 `T: Sync`。)
- guard 藏一個 `PhantomData<*mut ()>` 讓它 **!Send**:A 執行緒鎖、B 執行緒解
  直接編譯不過——「臨界區屬於誰」是推理的地基,不能讓它漂移。

## 三不(trade-offs)

| 性質 | 本實作 | 代價/升級路 |
|---|---|---|
| 不重入 | 同執行緒二次 lock = 自旋死鎖 | 重入鎖要記 owner+深度,慢且多半是設計異味 |
| 不公平 | 誰搶到誰贏,可能飢餓 | ticket lock(取號排隊)換公平,代價是放鎖必喚下一位 |
| 不毒化 | 臨界區 panic → unwind 放鎖,資料可能半套 | std Mutex 選 poisoning 顯式化;我們選使用者自負不變量 |

## 誠實邊界

未經 loom 驗證(guard 的 `Deref` 直接回借用,與 loom `UnsafeCell` 的
closure/ptr 存取模型不合,要驗得改寫成 `with_lock(f)` 形狀)——Ordering
論證是紙上推理 + 雙執行緒壓力測試,不是模型檢查。面試被問到就照實講。
