// ═══ 背靠背 8/4 — 場一(25–30m;clarify → state 表 → 骨架,body 全 todo!)═══
//
// Problem 1 — Fanout Gateway
//
// You're building the message-fanout component of a small pub/sub gateway
// for our edge boxes. Rust, single file, tokio available.
//
// - Publishers connect over TCP and send newline-delimited messages.
// - Subscribers connect and receive every message published *after* they join.
// - A subscriber that falls behind must NOT slow down publishers or other
//   subscribers.
// - Subscribers come and go at any time; dead connections must get cleaned up.
// - Ordering from any single publisher must be preserved for every subscriber.
//
// Deliverable: clarify questions first (typed, English, in chat), then your
// state table (as comments here), then the skeleton with todo!() bodies.
//
// Part B (sizing — five-line format, whenever ready): 10,000 subscribers,
// aggregate publish rate 2,000 msgs/sec, average message 512 bytes, each
// subscriber holds a private buffer of the last 256 messages. Estimate the
// steady-state buffering memory and rule: does this design survive on a
// 4 GB edge box?
//
// For the user space, we have 10000 * (~128 Bytes(for the future task) + 256 Bytes) + 256 * 512 bytes(boardcast)
// + 10 * (~128 Bytes)
// = 10k * 384 Bytes + 128 * 1024 Bytes + 1KB = ~3MB + 128KB + 1KB
// so we can survive on 4GB edge box.
// 規則:裁決抄紙(寫成註解)、規則 read back、多段問編號逐答。

/*
* 1. so we will have some connection want to be publisher and some connection are subcribers?
* 2. New subscriber should read message from oldest or maybe some data we preserve, or even from newest message?
* 3. Ordering musy be preserved means that the order of its messages should keep?
* 4. So for every publisher we need to be read from subscriber in seperate? or they can store in same channel or something and get read from the channel?
* 5. How subscriber works? only when the subscriber connection send some operation code and we send data to them, or if there is new data, we make a consumer to consume the data?
* 6. I think we need to wait some operation code then send data to them, because the subscribers might up to 10000?
*/

// STATE:
// 1. I think I will use tokio::sync::boradcast to be the bridge for publisher / subscribers.
//      If we cannot use tokio, we can use a mpsc, 10 producers and 1 consumer to publish data and
//      consume
// 2. I might use tokio runtime to handle this problem
// 3. one thread to receive new connection and check if it's publisher or subscriber, spawn new task
//    for this connection
// 4. boradcast with 256 message length

// RULING: slow subscriber → bounded buffer, on Lagged → disconnect (no silent gaps)
// RULING: write timeout → kick; never retry write_all (TCP owns retransmission)
// RULING: SUB is receive-only after handshake; push model, no poll op-codes

async fn publish(mut stream: tokio::net::TcpStream, tx: tokio::sync::broadcast::Sender<Vec<u8>>) {
    // read buf
    // try to send in tx
}

async fn subscriber(
    mut stream: tokio::net::TcpStream,
    rx: tokio::sync::broadcast::Receiver<Vec<u8>>,
) {
    // try to read in rx
    // handle the case Lagged, if err return
    // try to send data
}

#[tokio::main]
async fn main() {
    // create a boardcast
    // bind and create tcp listener
    loop {
        // accept and check the connection
        // read and decode
        match todo!() {
            // PUB
                // tx = tx.clone()
                // tokio::spawn(async {publish}) 
            // SUB
                // rx = tx.subcribe() 
                // tokio::spawn(async {subscriber})
        }
        todo!()
    }
}

// ═══════════════════════════════════════════════════════════════════
// ═══ 場一批改(Claude,8/4)═══
//
// 【Part B】鏈對、基準錯、五行頭缺席:
// 1. ✗ 基準偷換:題面 "each subscriber holds a *private* buffer" 是考官的 Given。
//    → 題面版:10k × 256 × 512 B = 1.31 GB(4 GB 的 1/3,活但醜)
//    → 你的版:共享 ring 128 KiB + 10k 個 u64 游標 ~80 KB
//    滿分劇本 = 指出不一致(這本身就是 clarify)→ 兩個都算 → 亮 10,000× delta:
//    ✓ "The stated assumption is a private buffer per subscriber — my design
//       doesn't work that way. Want me to size the stated version, mine, or both?"
// 2. ✗ 五行頭沒用(點名兩次)。Given 行 = 不一致偵測器:重述到 "private buffer"
//    那條時,跟自己設計的矛盾當場現形。跳過 Given = 偵測器沒開機。格式不是儀式。
// 3. ✗ 漏 kernel socket send buffer:10k 條 × ~32 KiB ≈ 320 MB——user-space 總數
//    的 100 倍,而且慢 subscriber 的積壓物理上就住在 send buffer 裡。
//    ("for the user space" 的 scoping 意識記一分,但 verdict 問的是整台箱子。)
// 4. ✗ 漏 Sanity:出口頻寬 = 2k msg/s × 512 B × 10k subs = 10.24 GB/s ≫ 任何 NIC。
//    先死的是網卡不是記憶體;滿載全場 Lagged → kick 政策把整群踢光。senior 句:
//    "Memory survives; the NIC doesn't — the real fix is a rate/subscriber budget."
// 5. ✓ 記上:共享 ring 128 KiB ✓、task 開銷 MB 量級 ✓(實際 ~1 KiB/task → ~10 MB,
//    結論不變)、方向(記憶體活)✓。
//
// 【場一結案卡】
// Clarify  ✅ 六問命中:join 語意/ordering 範圍/push-pull,slow-sub 政策主雷自己拆
// 設計     ✅ broadcast 選型精準——Lagged 就是 disconnect 政策的現成實作
// 骨架     ✅ 一輪修達標:channel/listener 出 loop、spawn 現形、subscribe 位置=join 語意
// 真洞     ✗ retry 反射:對死連線疊 timeout/retry 機制 = scope creep 換皮。裁決句:
//          timeout→kick 可以;timeout→retry 不行(write_all 可能已送半包=撕裂幀;
//          TCP 自己會重傳——app 層 write_all 報錯 = 連線已死,沒有東西可 retry)
// 抄紙     ⚠ 三催才上牆;8/6 要變成裁決落地的「當下」動作,不是欠帳
// Part B   ⚠ 見上 1–4;鏈算對、基準錯、格式缺席
//
// 【broadcast 內臟(題外話沉澱)】
// 一份共享 ring + 每 receiver 自帶 u64 游標;slot 蓋絕對序號,讀前驗
// slot.pos == next,對不上 = Lagged——Vyukov per-slot sequence + 自由跑
// 計數器,同族思想。釋放時機 = min(最慢讀者讀完 rem→0 當場 drop,
// ring 滿被新訊息覆寫)——「sender 永不阻塞」的代價就寫在這一行。
// subscribe() 把游標生在當前 tail = 「join 之後才收」語意的實作位置。
// ═══════════════════════════════════════════════════════════════════
