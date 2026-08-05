//! mock_logring_0805_attempt —— 8/5 晚 mock 題原稿 + 逐行批改(不可編譯,存檔用)。
//! 參考解在 `sol_logring_0805.rs`(rustc 驗證 + smoke 3/3 ✓)。
//!
//! ══ 批改總表 ══════════════════════════════════════════════════════
//! 【編譯層】rustc 實測兩波共 11 條:
//!   第一波 parse/resolve ×6:C++ 參數序、derive(Send)、少 self.、
//!     幻想 API Vec::new_with、unwarp typo、缺 use 塊。
//!   第二波 borrow/move ×5:guard 沒綁 mut(E0596)、Option 槽 unwrap
//!     move-out ×3(E0507)、同句左改右讀 guard 打架(E0502)。
//! 【邏輯層】6 洞,前 4 條致命:
//!   ① total 帳只出不進(少 += es)→ 淘汰永不觸發;真觸發則 usize underflow。
//!   ② ShowLog 的 delta/sz 宣告後全程不動 → 無窮迴圈,且條件每圈讀同一格。
//!   ③ head-1 / index-delta 都沒走 modulo → head==0 或倒走跨 0 直接 panic。
//!   ④ usize::to_be_bytes = 8 bytes,取前 4 = 高位全零 → size 欄恆寫 0。
//!   ⑤ ptr = sz + 1 off-by-one,第 0 byte 永遠空著。
//!   ⑥ 題面 uint32_t &size out-param 沒對應(Rust 形 = 回傳 usize)。
//! 【形狀層】byte 預算、條目數變動 → VecDeque + running 總帳;
//!   固定 1024 槽 Option 環是把 a 題肌肉硬套到不對的預算軸,
//!   E0507/E0502/modulo 三族錯全是這個選型的衍生成本。
//! 【五支柱稿面判定】P2 abstract ⚠(FIFO/newest-first 方向全對,狀態選型過重)
//!   P3 iterate ✗(兩函式各有致命洞)| P5 dry-run ✗(smoke 缺席,②③⑤全是
//!   dry-run 一圈就會炸的洞)| P1/P4 口頭項,稿面無據不計。
//! 【定性】架構直覺對(eviction 方向、FIFO、newest at lowest address 都對),
//!   Rust 手感被 C++ mock 串味 + 深夜放大;明天全天 Rust 無此切換成本。
//!   帶進 §E 的只有兩條:guard 要 let mut、Option 槽取值 as_ref/take 不 unwrap。
//! ═══════════════════════════════════════════════════════════════════

// ✗【缺 use 塊】Arc/Mutex 沒 import:use std::sync::{Arc, Mutex};
//   (8/4 才記過「use 塊忘光=衰退訊號」,明早 §E 掃一眼)

struct Entry {
    log_id: u32,
    size: usize, // ⚠ C 端是 uint32_t;而且 size≠body.len() 的不一致變成可表示
    body: Vec<u8>,
}

impl Entry {
    fn new(log_id: u32, size: usize, body: &[u8]) -> Self {
        Self {
            log_id,
            size,
            body: body.clone().to_vec(), // ⚠ clone 到的是引用本身,白做工;body.to_vec() 即可
        }
    }

    fn entry_size(&self) -> usize {
        4 + 4 + body.len() + 4 // ✗ E0425 少 self.;⚠ 尾 +4 來歷不明——配 ShowLog 的 -8
                               //   等於默默把 log_id 踢出輸出,和題面草圖「4+4+256」矛盾
    }
}

struct Inner {
    head: usize,
    tail: usize,
    len: usize,
    entry_size: usize, // ⚠ 和 Entry::entry_size() 同名不同義,讀者要猜;total_bytes 之類更誠實
    entry_slots: Vec<Option<Entry>>, // ⚠【形狀主裁】byte 預算下固定槽環買不到東西,VecDeque 全免
}

impl Inner {
    fn new() -> Self {
        Self {
            head: 0,
            tail: 0,
            len: 0,
            entry_size: 0,
            entry_slots: Vec::new_with(1024, || None), // ✗ 幻想 API;正解 vec![None; 1024](需
                                                       //   Entry: Clone)或 resize_with(1024, || None)
        }
    }
}

#[derive(Send, Clone)] // ✗ Send 是 auto trait 不可 derive;Arc<Mutex<_>> 本來就自動 Send+Sync,
                       //   什麼都不用寫,留 #[derive(Clone)] 就好
struct LogRing {
    inner: Arc<Mutex<Inner>>,
}

impl LogRing {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::new())),
        }
    }
    fn AddLog(&mut self, Entry entry) { // ✗ C++ 參數序,Rust 是 entry: Entry(串味本尊)
                                        // ⚠ Clone 把手 + Mutex → 慣例 &self,內部可變性就是為了這個
                                        // ⚠ 命名慣例 snake_case:add_log(warning 而已,說一句就好)
        let inner = self.inner.lock().unwrap(); // ✗ E0596 之後要改欄位 → let mut inner
        let es = entry.entry_size();
        while es + inner.entry_size >= 1024 { // ⚠ 邊界 clarify:「exceeds」→ 用 >;
                                              // ✗ 主洞①的共犯:entry_size 永遠 0,這迴圈永不進;
                                              // ✗ 單筆 >1024 時(此題到不了)會無窮迴圈+underflow,
                                              //   缺 !is_empty() 護欄
            inner.entry_size -= inner.entry_slots[inner.tail].unwrap().entry_size();
            // ✗ E0507 unwrap 按值吃掉 Option,Vec 索引處搬不出來 → 淘汰語意用 .take()(順便清槽)
            // ✗ E0502 同一句左改右讀,透過 guard deref 打架 → 拆兩行:先取值再改帳
            inner.tail = (inner.tail + 1) % 1024;
            inner.len -= 1;
        }
        inner.entry_slots[inner.head] = Some(entry);
        inner.head = (inner.head + 1) % 1024;
        inner.len += 1;
        // ✗【邏輯主洞①】少了 inner.entry_size += es;——帳只出不進,淘汰永不觸發
    }

    fn ShowLog(&self, dma_buffer: &mut [u8]) { // ✗【洞⑥】題面 uint32_t &size out-param 沒對應
                                               //   → Rust 形狀 = 回傳 -> usize(實際寫入長度)
        let inner = self.inner.lock().unwrap(); // ✓ 只讀,不用 mut,這行對
        // dma_buffer's length is 4
        // 4 bytes: 0 0 0 4
        let maximum_sz = dma_buffer.len();
        let index = inner.head - 1; // ✗【洞③】head==0 直接 usize underflow panic;
                                    //   環上退格 canonical 形 = (head + cap - 1) % cap(§F 才打過)
        let delta = 0; // ✗【洞②】沒 mut、迴圈裡也從不遞增
        let sz = 0;    // ✗ 同上 → 條件永遠用初值,無窮迴圈
        // TODO for index checking
        while delta < inner.len  && maximum_sz > sz + (inner.entry_slots[index].unwrap().entry_size() - 8) {
            // ✗ E0507 同前;⚠ 條件每圈讀的是固定 index 那格,不是 index-delta;
            // ⚠ -8 魔數 = 檔頭 +4 的連鎖(把 log_id 踢出輸出),兩處要一起裁
            let ptr = sz + 1; // ✗【洞⑤】off-by-one,第 0 byte 永遠空著;應為 sz
            let entry = inner.entry_slots[index - delta].unwarp();
            // ✗ unwarp typo(E0599);✗ E0507;✗【洞③】index-delta 倒走跨 0 沒 modulo
            let num_bytes = entry.size.to_be_bytes();
            // ✗【洞④】size 是 usize → to_be_bytes 給 [u8; 8],下面取前 4 = 高位全零,
            //   寫進 buffer 的 size 恆為 0 → (entry.size as u32).to_be_bytes() 或欄位改 u32
            for offset in 0..4 {
                dma_buffer[ptr + offset] = num_bytes[offset]; // ⚠ 逐 byte 迴圈可用 copy_from_slice
            }
            for offset in 0..entry.body.len() {
                dma_buffer[ptr + 4 + offset] = entry.body[offset];
            }
            // ✗【洞②】迴圈尾少 delta += 1; sz += 寫入量;(以及 index 的環上遞減)
        }
    }
}


fn main() {
    println!("Hello LeetCoder"); // ✗【P5】smoke 缺席——洞②③⑤全是自寫一筆資料跑一圈就會炸的
}
