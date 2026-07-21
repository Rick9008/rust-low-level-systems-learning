# mpmc_list 設計取捨(Michael–Scott,教學版)

對應程式碼:`reference/src/concurrency/mpmc_list/`(`mod.rs` 教學殼 + `core_impl.rs` 演算法)。
前置閱讀:[mpmc_ring](mpmc_ring.md)、[mpsc_list](mpsc_list.md)(佔位/發布與縫的概念)。
定位:**deep-dive 讀物**——讀懂能講,不排手搓。

## 一句話:M-S 沒有縫,所以它才是正式的 lock-free

Vyukov 家族(mpmc_ring / mpsc_list)的佔位與發布是兩步,中間的縫讓
「贏家睡死、全隊卡住」成為可能——lockless 而非 lock-free。
M-S 的 push 用**一個 CAS 把新節點接上唯一的 null next**:接上的瞬間
既完成佔位也完成發布,縫不存在。我 CAS 輸了 ⇔ 別人成功了 ⇔ 系統整體
有進度——這就是 lock-free 正式定義的達成方式。

## help:把「必須完成的第二步」變成「任何人都能替你完成」

接鏈之後 tail 還要推進——如果只有接鏈者能推,tail 推進就是臨界區,
lock-free 性質毀滅。M-S 的解法:**誰看到 tail 落後(tail.next 非 null)
誰幫忙推**。對照 Vyukov:那裡輸家只能「等」贏家發布;這裡大家「幫」
落後者收尾。兩種哲學,一個換進度保證、一個換 per-op 成本。

## 教學版的邊界:退休節點 Drop 才回收

被 pop 越過的 dummy 不在運行期釋放——記憶體帳 = **歷史 push 總量**,
生產不可用。這是刻意的:運行期安全回收(誰能 free?何時確定沒人在讀?)
就是 reclamation 問題本體,工業解是 epoch(crossbeam-epoch)或 hazard
pointer,整套機器超出 45 分鐘與本 repo 範圍。教學版把問題攤開而不是藏起來:

- 不回收 ⇒ 任何執行緒持有的節點指標永遠有效 ⇒ **免 hazard pointer**,
  loom 直接驗得動(工業版的 epoch 邏輯 loom 驗不動)。
- 所有節點永遠串在一條 `origin` 起頭的 next 鏈上 ⇒ Drop 兩段式:
  origin..=head 只收 Box(值已邏輯搬走),head 之後連值一起收。
- 順帶簡化:經典 M-S 的「pop 端 help tail」是為了不讓 tail 指向已釋放
  節點——不釋放,這個 case 自動安全。

## pop 的「偷看再 CAS」

多個 popper 可能同時唯讀同一個 next 的 val(bitwise copy 到 MaybeUninit),
head CAS 的唯一贏家才 `assume_init`;輸家的副本是 MaybeUninit,丟掉不會
drop——不會 double-drop。「誰的 val 還活著」由 head 位置隱含追蹤,
Drop 靠它分辨兩段。

## 選型帳

| 需求 | 選擇 |
|---|---|
| 容量可以有硬上限 | [mpmc_ring](mpmc_ring.md)——幾乎總是更好的工程答案(無配置、無回收) |
| unbounded + 單 consumer | [mpsc_list](mpsc_list.md)——免 reclamation、push wait-free |
| unbounded + 多 consumer + 正式 lock-free | M-S + epoch(用 crossbeam,別手寫)|
| 面試被問「lock-free 到底 free 在哪」 | 用 M-S(help)vs Vyukov(縫)這一對回答 |
