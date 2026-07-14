# sharded_map 設計取捨

對應程式碼:`reference/src/sharded_map.rs`。

## 分片的數學

單一 `Mutex<HashMap>`:所有操作序列化,吞吐上限 = 1/持鎖時間。
N 個 shard、均勻 hash:兩個隨機操作撞同一把鎖的機率 1/N,競爭期望降 ~N 倍。
Shard 數取 2 的冪 → 選 shard 用 `hash & (N-1)` 位遮罩,O(1) 無除法。

Shard 數的取捨:太少競爭仍高;太多浪費記憶體(每 shard 一個 HashMap + Mutex)
且 `len()` 這類跨 shard 操作變慢。經驗值:2–4 × CPU 核數。

## 兩個必踩的坑

1. **Hasher 必須全 map 共用一份**。每次操作 `RandomState::new()` 會得到不同 seed,
   同一 key 這次落 shard 2、下次落 shard 5——key「消失」。RandomState 存在 struct 裡。
2. **`get(&K) -> Option<&V>` 編譯不過**,而且這是 Rust 在救你:`&V` 指向 shard 內部,
   MutexGuard 在函式返回時釋放,`&V` 立即懸空。出路:
   - `get_cloned`(V: Clone,適合小值)
   - `with(key, |v| ...)`(closure 在持鎖區間內借用,零複製;f 必須短、
     且不得重入同一 map——同 shard 重入 = 死鎖)
   - 值存 `Arc<V>`(clone 只是 refcount +1)

## 鎖不變量:最多一把

所有單 key 操作只鎖一個 shard;`len()` 依序取放、絕不同時持兩把。
「永不同時持兩把鎖」⇒ 不存在 lock-ordering 死鎖。代價:`len()` 是**非線性化快照**
(讀 shard 3 時 shard 0 可能已變),只能當監控值。要精確值就得一次鎖全部
(按索引順序取鎖避免死鎖)——O(N) 持鎖、全 map 暫停。

## upsert 為什麼要一段式

`get_cloned` + `insert` 兩段式的 read-modify-write 會丟更新:兩執行緒同時讀到 5、
各自寫回 6。`upsert` 在同一把 shard 鎖內完成 entry + 修改,天然原子。
這與 `AtomicUsize::fetch_add` vs `load; store` 是同一課。

## Production 對照

- dashmap:分片 + RwLock,API 模仿 HashMap,guard 型別讓借用可以安全逸出。
- papaya / flurry:epoch-based reclamation,讀路徑無鎖。
- 讀多寫極少:`RwLock<HashMap>` 或 arc-swap 快照整張表。

## 互動教材

[artifacts/sharded_map.html](artifacts/sharded_map.html) —— 鎖競爭模擬器:
單一 `Mutex<HashMap>` 與 `[Mutex<HashMap>; N]` 並排跑同一批 op,即時比對 blocked / 排空輪數 / 吞吐。
可切 workload(uniform / skewed / hot key)與 shard 數,看 hash 選 shard 的實際過程,
也看 hot key 下分片一次都沒省到的那個誠實失敗。
