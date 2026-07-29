# sim i–n 中文對照版

⚠ **練習時先讀英文版**([sim-problems.md](sim-problems.md))——面試全英文,看英文題幹本身就是練習。這份只做兩件事:review 時對照確認你沒讀歪、複盤時快速回憶。

---

## Sim i — DMA dispatcher v2

系統有 **6 台 DMA engine**(id 0–5)。上游持續送來 DMA request,每張 request 涵蓋從 `block_start_pos` 起、連續 `block_nums` 塊。一台 engine 一次做一塊。你的工作:接 request、把塊餵給 engine、**一張 request 的所有塊都完成後**回報它。

API 對照:`get_dma_request` 拉新 request|`get_cancel_request` Phase 2 才有,Phase 1 永遠 `None`|`send_dma_request_to_engine(engine, 第幾塊, 塊位置)` 派工|`get_dma_result_done` 哪台 engine 剛做完(**只給 engine id**)|`wait_event` 阻塞等事件(回 `false` = 模擬結束,真面試永遠 `true`)|`submit_dma_request_result_done` 整張完成回報。

## Sim j — Sensor interrupt pipeline

感測器的硬體 FIFO 過 watermark 就發中斷;你的 ISR 跑在**中斷 context**。樣本要送達 logging thread,且不能拖慢中斷路徑。

API 對照:`HwFifo::read_fifo` ISR 端讀一筆(`None`=空)|`Ring::try_push` 滿了原樣還你——drop 政策是你的事|`Ring::try_pop`|`Waker::wake` 可合併(連按多次只醒一次)|`Waker::sleep` 可能 spurious 提前醒。實作 `isr()`(不准 alloc/block/log)+ `worker_loop()`(睡→醒→drain→log;`stop` 立起要 drain 乾淨再退)。

## Sim k — Per-core telemetry fan-in

機器有 **N 個 worker core**,各自產 telemetry record;全部要匯到單一 aggregator thread 寫出。**producer 絕不准被 aggregator 堵住。**

API 對照:每核一條 SPSC `Chan`(try 語意)|`make_channels(n, cap)`|Waker 同 sim j。實作 `produce()`(每筆一呼,不准 block)+ `aggregator_loop()`(掃全部 channel;`budget`=一輪從單一條最多拿幾筆,防熱核餓死冷核;stop 前 drain 乾淨)。

## Sim l — MMIO command queue

透過 memory-mapped 的 **submission ring** 和 **completion ring** 驅動加速器。提交:把 descriptor 寫進下一個 submission slot,再把新 tail 寫進 **doorbell 暫存器**。device 消化 descriptor、把 completion 貼到 completion ring 並推進 completion tail。

API 對照:`Reg::SubmitHead` device 消費到哪(唯讀)|`Reg::Doorbell` 通知新 tail(唯寫)|`Reg::CompTail` device 寫到哪(唯讀)|`slot_write(idx, d)` 寫 descriptor(idx = 序號 % cap 自己算)|`comp_slot_read(idx)` 破壞性讀走完成|`barrier()` 對 device 的寫入柵欄——之後它才保證看得見你先前的 slot_write。實作 `submit()`(滿了立刻 `Err(Full)`,不等)+ `poll_completions()`(Phase 2 會亂序)。

## Sim n — Priority job scheduler

計算節點有 **4 個 worker slot**。job 帶優先權進來(數字越大越急);把「最急且可跑」的 job 派給空 worker,做完回報。

API 對照:`Job { job_id, priority, deps }`(deps Phase 1 永遠空)|`get_job`|`assign_job_to_worker`|`get_worker_done`(只給 worker id)|`wait_event`|`submit_job_done`。實作 `run()`。

## Sim m — Engine watchdog(R1 延伸)

DMA dispatcher 同款場景,但 engine 偶爾會 **hang 住、永遠不回報 done**。request 仍必須完成。多給你:`now_ms()` 時鐘|`wait_event_timeout(ms)` 睡到有事件或超時,先到先醒(醒了不保證有 done,自己 poll)|`submit_dma_request_error` 放棄整張往上報(Phase 2:同塊 3 次 timeout 後)。clarify 可得:一塊正常 ~10ms;timeout 值自己定並講理由。
