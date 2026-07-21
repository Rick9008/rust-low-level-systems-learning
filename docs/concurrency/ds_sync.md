# ds_sync(同步策略對照組)設計取捨

對應程式碼:`reference/src/concurrency/ds_sync/`。相關:[arena_lockfree](arena_lockfree.md)、
[spsc_ring](spsc_ring.md)、[bounded_queue](bounded_queue.md)、
[dsu](../ds/dsu.md)、[lru](../ds/lru.md)、[fd_registry](../io/fd_registry.md)、
[thread-safe-spectrum](thread-safe-spectrum.md)(七站光譜的口述版)。

## 這組模組回答什麼

`ds/` 回答「這個結構怎麼寫」;`ds_sync/` 回答「多執行緒要共享它時,你有哪幾檔、
各付什麼」。四個模組各佔光譜一格,配對地圖見 `ds_sync/mod.rs` 的表。

## 塌縮原理(arena_locked 的存在理由)

`arena_lockfree` 的 200 行機關,沒有一行在描述「stack / slab」本身,
全部在服務「無鎖」這個約束。把 Mutex 放回去,逐項蒸發:

| lock-free 版機關 | Mutex 版 | 蒸發原因 |
|---|---|---|
| gen-tag + pack/unpack | 不存在 | ABA 需要「讀與 CAS 之間的窗口」;臨界區裡沒有窗口 |
| atomic `next` 侵入鏈 | `Vec<u32>` stack | 讀 stale next 的並發讀者不存在 |
| `MaybeUninit` + unsafe | `Option<T>` | 初始化狀態交還型別系統 |
| Acquire/Release 論證 | 不用寫 | lock/unlock 自帶 happens-before |
| loom 窮舉 | 幾乎不必要 | interleaving 只剩「誰先拿到鎖」 |

反向讀法同樣成立:看到 generation tag、侵入鏈、`MaybeUninit`,
不用讀函式體就知道這份程式碼想無鎖。**複雜度住在同步策略裡,不住在資料結構裡。**

## Mutex vs lock-free 的老實帳

- 無競爭:`Mutex::lock` 是 futex fast path——一個 CAS 進、一個 store 出,
  與 lock-free 的單發 CAS 幾乎同價。「無鎖比較快」在低競爭下不成立。
- 高競爭:Mutex 讓等待者睡(不燒 CPU、不 ping-pong cache line);
  lock-free 的 CAS 重試風暴可能**更慢**。
- lock-free 真正買的是三樣 progress 保證:持鎖者被 preempt 時別人仍有進展、
  無 priority inversion、tail latency 可控。是延遲/活性,不是吞吐。
- Mutex 白送複合原子性(alloc + 寫值同臨界區);lock-free 只有單字原子性,
  要靠所有權轉移協議拼裝。
- 升級階梯不是「Mutex → lock-free」:全域鎖 → sharding(`sharded_map`、
  `lru_locked`)→ per-thread cache(jemalloc 式,fast path 零同步)→
  最後才是全域共享 lock-free。最快的同步是不同步。

## 鎖階梯:coarse → fine → optimistic/lazy → lock-free(`list_fine` 補的那階)

Herlihy 教科書的經典演進,repo 各階對應:

1. **coarse**(`arena_locked`、`lru_locked::LockedLru`):一把鎖包全部。
   正確先行——LRU 這格的 coarse 版還是**精確全域逐出序的唯一保有者**,
   升級到 sharded 是拿逐出品質換吞吐,先量到鎖競爭再升。
2. **fine / hand-over-hand**(`list_fine`):每節點一鎖,走訪時「鎖下一個、
   才放上一個」。並行度來自不同區段互不擋(pipeline)。死鎖自由的證明
   = 所有人按鏈表位置順序拿鎖(全序)+ free-list 是葉鎖。
   誠實帳:每步付一次 lock/unlock,**單點熱點下比一把大鎖慢**——
   贏面只在長鏈 + 存取分散。
3. **optimistic / lazy**:無鎖走訪、到點才鎖 + 驗證(或 mark 後懶刪)。
   需要「節點被摘走後記憶體仍可讀」——index arena 天然給這個保證
   (指標版要 epoch/hazard)。repo 不實作,面試會口述即可。
4. **lock-free**(Harris list):CAS + next 指標裡藏 mark bit 做邏輯刪除。
   研究級:mark/unlink 兩階段、與 insert 的 CAS 交錯、遍歷要幫忙收屍。
   這也是 tree(lock-free BST)/ trie(Ctrie)不開檔的原因——
   複雜度跳一個量級,面試永遠答 sharding 或 fine-grained 就夠。

## 什麼結構值得無鎖(dsu_lockfree vs「LRU 無鎖版不存在」)

判準是**寫入的方向性**與**熱點的分散性**:

- DSU 適合:parent 指標單調向根、root 資格一去不返 ⇒ 舊 expected value
  永久失效,免 generation tag(對照 arena:head 會指回舊索引,必須掛 gen);
  path compression 還會把熱點攤平。代價:union-by-rank 的兩處寫無法單 CAS,
  換成固定隨機 priority(單字 link);攤銷 α 弱化為期望 O(log n) 樹高。
- 精確 LRU 不適合:**get 是寫**(promote 要動鏈表)⇒ RwLock 無效;
  promote = unlink + relink at head,多字原子、熱點全砸在 head;
  節點會被逐出 ⇒ 還要 reclamation。單鎖精確版(`LockedLru`)扛不住
  鎖競爭時,工程解是放棄精確:sharding(本組的
  `lru_locked`)或近似 recency(CLOCK 的 per-entry atomic flag、Redis 抽樣
  timestamp、W-TinyLFU/moka 的 per-thread buffer + frequency sketch)。

## 語意稅(無鎖化後 API 含義的弱化)

- `connected == false` 是快照:「曾有一瞬間不連通」,回傳途中可能已被 union。
  線性化論證(root 資格單向消失 ⇒ 複查 `parent[rx]==rx` 撐起整段區間)
  註在 `core_impl.rs`。
- `components` / sharded `len` 是統計值,不能當同步旗標。
- 單操作可線性化,操作的**組合**不原子——呼叫端要自己帶外部同步。

## loom 驗了什麼(`tests/loom_dsu.rs`)

同 pair 雙 union 恰一人贏(link CAS 的 expected value = root 自指定義)、
鏈式 union 收斂 + components 精確、find 的 halving CAS 與 union 交錯
(halving 失敗被忽略仍正確——它只是 hint)。
若把 link 的 `compare_exchange` 改成 blind store,第一個劇本會抓到雙贏。
