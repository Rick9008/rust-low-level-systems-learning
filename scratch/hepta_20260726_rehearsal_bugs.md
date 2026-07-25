# 7/25 彩排漏洞卡 —— 2026-07-26 凌晨沉澱(e2#2 + d#1 + 流程根因,3 卡)

> 源:PROGRESS 計時表 e2#2/d#1 兩列 + SCHEDULE「7/25 收帳」段。
> 根因總表:①需求在讀過與寫完之間蒸發(無回讀迴圈)②done 宣告在感覺不在證據 ③狀態變更沒押在操作確認成功之後。80% 是流程洞不是知識洞。
> **已上板(2026-07-26 凌晨,「Rust Low Level Notes」)**:卡A《e2 洞①回鍋》`30ffd9b9`|卡B《d#1 三洞 REQ 處方》`9bcf6e05`|卡C《流程鐵律》`88cc04ee`。改卡要同步改這裡。

## 卡 A:e2 洞① 回鍋——狀態變更押在 take()==Some 之後

- **戰報**:e2#1(7/21)`len -= 1` 逃出 `is_some` 守衛 → 紅測結案;e2#2(7/25)**同一洞原樣重犯**(len 減值+gen bump 排在 gen 驗印後、佔用確認前),forged token(gen 恰好對上的空槽)→ len underflow panic。初版更慘:unregister 連 generation 都沒驗(`_generation` 底線自首)。
- **同族**:lru unlink 殘指標(7/22)、b#1 queue 沒清(7/20)——家族名:**bookkeeping 跟著願望走,不跟著事實走**。
- **固定形狀(封印即興)**:`let v = slot.take(); if v.is_some() { len -= 1; gen += 1; } v` ——記帳永遠在 if 裡面。check 的定義 = 「印章對」**且**「真的移除了東西」。
- **亮點反面**:抓到它的是 e2#1 結案時寫的紅測(繼承進 e2#2)——oracle 5/5 全綠抓不到。**「修洞必寫 counterexample」的複利首例:紅測放哨 4 天後自動咬人。**
- **面試 narrate 句**:"I'm gating the bookkeeping on the take actually succeeding — this exact bug bit me twice this week."

## 卡 B:d#1 三洞——需求回讀迴圈(REQ 註解處方)

- **戰報**(d 首寫,core 15m):①`idle_timeout` **整條需求蒸發**(參數沒用過,沉默連線永生)②echo 只回 payload,spec 白紙黑字 "same wire format"(要 `[u32 len BE][payload]` 重新包頭)③自測零條——boundary 又被跳過(a#1/b#1/e2#1 同款死因)。另:自測寫死 port `AddrInUse`(當天才默過 `:0`+`local_addr` 肌肉,當天沒用上;後補 ✓)。
- **根因**:clarify 問了 4 個好問題,但**沒問到的需求恰是掉的需求**——需求離開工作記憶,不是不懂。
- **處方(60 秒,場上可執行)**:開寫前把需求逐條抄成檔內 `// REQ:` 註解;喊 done 前逐條打勾。d#1 五條需求 = 五行註解。
- **有拿到的**:`break 'parsing` 帶標籤跨巢狀迴圈一次寫對(裸 break 只斷內層 for);`tokio::time::timeout(idle, read)` 包讀 = idle 重置的標準形(heartbeat 是 bytes,自動續命);parser 重用裁決正確。

## 卡 C:流程鐵律——「喊綠沒驗」與 boundary 保留區

- **戰報**:一晚兩次「喊綠沒驗」(e2#2 自測紅著喊綠、d#1 同款);**Claude 同罪一次**(clippy 失敗被 tail pipe 吞 exit code 照樣 commit)。boundary 段四場被 core 吃掉(a#1 0 分鐘/b#1 1 條沒 trace/e2#1 部分/d#1 零條)。
- **鐵律一**:說「綠」之前,螢幕上要有那行 **`test result: ok`**。感覺不算證據,編譯過不算證據。
- **鐵律二**:**最後 10 分鐘是 boundary 的,神聖不可侵犯**——protocol 的 5/5/20/10/5 就是這個意思;core 到 20 分硬停,先寫測試再回補。
- **根因三條總表**(7/25 全日覆盤):①需求蒸發 → REQ 回讀 ②done 靠感覺 → 證據鐵律 ③狀態變更沒押確認 → 固定形狀。80% 流程洞,checklist 就堵得住——7/26 三場計時的驗收標準就是這三條。
