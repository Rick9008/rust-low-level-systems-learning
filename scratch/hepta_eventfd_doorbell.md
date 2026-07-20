# eventfd 與門鈴哲學(2026-07-20 Q&A 沉澱)

> 完整圖解頁:`docs/artifacts/qa_eventfd_doorbell.html`(兩個 stepper:門鈴的一生、file offload 回收)
> 行號的家:`reference/src/event_loop.rs`、`reference/src/mini_runtime.rs`

## 一句話骨架

**epoll_wait 只聽得見 fd → 任何想叫醒它的事件都得偽裝成 fd → eventfd 就是那顆萬用門鈴。**
門鈴協定永遠兩步:先放貨(queue.push),再按鈴(write efd)。門鈴≠資料。

## fd 是什麼

- fd = process 檔案表索引 → kernel 物件 handle;`read`/`write` 是動態分派(`file_operations` vtable),像 `Box<dyn FileLike>`
- eventfd **沒有 buffer**,整個物件 = 一顆 u64:`write` = counter += v、`read` = 取走歸零;LT:counter > 0 ⟺ 可讀
- `write(efd, &1u64, 8)` 的 1 是**加數**,不是資料

## 訊號帶狀態:五個化身(口述彈藥)

| 原語 | 狀態 | 解掉的 race |
|---|---|---|
| condvar + predicate | 無狀態 → 醒來重查 | notify 先於 wait 蒸發 |
| park / unpark | permit(飽和在 1) | wake 先於 park |
| **eventfd** | counter(累加) | wake 先於 epoll_wait |
| signal handler | per-signal flag + self-pipe | handler 環境放射性 |
| timer | timeout 只是提示,真相 = 時鐘 vs wheel | 早醒/晚醒/假醒全無害 |

裸訊號五個場景全蒸發;解法同一句:**讓訊號留下狀態,醒來查狀態。**

## 回程分流

- token = 註冊時**每 fd 各掛各的**號碼牌;只有 loop 自己的 eventfd 掛 `WAKE_TOKEN = u64::MAX`(sentinel,assert 不准外人用)
- dispatch:`WAKE_TOKEN → drain(消音,LT 不 drain 下輪還報)+ woken = true(字條)+ continue(不進事件列表)`
- **唯一一次真正喚醒 = kernel 讓 epoll_wait 返回**;drain 不喚醒任何人;woken 是寫給 caller 的 memo(證據被 drain 消滅了,不留字條資訊就丟)
- `self.wake: Arc<EventFd>`:同一顆鈴兩張臉(WakeHandle 按鈕 / loop 消音器);Arc 防 dangling fd(fd 重用坑,與 generation 同族)

## Events 死在 reactor 邊界

- token 換到的**唯一東西是 Waker**(`FdRegistry<Waker>` = interest table,e2 drill 的成品;generation 不合 → 事件丟棄)
- executor 只看 queue,零 token 知識;**Waker 是兩個世界唯一介面**
- 資料靠 task 自己重試 syscall(readiness 模型;io_uring completion 才帶資料回來)
- register/reregister 分開:kernel 三 opcode 無 upsert;EEXIST/ENOENT 各暴露一類 bug;upsert 便利上層轉接頭自己搭
- queue 的三段演化:block_on 的 permit(1)→ mini_runtime 的 AtomicBool(1)→ tokio run queue(N)

## 特殊客戶

- **signal**:self-pipe trick 本尊;`handler(signum)` kernel 點名;flag 表 + 唯一一根 pipe;signalfd 不用(全執行緒 mask 前提 + 可攜性)
- **timer**:零 fd;deadline → epoll_wait timeout;返回路徑必經包裝層 → 無條件查 wheel;更早 deadline → 按門鈴
- **file offload**:file 進不了 epoll(永遠 ready、read 照卡磁碟)→ blocking pool + completion queue + eventfd 回收 = tokio spawn_blocking / tokio::fs

## 誰把 task 放進 queue?—— task 自己

- 入隊沒有外部負責人:`impl Wake for Task`(mini_runtime.rs:193)——`wake = inner.queue.push_back(self)`,「thread pool 骨架,payload 換成 re-poll」
- **Waker = type-erased `Arc<Task>`**(`Waker::from(Arc::clone(&task))`);reactor「叫一聲」實際執行的是 task 自己入隊
- 鏈:spawn 首次入隊(:215)→ poll 遇 WouldBlock → arm_io 存 waker → 事件來 → `wake_by_ref`(:344)→ `queue.push_back(self)`(:198)
- root future 對照:`RootWake` = 設 bool 旗(單元素 queue 塌縮)

## run queue 為什麼 Mutex 不 lock-free

- **爭用畫像**:單執行緒 pop + 偶發 wake push ≈ 零爭用;uncontended Mutex = futex fast path ~20-25ns、零 syscall
- 臨界區奈秒級(push/pop 兩下指標);且 **poll 期間不持鎖**(:316)——紀律是「臨界區小」不是「無鎖」
- lock-free 真價:MPSC unbounded = CAS loop + 記憶體回收沼澤(難點是回收不是佇列)
- **tokio 的全域 injection queue 也是 Mutex**;lock-free 的是 per-worker local queue + work-stealing——熱路徑給便宜結構,冷路徑給鎖
- 判準:lock-free 買的是爭用下的尾延遲;沒量到爭用就上 = 白付複雜度稅

## Handle 通行證(capability 模式)

- `#[derive(Clone)] Handle { inner: Arc<Inner> }`;Inner = run queue + reactor。clone = refcount bump,Send 跨執行緒
- 三持票人:①user code(`spawn`)②**IO 物件**(建構收 `&Handle` clone 進口袋;WouldBlock → `arm_io`、Drop → `disarm_io`)③Task 自己(Wake impl push 回 queue)
- **`.await` 零知識**:await = 純狀態機機械;Handle 只在 **leaf future 的 poll 內部**出場(組合層 join/select 從不碰票)
- tokio 為什麼不用塞:「遞票」→「摸口袋」——thread-local ambient(`Handle::current()`);前提 = async code 只在 runtime worker 上被 poll;代價 = "no reactor running" **執行期 panic**(顯式票是編譯期錯)
- 三設計:顯式 Handle(mini_runtime;編譯期保證,病毒式簽名)/ TLS ambient(tokio;API 同 std)/ 全域單例(async-std、smol;永不 panic 但不可配置)
- 敘事:**tokio 預設 ambient、保留顯式當逃生門**(`handle.spawn` / `handle.block_on` 治跨 runtime 與 runtime 外部)

## 面試句庫

- "The doorbell carries no payload — state lives in the queue."
- "The reactor's job ends at `waker.wake()`; the executor's job starts at `queue.pop()`. Events never cross that boundary."
- "I'm calling it Poller, not EventLoop — the loop belongs to the caller; mio's `Poll` makes the same distinction."

## 檔次

deep-dive 讀懂能講即可,**不是手搓目標**——面試 IO 題正確動作 = 3 行 Poller trait stub。
