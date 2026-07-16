# clarify 情境卡 —— 每張 5 分鐘

練的是 pillar 1:把題幹線索變成問題、把答案變成設計分支。
方法論見 [`docs/clarify-playbook.md`](../docs/clarify-playbook.md)。

## 規則

1. 計時 **5 分鐘一張**。讀題幹,寫下(紙上或註解裡):
   - 你要問的 **5 個 clarify 問題**;
   - 每問 **2 個可能答案 → 各自的設計後果**(一句話即可);
   - 最後一行:**30 秒定界宣言**(假設 + 結構 + full policy + shutdown)。
2. 寫完才開 [`clarify-answers.md`](clarify-answers.md) 對答案。
   對照重點:**你漏問了哪一類?**(掉不掉 / 速率 / 規模 / SLA / 偵測)
3. 題幹刻意含糊——含糊處就是該問的地方。不要腦補成你熟的那題。
4. **題幹讀英文版([`PROMPTS_EN.md`](PROMPTS_EN.md) 底部),五問也用英文寫**
   ——面試時這一步整段是英文的,中文版只當對照。

---

## 卡 1:telemetry hub

數千台 node 各自持續回報遙測訊號(溫度、電壓、錯誤計數),一台聚合服務
收下來給儀表板讀。訊號總量遠超過你能存的記憶體。設計 ingestion 端。

## 卡 2:RPC gateway

一個 gateway 收 client 請求、轉發給後端服務,每個請求都必須有回應。
後端偶爾會慢下來,慢的時候請求還是持續進來。設計 gateway 的排隊與流控。

## 卡 3:market data feed

行情 feed 對每個 symbol 高頻推送報價 tick。策略端只在乎每個 symbol 的
**最新**報價;策略端讀取的速度時快時慢。設計 feed 與策略端之間那一層。

## 卡 4:log shipper

每台主機跑一個 agent,收本機所有程序的 log、送到遠端收集器。
網路每天會 flaky 幾次,每次幾秒到幾分鐘;應用程式寫 log 的呼叫不能被卡住。
設計 agent 的緩衝與送出。

## 卡 5:sensor bridge

單一硬體裝置經中斷/DMA 把訊號推進來,爆發時每秒百萬級;你的橋接程式
把訊號轉給上層消費者。裝置端沒有任何暫停機制。設計橋接層。

## 卡 6:health prober

對幾百台機器定期做 health check(TCP 連線 + 應用層 ping)。節點死掉要在
可預期的時間內標紅;prober 本身不能把目標機器打掛、也不能把自己撐爆。
設計排程與併發。
