# Hepta 卡草稿 — 2026-07-23 深夜場(signal_pipeline 讀 + SB/fence 追問串 + c#1)

> **已上板(2026-07-24 01:50,CLI 復活後直上)**:
> 「Rust Low Level Notes」×3——卡1 `cf8ae0bb`、卡2 `1a750991`、卡6 `2d4fc024`;
> 「Memory Order notes」×3——卡3 `acbb57cf`、卡4 `82550b56`、卡5 `4ba40626`。
> 本檔保留為 git 側源文件。

---

## 卡 1|Vyukov ring dif 表終版 + 「交換位子」傷疤處方

**dif = seq − pos,seq 永遠單獨站等號左邊(名牌 − 票)。** pos 只出現在右邊當底數:章是「加在票上」的東西(`seq ∈ {pos, pos+1, pos+cap}`)。

一格 slot 的 seq 四站:`pos`(輪空)→ `pos+1`(發布章,+1=換手給 consumer)→ `pos+cap`(釋放章,+cap=換圈給 producer)。**兩張章、兩個受益人,章的數字=下一個受益人的綠燈方程式的解。**

Push 三態(行動表):
- `== 0` 空、輪到這張票 → CAS 搶
- `< 0` 上一圈沒走完,兩張臉:`−cap`=佔位未發布的縫 / `−cap+1`=真滿 → `Err(v)`
- `> 0` 票過期(`+1` 被搶走已發布 / `+cap` 已釋放下圈)→ 重讀 **tail** 拿新票(不是「等」)

Pop 側:綠燈唯一 `seq == pos+1`;單一 consumer 只會見到 dif ∈ {0, +1}(+cap 的章自己蓋,追不上自己)。

**dif 地板定理:dif ≥ −cap 恆成立**(CAS 進場前提 dif==0 → tail 追不過 head+cap);踩地板那張臉必是縫。−cap 縫可達:A 佔 slot 卡縫,其他 producer 填滿推進 tail 一整圈,新 producer 探同 slot 見 dif=−cap。同瞬間 pop 側 head-of-line 卡死——**一個被 preempt 的 producer 凍住兩端 = formally not lock-free 的雙面現場**。

**CAS 搶的是游標(票),seq 永遠只 store 不 CAS**:取票機把「N 人搶一點」疏散成「每人各管一格」——競爭集中在票、資料同步分散在章。

傷疤帳:「交換位子」7/23 一天內第 4 次現身(quiz→筆記→drill 兩輪→英文 Q5 口述把 CAS 對象講反)。處方=寫法內化:dif 形式、seq 在左;**英文口述時最新學習最先掉,L2 負載要單獨練**。

## 卡 2|睡法譜系 + 喚醒鏈遞迴終點

按事件醒(kernel 提早叫:blocking read / condvar / park / epoll)vs 按時間醒(sleep=**throttling 節流器不是鬧鈴**——管你查的頻率,不管事件到達時刻)。sleep 只在「multiple sources × no multiplexing primitive × no thread-per-source」三條件同時成立才出場。

**futex vs epoll 分界:等的東西不同**——等同進程另一條 thread 的通知=等一個記憶體位址=futex(condvar/park 底層);等 I/O=等 fd=epoll。同進程叫醒隊友沒有 fd 這回事。

**喚醒鏈:每個睡者配一個「造成他等的事件的人」**。consumer 睡空 ← producer 叫;producer 睡滿 ← consumer 叫(雙 condvar 互為鬧鈴,誰造成狀態轉變誰按門鈴);producer 睡「沒料」← 上游叫。鏈往上追終點=**硬體中斷**——interrupt handler 是唯一不用被叫醒的環節。*"Every sleeper is paired with whoever causes the event it waits for; the chain bottoms out at a hardware interrupt."*

Shutdown 三語意:**drain**(空且 stop 才退;殘料不丟;前提=owner 保證不再餵)/ hard cut(立退殘料丟)/ bounded drain(限量限時)。真實世界第四解=close-sender-first(std mpsc Disconnected);無 close 語意的 ring 用 stop 旗代用。

## 卡 3|Acquire 是條件句,不是新鮮度 →「最後一眼」原則

**Release/Acquire 的合約:「如果你讀到 release store 的值,那之前的一切也可見」——它從不承諾你會讀到。** 讀 stale 完全合法(coherence 只禁時光倒流,不禁遲到)。

SPSC 平常沒事因為 consumer 是**輪詢者**:這圈讀舊、下圈再讀,staleness 只是延遲;acq/rel 買的是「看到之後的完整性」(不讀半包),不是「馬上看到」。

**park 改變賭注:掛牌後的 re-check 是最後一眼**,讀舊不再是延遲是永眠。需要的性質升級成「這眼錯過,對面必接住」= SB 的 at-least-one-sees,只有 SeqCst 在賣。

「最後一眼」必須包含**所有別人會寫的旗子**:貨旗(re-pop)+ 收工旗(stop)。token 只有一格,救一次救不了每次。→ **輪詢的人可以靠最終可見性活;要睡的人不行。fence 只出現在睡覺門口,不進熱路徑。**

## 卡 4|SB 為什麼只有 SeqCst 治得了(模型層 + x86 層)

SeqCst 在 SB 局保證的是「**不可能兩邊都沒看到**」(至少一邊,非兩邊都看到)。

**flag 全 SeqCst 為什麼不夠**:危險邊是每條 thread 內部**跨變數的 Store→Load**(ring store vs 旗 load)。SeqCst 存取只把「自己」編進總序 S,對鄰居只有普通 release/acquire 語意——release 管「之前不沉」、acquire 管「之後不浮」,**兩個合約都沒碰「store 在前 load 在後」這條邊**。釘住它只有:四存取全 SeqCst(tail 在 ring 內部管不到)或 fence(SeqCst)。

**x86 對照**:`store(SeqCst)`=xchg(寫+沖,屏障綁在**這一筆**);`load(SeqCst)`=**普通 mov**(屏障全掛寫側!);`fence(SeqCst)`=mfence(**無差別全封**:之前所有讀寫 × 之後所有讀寫,不挑變數)。會沖 buffer 的是「SeqCst 級的寫側動作」。producer 側對旗子做的是 load、tail store 包在 ring 裡——**沒有一筆 store 可升級成 xchg,唯一能放屏障的位置就是 fence**;consumer 側有 store(SeqCst) 在 x86 被 xchg 碰巧修好——但單邊修好=沒修,SB 是雙人舞。

## 卡 5|四種超車 × 兩張單向牆 + ordering 的兩個讀者

重排=B 超車 A。四種:① L→L ② L→S ③ S→S ④ **S→L(SB 元凶)**。
fence(Acquire)=單向牆,錨點=牆**上方的 load** → 擋①②(凡前者是 Load);
fence(Release)=單向牆,錨點=牆**下方的 store** → 擋②③(凡後者是 Store)。
**② 雙牆都擋**(表格出現兩次的原因);**④ 雙牆都撲空**(前者 store 不在 Acquire 名單、後者 load 不在 Release 名單)→ 只有 SeqCst 雙向全封。圖已上 signal_pipeline FAQ Q5。

**沒有人在等任何人,Release 也不刷 buffer**——ordering 全是對自己指令流的禁令;跨執行緒效果是「碰上了才生效的條件合約」,不是阻塞握手。

**ordering 有兩個讀者:編譯器和 CPU**。x86-TSO 只放行 StoreLoad → CPU 那份 acq/rel 免費(mov);但編譯器那份永遠要付(Relaxed=任由 -O2 搬家/吊出迴圈/合併)。ARM 兩份都要付(ldar/stlr)。**ordering 是寫給編譯器和所有未來 CPU 的可攜合約;x86 只是碰巧打折。**

掛牌握手經濟學:每筆一次幾乎恆 false 的 load(branch predictor 吃掉)換掉每筆 µs 級 unpark syscall;spin-then-park 後 **syscall 次數 = 睡眠次數,不是訊息次數**。*"One relaxed load that's almost always false, versus one syscall per message — that's the whole trade."*

## 卡 6|c#1 傷疤卡(frame_parser_heartbeat 首跑)

oracle 6/6 一次綠。🔴 遺留:`may_compact` **drain 之後 ptr 沒 rebase**(累積 >4096 消費後 underflow panic)+ `..=4096` inclusive 多拿一枚。**洞長在唯一沒測的路徑裡**(e2#1 教訓第三次應驗)。修洞紅測先行:先寫 >4KB 連續 feed 看它紅。
Clarify 亮點:用 heartbeat(len==0 合法)反推 len 不含 header。
Incremental parser 鐵則:**殘量必須活在 parser struct**(TCP 是 byte stream,feed 拿到的是 chunk 不是 frame)。
