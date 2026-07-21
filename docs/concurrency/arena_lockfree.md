# arena_lockfree 設計取捨

對應程式碼:`reference/src/concurrency/arena_lockfree/`。前置:[spsc_ring](spsc_ring.md)(acquire/release 基礎)、[lru](../ds/lru.md)/[tree](../ds/tree.md)(index-based 手法)。

## ABA:lock-free 的第一大坑

Treiber stack 的 pop:讀 head=A → 讀 A.next=B → CAS(head, A→B)。
CAS 只比對「值還是不是 A」,但**值相等 ≠ 沒發生過事**:

```text
我:讀 head=A,next=B          他人:pop A、pop B、把 A push 回來(A.next=NIL)
我:CAS(head, A→B) 成功 ←—— head 現在指向 B,但 B 已經不在 stack 上!
```

裸指標世界裡這還疊加 use-after-free(B 可能已被釋放)。後果是靜默的結構損毀
——元素憑空消失或重複,離發生點十萬八千里才炸。

## 兩層解法(本實作)

1. **arena + index 換裸指標**:槽位永遠在 `Box<[Slot]>` 裡,索引永遠在 bounds 內
   ——use-after-free 這一類直接消失(最壞讀到「內容已換人」的槽位)。
   這也是 Rust 寫 lock-free 先走 arena 的原因:安全邊界清楚。
2. **generation tag**:head 打包成 `[gen:32 | idx:32]` 放進一個 `AtomicU64`,
   **每次成功 CAS 都 bump gen**。上面劇本裡 A 回到 head 時 gen 已 +3,
   舊讀者的 CAS 必敗重試。單字 CAS 原子地同時比對 (gen, idx)。

殘餘風險(誠實聲明):gen 是 u32,2^32 次操作後迴繞。一條執行緒要恰好
卡在 CAS 前睡到 42 億次操作完成才可能中招——實務接受;
不接受就 128-bit CAS(`AtomicU128` 不在 stable std)或 epoch reclamation。

## free list 也是同一個 stack

空槽管理用第二條 lock-free 鏈(`free`),與主鏈共用 `next` 欄位——
一個槽位任一時刻只在其中一條鏈上(或在某執行緒手中)。
free 鏈同樣要 gen:回收競爭一樣會 ABA。

`next` 必須是 atomic:落後的 popper 會讀到已被回收重用槽位的 next。
那個值是 stale 沒關係(gen 會讓它的 CAS 失敗),但用普通欄位就是
data race = UB。「無害的競爭」在 Rust/C11 模型裡不存在——loom 會直接抓。

## Ordering 論證骨架

- push:寫 value/next → **Release** CAS 發布;popper 的 **Acquire** 讀到後才碰 value。
- pop 成功:**Acquire**(看到 pusher 的寫)→ 讀 value → `free_slot` 的
  **Release** CAS(我讀完了)→ 下一任 alloc 的 **Acquire**(才能安全覆寫)。
- 所有權接力棒:stack 鏈 → popper 手中 → free 鏈 → allocator 手中 → stack 鏈,
  每一棒交接都是一條 Release→Acquire 邊。

## lock-free ≠ wait-free

CAS 失敗就重試:單一執行緒可能餓(每次都輸),但每次失敗 ⇔ 別人成功
⇒ 系統整體必有進展。這是 lock-free 的定義;wait-free(每個執行緒有界步數完成)
更強,實作也貴得多。

## loom 驗證

`tests/loom_arena.rs`:雙 pusher、雙 popper 搶 head、以及 ABA 劇本重演
(pop 卡半路 vs pop→回收→push 重用同槽位)。把 gen 拿掉這些測試會失敗。

## Production 對照

crossbeam-epoch(epoch-based reclamation,不限容量、無 gen 迴繞問題)、
hazard pointers。bounded 池:crossbeam::queue::ArrayQueue。

## 互動教材

[artifacts/arena_lockfree.html](artifacts/arena_lockfree.html) —— **ABA 重現機**:
同一組兩執行緒交錯逐格排兩次。關掉 generation tag,T1 的 CAS 會「成功」——
head 指向一個已經在 free 鏈上的槽位,不變量(一個槽位只能在一條鏈上)當場破掉,
而且當下什麼都不會爆。打開 generation tag,同一步的 CAS 因為高 32 位對不上而失敗、
重讀、重試,結構完好。頁面把 `head` 這個 `AtomicU64` 的 hex 與 64 bit 逐位攤開,
`(gen, idx)` 的低位相同、高位不同,一眼看得到 CAS 到底在比什麼。
