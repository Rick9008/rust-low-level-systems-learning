# Pool 默寫 + Arc 解剖 + CAS 原語譜 + 複測結帳(2026-07-23 Q&A 沉澱,壓縮版 9 卡)

> 默寫批改全文:`scratch/thread_pool.rs` 檔頭批改紀錄
> 源碼證物:本機 1.91 `alloc/src/sync.rs`(Arc 結構 :261、clone Relaxed :2207、drop Release :2642、acquire! fence :2674、upgrade :3081)
> 相關舊卡:`hepta_20260722_lockfree_day.md` 卡 1(光譜)、卡 2(seq 時鐘)、卡 5(fence)、卡 6(lost-wakeup)

## 一句話骨架(今日主題)

**今天所有的洞同一個根:壓力下憑直覺猜(否定式條件、相對運算、「多人改→要最強 ordering」),而不是從語意推導。處方全是「落地」:正面條件、絕對狀態表、問「護送誰/擔保誰」。**

---

## 卡 1:方法卡——高壓下的兩個處方(默寫+複測戰報)

**處方 A:條件寫正面,不寫否定式。**
- 戰報:pool 默寫 rep#1,「退出 = shutdown ∧ 空」翻成 while 繼續條件,∧/∨ 連翻三次(De Morgan);第三輪英文註解 "or" 對、code `&&` 錯
- 驗法:四格真值表(該退出的只有「空 ∧ shutdown」一格);`&&` 版錯兩格=worker 夭折+丟 job 退場
- 處方:**`loop` + 正面條件 `break`**(pop 到 None ⇒ 空∧shutdown ⇒ break,謂詞已保證)——零否定、零 De Morgan
- 已固化傷疤(零提示寫對):⑤ store 進鎖、⑥ drop(guard) 再跑 job、④' join().expect
- 7/24 開機重默秒殺線:白紙全骨架 10m、首編 ≤3 錯、兩條件一次對

**處方 B:不心算——絕對狀態表 + 最小具體例。**
- 戰報:Q3 dif 算式連漏三次(數字傳播老失分);同日兩次成功(建構子 start=5、cap=3 走表)全是具體數字寫下來的時候
- off-by-one 的機制:拿現值做**相對運算**(seq+cap);正確姿勢=問「我要把格子留在哪個**狀態**」,抄狀態表的絕對值
- 三態表加註**誰的綠燈**:`seq=pos` → producer pos(空可搶);`seq=pos+1` → consumer pos(貨到);`seq=pos+cap` → producer pos+cap(下圈)。**寫完才 +1、取完才 +cap**
- 憑空推理一律降落:cap=3、格 0、整數走表
- 複測終局:dif = 21 − 29 = **−8 = −cap → 判滿且永不自癒**(誤存 pos 的下場)——凌晨 Q3 半題正式閉合

## 卡 2:Pool 佇列選型鏈——排隊與睡覺是兩件事

- `Mutex<VecDeque>` + `Condvar` 做兩件事:**push/pop 互斥** + **空時阻塞**
- lock-free MPMC ring 只換得掉第一件:Vyukov pop 空了回 None 不會睡;no busy-wait ⇒ 還是要停車層(鎖/futex 搬到慢路徑,沒消失);「檢查空→去睡」沒鎖罩 = lost-wakeup 變更難(tokio park/unpark 就在重建這協議);外加 bounded → full policy
- 成本帳:無競爭 lock/CAS 都 ~20ns、job 跑 µs–ms,queue 佔比極小;真瓶頸的答案是 **per-worker deque + work-stealing**(ws_deque/tokio),不是單條全域 lock-free queue
- **channel 兩件都包**:recv 會睡(parking 內建)、Receiver Clone;**drop-senders → drain 完才 Err(Disconnected),「Err ⇔ 空 ∧ 斷線」= 退出條件的 API 封裝**(正是卡 1 那兩條件)。std::mpmc 仍 nightly(1.91 實測 E0658 #126840);stable 走 mpsc+`Arc<Mutex<Receiver>>`(Mutex 唯一理由=Receiver !Sync)
- submit API 品味:`fn submit<F>(...) -> Result<(), F>` 原物退回(mirroring mpsc SendError);`Box::new` 在 shutdown 檢查後=拒收零配置;RPIT 版擦掉型別身分
- 面試句:**"Lock-free solves throughput, not blocking — as long as the semantics say 'sleep', a slow-path lock stays."**

## 卡 3:Box + 'static + Arc 解剖——'static 是性質,不是實作

- **Box 兩因**:closure 各是匿名型別大小不一 → 擦除成 `dyn FnOnce`(unsized)→ 佇列要**擁有** → Box。彩蛋:`Box<dyn FnOnce()>` 預設自帶 `+ 'static`
- **'static 方向**:不是「活多久」,是「**裡面沒有會過期的借用**」⇒ 想留多久都行(我的錯句 "live no longer than" 方向全反)
- 英文模板(出聲用):"`F: 'static` means the closure doesn't **borrow** anything shorter-lived — captures are **owned** or `&'static`. The worker may **outlive the caller's stack frame**; that's why `thread::spawn` requires it."
- **Arc 真實長相**:握把 `{ NonNull<ArcInner>, PhantomData, alloc }` 一字寬;堆上 `ArcInner { strong, weak, data }` **data inline 無 UnsafeCell** → Arc 只發 `&T`。分工句:**Arc 管活多久,Mutex 管誰能改**(Mutex 才把 data 包 UnsafeCell)
- 「擁有」三件套缺一不可:NonNull(不表達所有權)+ PhantomData(dropck 聲明)+ Drop impl(兌現)
- 判準:`Arc<T>: 'static ⇔ T: 'static`;反例 `Arc<&'a str>` 不是。收束句:"`'static` is about what the type **borrows**, not where it **lives**. Arc satisfies it by **owning**."
- 彩蛋:weak 的 usize::MAX 哨兵(鎖 upgrade);`unsafe impl Send/Sync where T: Send+Sync` = 三段式範例(最後 drop 的可能不是建立的執行緒)

## 卡 4:計數器 ordering 一把尺——護送誰、擔保誰、要不要全域

- **前提修正:多執行緒同時改 ≠ 要強 ordering**。不丟更新=RMW 原子性(Relaxed 也保證);ordering 買的是「**跟著我一起被看見的還有誰**」——atomic 自己的值任何 ordering 都會到(coherence),Relaxed = 信到貨沒到
- 源碼實況:**clone = fetch_add(Relaxed)**(護送零資料:持有 Arc ⇒ 活著;交付同步是 channel/spawn 的事)|**drop = fetch_sub(Release) + 歸零者 Acquire fence**(不變式:每執行緒最後使用 happens-before 銷毀)
- **upgrade = 條件式 +1**:0 是永久墓碑,fetch_add 會復活屍體(UAF)→ `fetch_update`(肚子=weak CAS 迴圈)「>0 才 +1」
- **尺(兩問)**:①這個 atomic 護送哪些非原子資料?②動手前的不變式由誰擔保?有擔保 → 無條件+最弱(clone);無擔保 → 條件式 CAS(upgrade)
- **SeqCst 唯一商品:跨多變數、全執行緒一致的全域總排序**——SB/Dekker/IRIW 形狀才掏(兩邊都「先寫自己、再讀對方」;複測自產實例:writer 設 flag 讀 reader 數 × reader 加數讀 flag ✓)。Vyukov slot seq 純成對發布/接收 → Acq/Rel(凌晨 Q4 洞閉)
- 成本:ARM Relaxed fetch_add = 裸原子加;x86 locked RMW 本來就全柵欄

## 卡 5:CAS 原語譜——佔位語意有沒有前提,決定原語

- **判準**:無條件佔位 → swap/fetch_add(必成功、wait-free、無失敗語意);有前提佔位(「還是原樣才動手」)→ CAS。四實例:mpsc_list push=**swap**(掛鏈尾無條件)|M-S push=**CAS next**(null 才接,佔位=發布合一無縫)|M-S pop=**CAS head**(head 還是 h 才贏;多 consumer 仲裁,store 版=double-pop)|Vyukov=**CAS index**(seq 對了才領)
- **weak vs strong**:weak 允許**假失敗**(LL/SC 監視器分不清「碰過」和「改過」,同 line 寫入/中斷都誤傷;`Err(v)` 且 v==expected 就是它的長相)。**strong ≠ 重試到成功,= 重試到能說真話**(值真不同立刻 Err)。迴圈裡 → weak(外圈本來重讀重算);一發定勝負 → strong(Err 要有資訊量)。repo:ring/arena 迴圈全 weak、ws_deque steal 一發 strong。x86 兩者同碼
- **V0/V1/V2 思想實驗**(fetch_add 領票):V0=驗 dif→CAS(expected 綁定「驗的格=領的票」,fail-fast)|V1=票先發不可退+`while seq != 票號` 等(正確但阻塞;同格疊多圈 producer,seq=叫號機;死法=領票者死→永久洞、單執行緒自鎖)|V2=不檢查(輾過未消費值,狀態機錯亂)
- **兩種 loop**:V0 搶票重試(問「現在能領嗎」,可答滿走人,次數~競爭者)vs V1 等貨自旋(問「號碼到了沒」,不能走人,時長~consumer)
- tail 澄清:兩台都是自由跑計數器(不歸零、繞 2⁶⁴、&mask 只是投影);**沒人拿 tail 判可讀性**——那是 seq 的工作

## 卡 6:多生產者結構學——兩訊號柱 + unbounded 本質

- **柱**:SPSC 的 tail 兼兩職(先寫 data 再推 = 就緒);多生產者被迫先搶再寫 → tail 降級成「被預訂」→ **reserve 和 publish 必須分成兩個訊號**
- publish 訊號三設計:①per-slot seq(Vyukov)②per-slot flag(1 bit;滿判要讀 head)③全域 commit counter(純 counter;**commit 串行化,一人卡全隊 convoy**)。追問滿分句:**"A global commit counter works, but a stalled producer convoys everyone behind it — per-slot seq localizes the damage."**
- Vyukov 隱藏紅利:滿由 dif<0 判,**producer 不讀 head** → MPSC 退化時 head 降成非原子私有欄位(退化表那格的原因)
- mpsc_list 同構:swap tail=reserve、store next=publish;**Inconsistent 縫 = reserved-but-not-published 窗**;stub=角色會輪替(每個被消費節點成為新 stub;初始 next=null 就是「空」訊號,第一次 push 的 store 落點)
- SeqCst 買不到這裡任何東西:問題不是排序強度,是**訊號不存在**——沒寫的資料再強 ordering 也變不出可讀
- **unbounded 本質**:記憶體執行期分塊到貨 → 塊間必連 → list 躲不掉;ring resize=並發搬家(沒人做)。真實解=**linked list of ring segments**(SegQueue,粒度 32)。**兩個獨立旋鈕:連的粒度(1 vs 32)× 節點來源(heap vs arena 池)**;arena 當池可以(arena_lockfree 現成,回收論證照舊),當 unbounded 不行(固定=換皮 bounded、growable=又是 list)
- 彩蛋:mmap reserve 超大 VA + lazy commit =「實務上 unbounded」的連續 ring
- **裁決(D-5 scope 防線)**:實作層只練 Vyukov;柱+trade-off 句=認題層;flag/commit-counter/Disruptor/SegQueue 知道名字即可

## 卡 7:async 合約——poll 裡面不准等(executor challenge 主洞)

- **合約**:poll 三步——①看好了沒 ②好了 Ready ③沒好 → 登記「怎麼叫醒我」(waker 交出去)→ **立刻** Pending。「等」全 challenge 只准發生在一處:block_on 的 Pending 分支(park)
- 戰報:Delay 曾寫成「poll 裡 while 同步等到期 + 每圈 spawn 一條 thread」——把 poll 當成 blocking sleep,async 就白搭了;合約級提示一次才通
- 防永眠:wake 落在「Pending 返回後、park 前」→ **park/unpark 的 token 語意接住**(unpark 先到,下個 park 立刻返回)——7/22 卡 8 的 park-token 在 std API 層的化身
- 醒來 ≠ 完成:loop 重 poll、只認 Ready 出場(spurious wakeup 免疫)
- API 三件(tier-2 洞):`Waker::from(Arc<impl Wake>)`(Wake trait 存在的意義)、`cx.waker().clone()` 拿 owned(`Arc<&Waker>` 是生命週期炸彈)、迴圈重 poll 要 `as_mut().poll`
- clarify #5 口述題(30 秒):我的 spawn-per-poll = 「不存 waker,每次給最新 waker 一條新信差」——**冗餘換正確**,被 join/select 提前重 poll 就多生 thread;production = 存 `Arc<Mutex<Option<Waker>>>` 每 poll 更新、一條 thread 讀最新

## 卡 8:Waker 的型別擦除——為什麼 Arc<W: Wake> 能變成 Waker(executor 卡點)

**四層介面(由具體到擦除):**
```
trait Wake {                             // 你 impl 的:安全、符合人體工學的一層
    fn wake(self: Arc<Self>);
    fn wake_by_ref(self: &Arc<Self>) { self.clone().wake(); }  // 預設:clone 再 wake
}
struct Waker { /* 內含 RawWaker */ }     // 執行時真正流通的把手(型別已擦除)
impl Waker { fn wake(self); fn wake_by_ref(&self); fn clone(&self) -> Waker; }
struct Context<'a> { /* 借 &Waker */ }
impl Context { fn from_waker(w: &Waker) -> Context; fn waker(&self) -> &Waker; }
```
**關鍵轉換(std 提供的 From impl):**
```
impl<W: Wake + Send + Sync + 'static> From<Arc<W>> for Waker
```
- 你的四步:`Arc::new(ThreadWaker{..})` → `Waker::from(arc)`(擦除)→ `Context::from_waker(&waker)`(借)→ `cx.waker().clone()`(給信差,refcount++)

**為什麼能這樣轉(核心):**
- `Waker` 底層 = `RawWaker { data: *const (), vtable: &'static RawWakerVTable }`——**手刻 vtable 的胖指標**;vtable 有 4 根函式指標:clone / wake / wake_by_ref / drop
- `From<Arc<W>>` 幫你**自動生成這張 vtable**:`data` = Arc 的裸指標;每根 vtable 函式 = 「把 data 還原成 `Arc<W>` → 呼叫對應的 Wake trait 方法」。你只寫 trait,vtable 是編譯器產的
- **同一個型別擦除動機,跟今天卡 3 的 `Box<dyn FnOnce>` 一模一樣**:executor 要能握著「某個 waker」而不知道它的具體型別 → 擦成 data+vtable。`Wake` trait 是安全糖衣;裸路是 `RawWaker`/`RawWakerVTable`(手寫 unsafe)
- **bound 為什麼是 `Send + Sync + 'static`**:Waker 會被丟進別的執行緒(你的 timer thread)、活得比 poll 久 → 三個都要。你的 `ThreadWaker { thread: Thread }` 剛好滿足(Thread 是 Send+Sync+'static)
- `clone()` 為什麼便宜:走 vtable 的 clone = `Arc::clone` = Relaxed fetch_add(接卡 4);多一個信差就多一個 strong count,drop 時減回去

**一句話收束**:Waker 是「async 版的 `Box<dyn Fn>`」——手刻 vtable 的型別擦除把手;`Wake` trait + `From<Arc<W>>` 是讓你不用手寫那張 vtable 的安全捷徑。

## 卡 9:Task / Waker / Context 三者關係——poll 迴圈的資料流

**三者各是什麼(一句話定位):**
- **Task** = executor 負責驅動的一個 future + 它的狀態;是 run-queue 裡排隊的單位(「誰該被 poll」)
- **Waker** = 「把**這個 Task** 重新排回 run-queue」的回呼把手;真實 executor 裡它握著指向 Task 的 Arc,`wake()` = 把 Task 推回佇列
- **Context** = 一次 poll 呼叫的「環境包」,目前唯一內容就是 `&Waker`;`poll(&mut cx)` 靠它把 waker 交給 future

**資料流(通用 executor,一圈):**
```
executor 挑一個 Task
  → 造一個「代表重排這個 Task」的 Waker → 包進 Context
  → future.poll(&mut cx)
      ├─ Ready  → Task 完成,丟出 run-queue
      └─ Pending → future 必須先 clone 一份 waker 藏好(存 Delay/IO 事件源)
                   然後外部事件發生時,有人呼叫 waker.wake()
                   → 「Task ready」→ executor 重新 poll 這個 Task(給新的 Context)
```
**關鍵因果**:Pending 之前**一定要把 waker 交出去**,否則沒人叫得醒 → 永眠。這就是 poll 合約第 3 步(卡 7)。

**把 block_on 對應到真 executor(退化表):**
| 通用 executor | 你的 block_on |
|---|---|
| Task(future+狀態) | 單一 pin 住的 future + 當前執行緒 |
| run-queue | 執行緒的 park/unpark(佇列長度 1) |
| Waker = 重排這個 Task | ThreadWaker(current thread),`wake()`=`unpark` |
| 「schedule task」 | unpark 執行緒 |
| Context 包 &Waker | 一樣,`Context::from_waker(&waker)` |
| 重新 poll | loop 回頭再 `poll` |

- **為什麼 block_on 不需要顯式 Task 型別**:只有一個 future、run-queue 長度恆為 1,「哪個 task ready」不用問——醒來就是它。多 task(spawn/join/select)才需要 Task 當可排隊、可定位的實體
- **Context 為什麼是獨立一層而不直接傳 &Waker**:預留擴充(未來塞 budget/LocalWaker 等);現在它幾乎是 `&Waker` 的薄包裝,但簽名穩定不破 API
- 串起卡 7/8/9:**Context 送信封(poll 環境)→ 裡面裝 Waker(型別擦除的重排把手,卡 8)→ Waker 綁定 Task(誰要被重排)**;三者是「一次 poll」的完整語境

## 今日產出帳(白天)

- pool 骨架默寫 rep#1:22m + 3 輪修到 0 error;主傷疤=卡 1 處方 A;⑤⑥④' 零提示寫對;批改紀錄入 `scratch/thread_pool.rs` 檔頭
- SCHEDULE 7/23 實績入帳、7/24 開機加重默 10m(秒殺線);PROGRESS thread_pool 列 + 下次複習表已記
- std::mpmc 可用性實測(1.91 E0658)
- **凌晨快考複測(午後,閉卷 3 題+追問戰)**:Q4(SeqCst 誤記)閉 ✓|Q1(原語判準)閉 ✓——經 V0/V1/V2 思想實驗打穿|Q3(dif 算式)閉 ✓——最終場 dif=21−29=−8 自己走完;殘留一行:consumer 綠燈=pos+1;過程中自產 SB 實例 ✓
- **executor challenge ★ 完成(晚 7:30–8:30 留在公司,閥門日守住)**:oracle 5/5 綠;戰報=卡 7;PROGRESS #6 已勾
- 未跑:重打卡#2、signal_pipeline 讀+drill(滑帳待定);晚上在家:c#1(45+30)+ litmus 口述 + Q5 英文 30 秒 + 卡 7 兩句口述(park token / clarify #5)
