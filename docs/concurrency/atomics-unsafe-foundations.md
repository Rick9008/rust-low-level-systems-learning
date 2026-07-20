# atomics / unsafe 地基筆記(memory ordering · RMW · UnsafeCell · MaybeUninit)

> 2026-07-18 一場 Mode C 討論的沉澱。全部圍繞 SPSC ring 的 `push`/`pop` 長出來,
> 但每一條都是可攜的心智模型。設計 `buf: Box<[UnsafeCell<MaybeUninit<T>>]>` 時每一層
> 都能一句話辯護,就是這份筆記的目的。
>
> 對應互動頁:`docs/artifacts/qa_atomic_ordering_rmw.html`(ordering 側)、
> `docs/artifacts/qa_unsafecell_maybeuninit.html`(cells 側)。

---

## Part A — Memory ordering 到底保護什麼

### A1. 一句話:鎖的不是那個 atomic,是它周圍的普通記憶體

> **Memory ordering 保證的根本不是那個 atomic 變數本身。它保證的是「這個 atomic 操作
> 前後那些*非 atomic* 讀寫」被*別的執行緒*看到的順序與可見性。**

atomic 變數本身永遠不可分割、永遠最終會被看到——不管 Relaxed 還是 SeqCst。ordering
這個旋鈕調的是**它旁邊那些非 atomic 指令**(在 ring 裡就是 `buf[s]` 槽位讀寫)。
盯著 `tail`/`head` 那個「數字」看,永遠不懂;差的從來不是數字,是跟著數字走的資料。

### A2. 為什麼會有「順序」問題:兩個重排者

源碼順序 ≠ 實際執行順序,中間有兩個搗亂的:

1. **編譯器**——為優化重排指令(load 提前、store 延後、塞進暫存器),受 as-if 規則約束。
2. **CPU**——執行期亂序:store buffer、亂序執行、快取同步延遲。

兩者在**單執行緒下完全無害**(as-if 保證你看到自己的操作照程式順序)。問題只在
**另一條 thread 偷看你的記憶體**時才爆——而 `buf[s]` 正是兩條 thread 都碰的地方。

### A3. Release / Acquire = 單向牆 + 配對才有魔法

看 producer 兩行:

```
buf[s] = item;             // (1) 普通寫:payload
tail.store(t+1, Ordering);  // (2) atomic 寫:flag
```

- `Relaxed`:允許別的 core 先看到 (2) 再看到 (1) → 旗子先升、資料沒到 → consumer 讀垃圾。
- `Release`:**單向牆**——排在它前面的記憶體寫必須先完成先可見,才輪到這個 store 可見。
  → 「任何看到新 `tail` 的 thread,一定也看得到 (1)」。

consumer 對稱:`load(Acquire)` 保證「排在它後面的讀不准提前;且一旦讀到被 Release 的值,
releaser 在 Release 前的所有寫對我可見」。

**配對才成立**:只有當 consumer 的 Acquire-load *真的讀到* producer Release-store 出去的
值,兩者才 **synchronizes-with**,進而 producer 的 (1) **happens-before** consumer 的 (4)。
這條 happens-before 就是「不讀到垃圾」的唯一根據。單一 Release 或單一 Acquire 什麼都不保證。

### A4. ordering = 一套「重排許可證」,管到 source → 編譯器 → CPU 三層

你在 atomic 上寫的 ordering 標註,**本身就是**給編譯器 + CPU 的命令——不是「只管 atomic」。

- `Relaxed` = 最大許可:「隨便重排這些普通存取,我不在乎跨 thread 順序」
- `Release`/`Acquire` = 單向撤銷:「這條線,前面的不准掉下來 / 後面的不准爬上去」
- `SeqCst` = 最小許可 + 全域單一總序

`Release` 同時是**編譯器 barrier**(語言模型規定不准跨它重排前面的寫)**和 CPU barrier**
(弱架構吐 fence)。具體:x86 上 `store(Release)` 編成普通 `mov`(TSO 硬體免費給 release),
成本純粹是「編譯器不准重排」;ARM 上吐 `stlr`/`dmb`。同一標註兩平台編出不同東西、語意一致——
這就是 ordering 寫在**語言**裡而非叫你手插 fence 的原因。

**「被重排/優化掉」的恐懼都被擋住**:跨不過 Release/Acquire;而且 Release 建立的跨 thread
happens-before 強制編譯器**保留**「別 thread 可能合法觀察到的寫入」→ 不准 dead-store 刪除。

### A5. 觸發條件:不是「有沒有資料」,是「跨 thread 撞同一格」

> 同一位址 + 兩條 thread + 至少一方寫 → 需要 ordering(這就是 data race 定義)。
> 少任何一個 → 永遠不需要。

`buf[s]` 要保護不是因為它是資料,是因為 **producer 寫它、consumer 也讀它**。單執行緒的
payload(例:ring 的 `Drop` 在 `Arc` 歸零後清 `[head,tail)`)零同步——有 payload 但無並發。

---

## Part B — SPSC 應用:兩條 edge、兩種 hazard、一把尺

ring 的 `push`/`pop` 各三個 atomic op,推導(不是背)如下。

### B1. 兩條 edge、兩種 hazard

- **`tail` edge**(producer `store(Release)` → consumer `load(Acquire)`):**發布 data**。
  producer 寫槽位才升 tail;consumer 看到 tail 才讀槽位。破了 → consumer 讀到**半寫的元素 = 垃圾 UB**。
- **`head` edge**(consumer `store(Release)` → producer `load(Acquire)`):**發布「槽位已空」**。
  consumer 讀完(`slot_take`)才升 head;producer 看到 head 才覆寫。破了 → producer
  **覆寫 consumer 還在讀的槽 = WAR 撕讀 UB**。

記法:**兩條邊、兩種 hazard**。`tail` edge 兩端 = row3(store)+ row5(load);
`head` edge 兩端 = row6(store)+ row2(load)。

### B2. 每個 op 的 ordering(推導表)

| # | 操作 | Ordering | 配對 | 弱化會怎樣 |
|---|---|---|---|---|
| 1 | push: load `tail` | Relaxed | —(單寫者=我) | Relaxed 已是地板;去 atomic → data race(consumer 也讀 tail) |
| 2 | push: load `head` | **Acquire** | consumer 的 `head` store(Release) | Relaxed → producer 覆寫 consumer 還在讀的槽 = **WAR UB**(非「讀舊值」) |
| 3 | push: store `tail` | **Release** | consumer 的 `tail` load(Acquire) | Relaxed → consumer 看到新 tail 卻讀到半寫元素 = **垃圾 UB** |
| 4 | pop: load `head` | Relaxed | —(單寫者=我) | 同 row 1 |
| 5 | pop: load `tail` | **Acquire** | producer 的 `tail` store(Release) | Relaxed → consumer 讀到還沒寫好的元素 = **垃圾 UB** |
| 6 | pop: store `head` | **Release** | producer 的 `head` load(Acquire) | Relaxed → producer 覆寫 consumer 正在讀的槽 = **WAR UB** |
| 7 | (程序順序,非參數) | `slot_write` 必在 `tail` Release **之前** | — | 對調 → consumer 讀到未寫的槽 |
| 8 | Drop 的兩個 load | Relaxed | —(Arc 歸零,無並發) | 有 payload 但單執行緒 → 零同步 |

### B3. 判斷任何 load 用 Relaxed 還 Acquire 的一把尺

> **只問一句:「我讀的是*自己 thread* 寫的值,還是*別條 thread* 寫的、且後面有資料靠它變可見?」**
> - 讀自己的(`tail`@P、`head`@C)→ **Relaxed**(讀自己 thread 的寫,任何 ordering 免費有序;沒 payload)
> - 讀別人的、且有 payload 靠它(`head`@P、`tail`@C)→ **Acquire**

row 1 vs row 2 的差別**整個只在「誰寫的」**:producer 讀 `tail` 是讀自己 → Relaxed;
producer 讀 `head` 是讀 consumer 寫的、且槽位覆寫安全靠它 → Acquire。同一函式兩個 load、兩種 ordering。

**關鍵反直覺**:producer 讀到 stale(較小)`head` 是**安全**的(保守地看成更滿 → 回 Err 重試)。
所以 row 2 用 Acquire **不是為了讀到最新的數字**,是為了那條保護「slot 覆寫」的 happens-before。
別用「讀不到最新值」的框架想,永遠用「哪個非 atomic 槽位存取被保護、防哪種 hazard」。

---

## Part C — Read-Modify-Write（fetch_add / swap / compare_exchange…）

### C1. 一句轉念:RMW = 一個 load + 一個 store 融成不可分割的一步

`fetch_add(1, o)` = 讀現值 → 加 → 寫回,中間不准插隊。所以「兩個半邊」框架直接套:
- **load 半**:這步要**看到**別 thread 的 payload 嗎?→ Acquire
- **store 半**:這步要**發布** payload 給下一個讀的人嗎?→ Release
- 都要 → AcqRel

這就是為什麼 RMW 的 ordering 選單多一格:Relaxed / Acquire / Release / AcqRel / SeqCst。

### C2. 兩個正交問題,永遠拆開問

- **問題 A(要不要 RMW?)= 原子性**:「同一 atomic,是不是*多個* thread 在寫?」
  多寫者 → 要「讀+改+寫」融一步,否則 lost update。跟 ordering 無關。
- **問題 B(哪個 ordering?)= 可見性**:上面「兩個半邊」。

兩者獨立:`fetch_add(Relaxed)`(原子但零排序=純計數)、`compare_exchange(AcqRel)`(原子+雙向)都合法。
**先答 A 決定用不用 RMW,再答 B 決定掛哪個 ordering。**

### C3. compare_exchange 為什麼吃兩個 ordering:結局數,不是半邊數

`compare_exchange(old, new, success, failure)`:成不成看比對 → 分岔成**兩種結局**:
- **success**:load + store 都發生(完整 RMW)→ 可 `AcqRel`。
- **failure**:**只有 load,store 沒發生**→ 最多 `Acquire`,**不能 Release**(沒寫,無物可發布)→ 給了會 panic。

`fetch_add`/`swap`/`fetch_*` 無條件、永遠寫 → 只有一種結局 → 一個 ordering。
「一個 ordering 參數」≠「一個半邊」:`fetch_add(AcqRel)` 這單一參數照樣同時給兩個半邊語意;
參數**數量**只反映**結局數**。

### C4. RMW 獨有性質:永遠讀到 modification order 的最新值

普通 Relaxed load 可能讀 stale;RMW **保證讀到該 atomic 修改順序裡最新的那個寫**。
所以兩 thread 各 `fetch_add(1, Relaxed)` **一個增量都不掉**——這是計數器類敢用 Relaxed 的底氣
(原子性 + 讀最新 RMW 免費給;不 publish payload,所以 ordering 不用加)。

### C5. 錨:`Arc` refcount 就是 head-edge 的同構

- `clone` → `fetch_add(1, Relaxed)`:只是多一個持有者,**不 publish 不 consume 資料** → Relaxed。
- `drop` → `fetch_sub(1, Release)`:每次遞減都 Release(「我對物件的使用發布出去」)。
- 減到 0,真正 free 前 → `Acquire` fence:先看到別 thread 在其 Release 前對物件做的所有事,才敢釋放。

`Arc` 的「用完 Release / 動手前 Acquire」與 ring 的 `head` edge **完全同構**——同一個 WAR 防護。
clone 敢 Relaxed 而 drop 要 Release 的差別:**clone 沒 payload 交接,drop 之後那塊記憶體要被回收=最硬的 payload**。

### C6. MPSC 的 `fetch_add`:Relaxed + per-slot 旗標

多 producer 搶同一 `tail`:`pos = tail.fetch_add(1, Relaxed)`。為什麼 Relaxed 夠?
搶格子當下**還沒寫資料**,這步沒 payload;唯一格號由 RMW 原子性 + 讀最新免費保證。

**但** SPSC 能拿單一 `tail` 當「資料備妥水位線」,是因為只有一個 producer、填槽順序 = tail 前進順序。
**MPSC 不行**——多 producer `fetch_add` 後**完成順序是亂的**(A 佔 pos5 慢、B 佔 pos6 快 → tail=7 但 slot5 沒好)。
所以 MPSC 的 Release/Acquire **不能掛在 `tail`,必須掛在每格自己的 ready 旗標(或 sequence number)**:
```
pos = tail.fetch_add(1, Relaxed)          // 搶唯一格號(無 payload)
buf[pos & mask] = item                    // 填資料
slot[pos & mask].ready.store(true, Release) // 發布這一格
// consumer: slot[head & mask].ready.load(Acquire) 看到 true 才讀那格
```
一句話:MPSC 把 SPSC「用 tail 當旗子」拆成「tail 只發號碼牌(Relaxed RMW)+ 每格各自舉旗(Release/Acquire)」。

---

## Part D — 為什麼是 `UnsafeCell<MaybeUninit<T>>`

### D1. 兩層正交,各解一個問題

- `UnsafeCell<…>` 解 **aliasing / 內部可變性**:讓你能透過兩條 thread 共享的 `&SpscRing`
  合法地**改** `buf[s]`。
- `MaybeUninit<T>` 解 **初始化 / drop 生命週期**:槽位一開始是空的,可能有也可能沒有有效 `T`。

兩個問題你都有 → 兩層都要。合起來:「一格**我能透過 `&` 改**、而且**可能有也可能沒有**有效 `T` 的槽位」。

### D2. UnsafeCell:唯一能「透過 `&` 改」的原始積木

Rust 別名模型:拿著 `&T` 編譯器就假設它不變(可 cache 進暫存器、重排、省略重讀)。
把 `&T` 轉裸指標去寫**不在 UnsafeCell 裡**的資料 → **立刻 UB**。

> `UnsafeCell` 是**唯一**告訴編譯器「這裡面即使透過 `&` 也可能被改,別做『不會變』假設」的標記。
> 所有內部可變性型別(`Cell`/`RefCell`/`Mutex`/`Atomic*`)骨子裡都是它。

它**不是 ptr**——是包住 `T` 的結構,`.get()` 交給你 `*mut T`。它自己**零同步**;無資料競爭要你
用 SP/SC 型別不變式 + head/tail fence 去證明(這也是每個 `unsafe` 上面要寫 SAFETY 的原因)。

### D3. 內部可變性階梯 + Cell vs UnsafeCell

全部建在 UnsafeCell 上,差別在**「誰幫你證明、代價、能不能跨 thread」**:

| 型別 | 誰證明安全 | 代價 | Sync? | 用在 |
|---|---|---|---|---|
| `Cell<T>` | 標準庫(單執行緒) | 零 | ✗ | 單 thread、整值換、不要 into 的 ref |
| `RefCell<T>` | 標準庫(執行期借用檢查,違反 panic) | 借用旗標 | ✗ | 單 thread、要真的 `&`/`&mut` |
| `Mutex`/`RwLock<T>` | 標準庫(上鎖) | 鎖/阻塞 | ✓ | 跨 thread、可接受鎖 |
| `Atomic*` | 標準庫(lock-free) | 一個 word | ✓ | 跨 thread、單字大小 |
| **`UnsafeCell<T>`** | **你** | 零(但你扛證明) | ✗(要自己 unsafe impl) | **上面都不合,你在造新抽象** |

> 安全包裝 = 標準庫替某個常見模式**預先寫好並證明過**的 unsafe,包成安全 API。
> `UnsafeCell` = 沒有現成模式對得上你要幹的事,`unsafe` 跟證明都得你自己供。它是最後手段。

**為什麼 ring 不能用 Cell?**(而且關鍵不是 `!Sync`)

`Cell` 有 `get`(要 `T: Copy`、回拷貝)、`get_mut`(要 **`&mut self`**——`Arc` 永遠給不了,
refcount>1 時 `Arc::get_mut` 回 None)、甚至 `as_ptr`(**確實**能透過 `&self` 給裸指標)。
所以「Cell 沒給指標的方法」是錯的。真正的區別:

- 有人以為理由是「Cell `!Sync` 要推翻」——但 **`UnsafeCell` 也 `!Sync`**(`Cell` 的 `!Sync`
  正是因為它包著 `UnsafeCell`),兩邊都要 `unsafe impl Sync`。這條**區分不了**兩者。
- 真正的區別:**`unsafe impl Sync` 那句「併發存取皆無競爭」的承諾,你守不守得住。**
  - `UnsafeCell`:唯一改資料的路徑是 `.get()` → `unsafe { *ptr }`,**沒有安全 mutation API**。
    能造成競爭的只有你自己審查過的 unsafe → 承諾守得住 → **sound**。
  - `Cell`:`.set()`/`.get()`/`.replace()` 是**安全**方法,透過 `&self` 就能改、零同步、零 `unsafe`。
    你一 `unsafe impl Sync`,安全碼就能 `ring.buf[0].set(..)` 兩 thread 同時呼叫 → **data race → UB**,
    且無 `unsafe` 警告 → 承諾是假的 → **unsound**。

> `UnsafeCell` 把「不准無同步地改」變成**編譯器強制的結構保證**(每次存取都逼你 `unsafe`);
> `Cell` 只能靠「你要記得別呼叫它的安全 API」這種**脆弱約定**。`unsafe impl Sync` 只能蓋在前者上。

**把 ring 套上階梯,看它是怎麼被逼到 UnsafeCell**:Cell/RefCell `!Sync` 出局 → Mutex/RwLock 要**鎖**
(否定 lock-free) → Atomic 只裝一個 word(裝不下任意 `T`) → **全刷完只剩 UnsafeCell** + 你自己的 fence 證明。

### D4. MaybeUninit vs Option

`MaybeUninit<T>` 跟 `T` **同大小同對齊**(不是為了「更緊密」)。真正目的是**語意**:

- **合法持有「還不是有效 T」的記憶體**:`[T; N]`/`Box<[T]>` 要求每格都是有效 `T`,配置未初始化的
  `[T; N]` 當場 UB。`MaybeUninit` 是**唯一** sound 的「有 T 的大小/對齊、但還沒放進有效 T」。
- **關掉自動解構子**:`MaybeUninit` 沒有 Drop,永不自動 drop 內容 → 首次寫不 drop 舊值、
  pop 後不 double-drop、ring Drop 只清 `[head,tail)`,全由你手控。

**Option 可以嗎?可以、sound、甚至自動處理 Drop——但是錯的高度**:

1. **大小**:`Option<T>` 常比 `T` 大(discriminant)。`Option<u64>`=16B、`MaybeUninit<u64>`=8B →
   buffer 兩倍、cache 命中砍半(對一個在乎 false sharing 的 ring 是硬傷)。例外:`Option<Box/&/NonNull>`
   有 niche 優化才同大小。
2. **重複狀態**:head/tail **已經**記錄哪格占用(`[head,tail)`)。Option 的 tag 是第二份、非 atomic
   的占用旗標,抄一遍。單一真相來源 vs 兩份。
3. **多餘 runtime 檢查**:你已用「`[head,tail)` + Acquire」證明初始化 → `assume_init_read` 是**無分支**
   move。Option 逼你 branch 檢查 tag(檢查已證明的事)或 `unwrap_unchecked`(同樣斷言、卻還付了 tag 大小)。

> Option 不是**錯**,是**高度錯**:拿非 atomic 的 per-slot tag 重新解決「這格占用了嗎」——但那件事
> head/tail atomic 已是單一真相。MaybeUninit 的立場:「占用與否由 index 全權負責;槽位只是一塊生的、
> 對齊好的儲存空間,不帶 tag、不帶解構子。」

---

## Part E — 驗證 unsafe / 並發 code(assertion + Miri + loom 三件套)

2026-07-18 spsc challenge review 學到的:**unsafe code 的 bug 常是 UB,而「有沒有崩」是爛訊號。**

### E1. 為什麼不能靠 SIGSEGV 當紅綠燈

- **UB 不確定**:同一個「drop 未初始化槽」,可能 SIGSEGV、可能 **exit 0 假裝沒事(假綠,最可怕)**、可能 hang。同一個 bug 一次崩一次不崩,你剛好都遇到了。
- **崩會遮住別人**:一個 test binary 裡所有測試同 process,一個 SIGSEGV 打掉整包,其他結果全丟 → 易崩的測試**單獨跑**或先 `#[ignore]`。
- **`#[should_panic]` 攔不到 signal**:它只攔 Rust panic(unwind),SIGSEGV 不是 panic。

### E2. 三層驗證(缺一不可)

1. **assertion + DropSpy 模式(std 測試)**:一般正確性用 assert;**Drop / leak 要用會數 drop 的型別**——`struct DropSpy(Arc<AtomicUsize>); impl Drop { fetch_add(1) }`。`u64` 沒解構子,Drop 漏收 / 走錯範圍**完全看不出來**。測:非 Copy(String)進出、容量邊界 + 環狀重用、帶未消費元素 drop 的**回收計數**。
2. **Miri(`cargo +nightly miri test -p <crate> --lib <test>`)**:確定性 UB 偵測器——未初始化記憶體、use-after-free、越界、aliasing(Stacked/Tree Borrows)、**還有 leak**。把隨機 SIGSEGV 變成「reading uninitialized memory at line X」+ backtrace。單執行緒跑(100k 的 smoke test 在 Miri 下太慢,過濾掉;並發交給 loom)。裝:`rustup +nightly component add miri`。
3. **loom(`cargo test -p reference --test loom_spsc`)**:並發窮舉 model checker。**只認 loom 自己的型別**——std 硬寫死的 challenge **不能直接跑**(要 sync_shim 機關:core 抽獨立檔走 `crate::sync_shim` 別名,lib 接 std、loom 測試接 loom、`#[path]` include 同一份原始碼)。抓 data race:降一級 ordering → `Causality violation: Concurrent read and write to UnsafeCell`。模型刻意小(cap1、2 元素)= 最大對撞 + 狀態不爆。

### E3. 心智模型

> **loom : 資料競爭 :: Miri : unsafe 記憶體。** 三者互補:assertion 證邏輯 + leak、Miri 證 `MaybeUninit`/`UnsafeCell` 記憶體操作、loom 證 ordering。

「感受窮舉」的鐵證:把一個 `Acquire` 降成 `Relaxed`,loom **建構出**那條會爆的交錯(不是 fuzz、不是跑很多次賭),印出「並發讀寫同一個 UnsafeCell 槽」→ 反證**每個 ordering 都是承重牆**。這是 CLAUDE.md「loom 幫你證明放鬆後仍對」的反面。

---

## 一頁速記(面試白板版)

- **ordering 保護的是旗子旁邊的普通記憶體,不是 atomic 本身;觸發 = 跨 thread 撞同一格 + ≥1 寫。**
- **load 用哪個 ordering:讀自己 thread 的 → Relaxed;讀別人的且有 payload 騎著 → Acquire。**
- **兩條 edge:tail edge 發 data(防垃圾讀)、head edge 發「槽已空」(防 WAR 撕讀)。**
- **RMW:先問幾個寫者(要不要 RMW),再問兩個半邊(哪個 ordering);cmpxchg 兩 ordering = 結局數。**
- **UnsafeCell 解 aliasing(逼 unsafe 稽核)、MaybeUninit 解 init/drop;Cell 因為有安全 mutation API 而讓 `unsafe impl Sync` 變假承諾。**
- **MaybeUninit 讓 index 當單一真相;Option 重複占用狀態 + 多分支 + 更大。**
- **驗 unsafe/並發別靠有沒有崩:assertion+DropSpy(邏輯+leak)/ Miri(單執行緒 UB,把 SIGSEGV 變精準診斷)/ loom(並發窮舉,抓 Causality violation)。loom:資料競爭 :: Miri:unsafe 記憶體。**
