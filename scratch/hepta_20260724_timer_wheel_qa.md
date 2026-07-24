# Timer + Timing Wheel + DS 機制 —— 2026-07-24 晚 Q&A 沉澱(壓縮版,回家複讀)

> 源碼:`rehearsals/src/timer_queue.rs`(min-heap 版,含 lazy-delete `del_id`)| `scratch/timer_queue2.rs`(wheel 版,第一版 11 error,檔頭有批改)
> 相關舊卡:`hepta_20260724_threadpool_full.md`(pool + 3 通用規則);park-don't-poll 見 signal_pipeline

> **待上板(晚上結帳填 ID)**:卡1 ____、卡2 ____、卡3 ____、卡4 ____、卡5 ____、卡6 ____、卡7 ____

## 一句話骨架

**Timer = min-heap(exact、next-deadline 免費、O(log n));wheel = 桶(O(1) 攤還、但 tick 量化、next-deadline 是弱點)。** 選型由 scale 定;delete 兩邊都靠懶刪除。中間串起一堆 Rust 機制:Vec 壓實、heap 刪除、sort_by_key、thread::Result。

---

## 卡 1:min-heap timer 設計 + tie-break 傷疤

- 結構:`BinaryHeap<Reverse<(deadline, id, interval)>>`——tuple 自帶字典序 Ord,`Reverse` 反成 min-heap,**不用手寫 `impl Ord`**。
- schedule = push;`next_deadline` = `peek().map(|t| t.0.0)`(O(1));`pop_due` = while 堆頂 deadline ≤ now { pop、收 id、**用「舊 deadline + interval」重排**再 push }。
- **傷疤(今天抓到)**:tuple 原本寫成 `(deadline, interval, id)` → 次鍵變 interval,同 deadline 排序錯。spec 要 `(deadline, id)` → 次鍵必須是 **id**。
  - repro:`schedule(2,10,3)` + `schedule(1,10,5)` → 回 `[2,1]`,spec 要 `[1,2]`(實測 left [2,1] vs right [1,2])。
  - **oracle 6/6 綠卻沒抓到**——同 c#1/b#1/lru 家族「綠 ≠ 沒洞,只測有人想到的」。洞在 tie-break 路徑。
- **meta**:reviewer 說有 bug 時,第一動作是**回去看 spec**(spec:38「依 (deadline, id) 排序」白紙黑字)。這比修 bug 值錢(pillar 1)。

## 卡 2:選型 —— scale 決定 heap vs wheel

- clarify 高槓桿問:**「最多幾個 timer?」**——直接鎖資料結構。
- **幾千~幾萬**(per-conn heartbeat / retry):**min-heap**,O(log n) schedule/fire、O(1) peek。簡單、cache 友善。
- **百萬 + 粗精度**(kernel scheduler / TCP 重傳):**timing wheel**,O(1) 攤還。
- **面試綠旗**:clarify 完立刻接 Big-O + 沒選的替代——"thousands → min-heap O(log n); millions coarse → hierarchical timing wheel O(1) amortized"。JD 明講「講 Big-O = massive green flag」。

## 卡 3:timing wheel 是什麼

- **時間的雜湊表**:按「何時觸發」把 timer 丟進**桶**,轉一根指針,轉到哪桶就倒那桶。
- 單層:N 格環形陣列 + current 指針,每 tick +1。排 d tick 後 → 塞 `(current+d)%N`。落地就倒。
- **成本**:插入 O(1)、每 tick 找到期 O(1)(只看當前格)。沒排序。
- **一圈只覆蓋 N tick** → 更遠的靠 `rounds`(剩幾圈)或 **hierarchical**(多層粗細輪 cascade,Linux/Kafka/Netty)。
- **vs heap**:heap 給「精確 next deadline」O(1)(park-until-next);wheel 是「一 tick 一 tick 轉」(配輪詢)。**這題要 next_deadline → heap 對。**
- 收尾句:_"A heap gives the exact next deadline in O(1) — perfect for park-until-next. A wheel trades exactness for O(1) amortized, wins at kernel scale, but it's tick-driven not sleep-until-exact."_

## 卡 4:tick 大小 + SLOTS 數量怎麼定

- **tick = 你能接受的最粗精度**(timer 量化到 tick 邊界,最壞晚 ~1 tick)= jitter 預算。
- **SLOTS × tick = 一圈跨度** = 免 rounds 的覆蓋範圍。`SLOTS ≈ 常見最大 timeout / tick`,進位到 **2 的次方**(`% SLOTS` → `& (SLOTS-1)`,一條 AND)。
- trade-off:tick 小 → 精度高但**每 tick 都醒**(idle 燒電);SLOTS 小 → rounds 大、空轉多;SLOTS 大 → 記憶體(空桶恆存)+ next_deadline 掃更久。
- **大範圍靠分層,不是灌 SLOTS**(每層 SLOTS 保持小,Kafka ~20/層、Linux 64~256/層)。
- 接主線:tick 小的代價「idle 也每 tick 醒」正是 tickless kernel 和你 heap+park 版避開的——**heap 沒有 tick 這旋鈕**。

## 卡 5:桶用 Vec 不用 map;Vec 刪除的 O(n) 機制

- **桶 = `Vec<Vec<TimeEntry>>`**,`now_ms % SLOTS` 直接索引(O(1) 定址、零 hashing)。桶內存取模式 = **append + 全掃**,不是查 key → Vec 最佳。map 只為 **cancel**(旁路索引 id→位置;生產級用侵入式雙向串列 + handle,O(1) unlink)。這題沒 cancel。
- **`retain`/`extract_if` 為何 O(n)**:讀寫雙指標**就地壓實**——read `r` 掃每個,write `w` 指下一個保留位;keep → 搬 r→w、w++;remove → 丟棄/移出、w 不動;最後 truncate 到 w。**每元素讀一次、最多搬一次 = O(n)**。
  - 對比 `remove(i)` 迴圈:每次都左移尾巴 → **O(n·k)**。retain 把所有移除併成一趟。
- `extract_if(.., |e| ...)` 的 predicate 拿 `&mut T`,可一趟同時「rounds==0 抽出 + 其餘 -=1」。

## 卡 6:heap 刪除 + lazy-delete(你的 del_id)

- std `BinaryHeap`:**只給 `pop()`(刪頂 O(log n))**,沒有任意刪。任意刪三路:
  1. 重建:`into_vec` → filter → `from`(heapify O(n))。
  2. **懶刪除/墓碑**:標記取消(`HashSet<id>`),`pop` 出來發現被取消就丟掉再 pop。O(log n) 攤還,代價=堆裡留死條目。
  3. indexed heap(真 O(log n) 任意刪/decrease-key):heap + `HashMap<id→index>` 隨 sift 同步。std 沒有。
- **經典算法(知道 index 時)**:末元素搬到 i → pop 尾 → 在 i sift(往上或往下)→ O(log n)。std 不暴露 index/sift,故走上面變通。
- **你的 `del_id`(min-heap 版)= 懶刪除,寫對了**:`del_ids: HashSet`,`pop_due` 撞到就 remove tombstone + `continue`(不 fire 不重排)。週期 timer 隨即消失(不 re-push)。
  - 小提醒(懶刪除通病,非 bug):被取消但還沒 pop 的 timer 仍在堆裡 → `len()` 會**多算**、`next_deadline` 可能指到它,直到它 pop 才清。生產若在意精確 len,才上 indexed heap。

## 卡 7:sort_by_key vs impl Ord;thread::Result vs Result

- **sort 一個欄位 → `sort_by_key(|e| e.id)`,不用 `impl Ord`**(只要 key: Ord,u64 本來就是)。id 唯一 → `sort_unstable_by_key` 更省。
- `impl`/`derive Ord` **只留給「型別要滿足 `T: Ord` 給容器」**(BinaryHeap<T>、BTreeSet<T>、無 key 的 `.sort()`)。derive 按**欄位宣告順序**做字典序。
- 兩版 timer 都沒手寫 Ord:heap 靠 tuple 的 Ord + Reverse;wheel 靠 sort_by_key。
- **`thread::Result<T>` 是別名不是新型別**:`= Result<T, Box<dyn Any + Send + 'static>>`(Err = panic payload)。`catch_unwind` / `JoinHandle::join` 都回它。拿訊息:`payload.downcast_ref::<&str>()`/`::<String>()`。用法跟一般 Result 一樣(`?`/match/unwrap)。
- 為何 pool 的 slot 是 `Option<thread::Result<T>>`:讓 panic 以 `Err` 回到 caller,而非炸 worker 或掛死 join。
