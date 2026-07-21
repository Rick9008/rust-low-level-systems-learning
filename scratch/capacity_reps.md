# 容量速算 reps(7/21 出題;5 分鐘/題,結帳表紀律)

規則:每題交出**算式 + 數字 + 荒謬檢查 + 一句裁決**。英文寫。
對答案:寫完喊 Claude 批改;沒過結帳條件(數字/式子/裁決三缺一)重寫該題。

## R1 metrics agent
An agent samples 2,000 counters every 10s, 32 bytes each, ships in 60s
batches. The collector can be unreachable for up to 5 minutes.
Size the buffer. State your drop policy and when it activates.

## R2 connection memory
A TCP proxy holds 50,000 concurrent connections; each needs a 16KB read
buffer + 16KB write buffer. Does it fit in 4GB RAM? If not, what do you
change first? (荒謬檢查這題是主角)

## R3 prober 反推
Dead nodes must be flagged within 20s. You chose debounce N=3 and
per-probe timeout 1s. Derive: max probe interval, probes/s for 500
targets, and worker-pool size if a probe can hold a worker for 1s.
