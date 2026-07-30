# Etched TPS 衝刺總覽(7/16–7/26 收帳)——給 Claude chat 更新記憶用

> 背景:Etched Technical Phone Screen,**7/28(二)8:45–9:30**,CoderPad(Rust 1.92 / edition 2024,tokio 可用、無 libc/mio),45 分鐘一題。
> 內線情報三份共識:一題一結構(a/b/c/e2 量級)、英文題面要認真讀、**直接寫+邊寫講知識點**(narrate 是考試本體)、epoll 出局(場上 3 行 Poller stub)、MPMC 沒考(口述應對即可)。
> JD 三支柱:concurrency / 封包流(wire protocol)/ event registry。

## 一、計時彩排帳(9 場,oracle = 寫完才開的參考測試)

| 場 | 日期 | 結果與收穫 |
|---|---|---|
| a#1 ring_drop_oldest | 7/19 | oracle 4 紅→當晚修綠;補課自寫紅測×3;drop-oldest 併發化=producer 兼職 consumer(policy 決定同步結構) |
| b#1 pool_graceful | 7/20 | 2 綠 3 紅→凌晨全綠;補課 7/22(紅測×3+新抓 3 洞,0.40→0.10s;loom 三變體佐證) |
| e2#1 fd_registry | 7/21 | 零紅但 review 抓 2 洞(boundary 段沒跑到的角落);generation slot map 招牌題 |
| e2#2 | 7/25 | oracle 5/5;繼承的自寫紅測抓到洞①回鍋(len/gen bump 未押在 take()==Some 之後)——「修洞必寫 counterexample」的複利首例 |
| c#1 frame_parser_heartbeat | 7/23 | oracle 6/6 一次綠;遺留 may_compact 雙洞 7/24 紅測修畢 |
| c#2 | 7/26 | **6/6 一次綠、30m(預算 45)、零洞——c 題型收斂結案**;傷疤(drain+ptr=0)肌肉重寫一次對 |
| d#1 tokio_frame_server | 7/25 | 三大洞(idle_timeout 蒸發/echo 掉 wire format/自測零條)修後 oracle 6/6;`:0`+local_addr 肌肉 |
| g#1 bounded_channel | 7/26 | 首跑 46m;oracle 4 紅同根(recv 不 drain)→修畢 6/6×3;大課:Drop 開燈協定+mutex 括號+「沒 join 的斷言不是斷言」 |
| f#1 telemetry_aggregator | 7/26 | oracle 4/5(far-jump 鬼資料)→ lazy validation 修法(比 eager 清掃好:record 嚴格 O(1))→ 5/5×3;回放驗證咬洞 |

輔助 reps:spsc 空白×3(首編 35→4→**0 錯**)、TCP 骨架默寫×3(7 洞→0→1.5)、pool 完整版默寫、e event_registry 快寫(retain_mut 進肌肉)、h timer_queue(heap+lazy-delete;wheel 版 post-TPS)、endian_pack drill、五張 clarify 卡(卡#5 sensor bridge 口述設計 7/27 補做——唯一未做)。

## 二、覆蓋帳

**九題型全部親手寫過**(7/25 關帳):a=ring/drop-oldest|b=thread pool|c=framer|d=tokio+std TCP server(d-std 7/27 早暖手)|e=event registry(快寫)+e2 fd_registry|f=telemetry aggregator|g=bounded channel|h=timer queue。
砍掉不練:dsu/graph/trie/tree challenge、epoll 家族 drill、mpsc_list/mpmc_ring drill(讀+口述應對)、wheel 修綠、ds_sync 補洞環。

## 三、傷疤/處方清單(Heptabase「Rust Low Level Notes」漏洞卡)

1. **boundary 段不自燃**——連三場要人點名;f#1 首次全自發 ✓(7/28 的頭號流程目標:35 分那格自己站起來)
2. **喊綠沒驗**→ 鐵律:說綠之前終端機要有 `test result: ok`
3. **沒 join 的斷言不是斷言**(g#1 空測試:孤兒執行緒 panic 被吞)
4. **clarify 答了還掉**(g#1 drain 合約)→ 處方:答案到手複誦回去 "so recv drains, then None — noted"
5. **Drop 開燈協定**:store/減數之後要 notify;而且 **notify 前拿一下 mutex(括號)**——store+notify 不拿鎖 = loom_lost_wakeup 親證的窗
6. **同餘鬼資料**(×3 現身):「桶不是你的,牌對了才是你的」——record 驗牌重置、query 驗牌拒答,兩扇門缺一鬼就進
7. **簽名裡的參數沒用到 = 漏讀警報**(e 的 handler_count 回了全域數)
8. **座標系別混**:Vyukov list 家族 head=寫入端;ring/教科書家族 tail=寫入端——上場開寫前一句話釘死或用 read_idx/write_idx
9. De Morgan 兩條件退出 → 寫 loop+正面 break
10. seq 永遠單獨站等號左邊(mpsc dif 表)
11. 狀態變更押在「確認移除成功」之後(e2 洞①,記未癒合)
12. 動筆前 clarify 清單對讀需求清單 30 秒(d#1 idle_timeout 蒸發的處方)

## 四、口述/英文資產

- 30 秒光譜(SPSC→MPSC→MPMC)已錄;Q1 why 層複測過(unconditional vs conditional claim)
- **Trade-off 三拍公式**(7/26 定):①價格(Big-O 每個字母指認)→②沒走的路 ≥2 條(每條用「軸」開頭)→③有效範圍(哪個假設一變就得重來)
- 時間預算:0–3 讀題/3–5 clarify/5–10 設計口述/10–35 寫/35–40 自測+boundary/40–45 trade-off
- clarify 五問決策表(掉不掉→full policy→容量算式→shard→SLA→怎麼知道死了)
- IRQ 喚醒鏈一句:軟體只能接力喚醒,無中生有的喚醒只有硬體中斷(timer tick 也是 IRQ;busy-poll 不在鏈上)

## 五、教材資產(claude.ai 鏡像已同步)

20 個模組互動頁 + 13 篇深讀(html_p)+ 12 頁 Q&A 圖解;7/26 新增 signal_pipeline ①-2「喚醒鏈的終點站(IRQ)」三區接力圖(md+html+鏡像皆更新)。

## 六、接下來

- **7/27(taper「說」日,請假在家)**:07:30 起|08:00 骨架默寫抽查(spsc use/impl、pool 兩條件、framer 簽名、TCP 六行、length-prefix 3 行、token pack、**bounded_channel 雙 Drop 六行**)|d-std 非計時暖手|卡#5 sensor bridge 完整口述設計|08:45 口述模擬一題|九題型掃描(全英文出聲,對 `rehearsals/recognition-scripts-en.md`)|漏洞卡全翻|漏問模式表|產出認題檢查表 `scratch/recall_checklist.md`|**23:00 熄燈**。鐵規:不寫新題、不開 oracle、不計時。
- **7/28**:07:30 起 → 08:00 暖手(小 drill+pillar-5 清單+時間預算)→ 8:45 上場。開場三句+檢查 CoderPad/Meet/耳機/水。
- Post-TPS 池:wheel 修綠、signal_pipeline challenge、sol_fd_registry doc lint、主管面 45m 塊、經驗故事三條。
