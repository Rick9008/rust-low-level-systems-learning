//! mock_logring_0805_fixed —— 原稿「同形狀」最小修正版。
//!
//! 保留你的固定槽環架構(head/tail/len + byte 帳),只修錯不換形;
//! 每處修改掛 ✔ 標籤,編號對回 `mock_logring_0805_attempt.rs` 檔頭批改總表。
//! 換形狀的參考解(VecDeque)在 `sol_logring_0805.rs`——兩份對讀:
//! 這份看「錯在哪、怎麼修」,那份看「形狀選對後整族問題直接消失」。
//!
//! 驗證:rustc --edition 2024 編過 + smoke 3/3 ✓(與 sol 同三段)。

use std::sync::{Arc, Mutex}; // ✔ 補 use 塊(原稿缺席)

const SLOTS: usize = 1024;
const BYTE_BUDGET: usize = 1024;
// 槽數帳:預算 1024B / 最小條目 8B → 最多 128 條 < 1024 槽,count-full 到不了,
// 所以 push 前不用檢查槽滿——這句要能講出來,不然 1024 槽就是魔數。

struct Entry {
    log_id: u32,
    size: u32, // ✔ 洞④根治:usize → u32 對齊 C 端 uint32_t,to_be_bytes 就是 4 bytes
    body: Vec<u8>,
}

impl Entry {
    fn new(log_id: u32, size: u32, body: &[u8]) -> Self {
        // ✔ ⚠項:size 欄位留著(照你的原設),但至少上一道鎖;
        //   sol 版的做法是整個欄位砍掉,讓不一致不可表示。
        debug_assert_eq!(size as usize, body.len());
        Self {
            log_id,
            size,
            body: body.to_vec(), // ✔ 砍掉 .clone()(clone 引用=白做工)
        }
    }

    fn entry_size(&self) -> usize {
        8 + self.body.len() // ✔ E0425 補 self.;✔ 尾 +4 砍掉——wire = log_id(4)+size(4)+body,
                            //   對齊題面草圖「4+4+256」,ShowLog 的 -8 魔數同步陪葬
    }
}

struct Inner {
    head: usize,
    tail: usize,
    len: usize,
    total_bytes: usize, // ✔ ⚠項:原名 entry_size 和 Entry::entry_size() 同名不同義,改誠實的名字
    entry_slots: Vec<Option<Entry>>,
}

impl Inner {
    fn new() -> Self {
        Self {
            head: 0,
            tail: 0,
            len: 0,
            total_bytes: 0,
            entry_slots: (0..SLOTS).map(|_| None).collect(), // ✔ 幻想 API Vec::new_with →
                                                             //   collect 形(免 Clone bound;
                                                             //   另一解 vec![None; N] 需 Entry: Clone)
        }
    }
}

#[derive(Clone)] // ✔ derive(Send) 砍——auto trait 不可 derive,Arc<Mutex<_>> 本來就 Send+Sync
struct LogRing {
    inner: Arc<Mutex<Inner>>,
}

impl LogRing {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::new())),
        }
    }

    fn add_log(&self, entry: Entry) {
        // ✔ C++ 參數序 Entry entry → entry: Entry;✔ &mut self → &self(Clone 把手+內部可變性);
        // ✔ 命名 snake_case
        let mut inner = self.inner.lock().unwrap(); // ✔ E0596:要改欄位 → guard 綁 mut
        let es = entry.entry_size();
        if es > BYTE_BUDGET {
            return; // ✔ 護欄:單筆自身超預算(body ≤ 256 到不了)——沒這行下面會清空後無窮迴圈
        }
        while inner.total_bytes + es > BYTE_BUDGET {
            // ✔ >= → >(「exceeds 1024」語意,剛好 1024 不淘汰;上場先問)
            let t = inner.tail;
            let dead = inner.entry_slots[t].take().unwrap();
            // ✔ E0507:take() 拿走所有權順便清槽;✔ E0502:先取 t、dead 再改帳=拆行,
            //   guard 上同一句左改右讀必撞
            inner.total_bytes -= dead.entry_size();
            inner.tail = (inner.tail + 1) % SLOTS;
            inner.len -= 1;
        }
        let h = inner.head;
        inner.entry_slots[h] = Some(entry);
        inner.head = (inner.head + 1) % SLOTS;
        inner.len += 1;
        inner.total_bytes += es; // ✔ 邏輯主洞①:帳要有進——原稿全檔最貴的一行
    }

    fn show_log(&self, dma_buffer: &mut [u8]) -> usize {
        // ✔ 洞⑥:uint32_t &size out-param 的 Rust 形 = 回傳實際寫入長度
        let inner = self.inner.lock().unwrap(); // 只讀不用 mut(原稿這行本來就對)
        let mut written = 0; // ✔ 洞②:sz → written,宣 mut、迴圈裡真的前進
        let mut idx = (inner.head + SLOTS - 1) % SLOTS;
        // ✔ 洞③:環上退格 canonical 形——head==0 不再 underflow;len==0 時 for 根本不進場
        for _ in 0..inner.len {
            // ✔ 洞②:delta 手動計數 → 有界 for,「忘了 ++」這種洞不可表示
            let entry = inner.entry_slots[idx].as_ref().unwrap(); // ✔ E0507:只讀 → as_ref()
            let es = entry.entry_size();
            if written + es > dma_buffer.len() {
                break; // 塞不下就停(AddLog 已保證總量 ≤ 1024,這是對小 buffer 的防禦)
            }
            // ✔ 輸出補上 log_id(原稿 -8 魔數把它踢掉了,與題面草圖矛盾);
            // ✔ 逐 byte 迴圈 → copy_from_slice;endian 維持你選的 BE(clarify 點)
            dma_buffer[written..written + 4].copy_from_slice(&entry.log_id.to_be_bytes());
            dma_buffer[written + 4..written + 8].copy_from_slice(&entry.size.to_be_bytes());
            // ✔ 洞④:size 已是 u32,to_be_bytes = 恰好 4 bytes,高位全零問題消失
            dma_buffer[written + 8..written + 8 + entry.body.len()].copy_from_slice(&entry.body);
            written += es; // ✔ 洞⑤:寫入起點 = written 本人,ptr = sz + 1 的 off-by-one 砍掉
            idx = (idx + SLOTS - 1) % SLOTS; // ✔ 洞③:倒走同樣走 modulo,跨 0 不炸
        }
        written
    }
}

fn main() {
    // ✔ P5:smoke 補齊(與 sol 同三段)——洞②③⑤全是跑一圈就炸的,一筆資料就能抓到。

    // ── smoke 1:順序 + 精確 bytes ── 最新(id=2)在最低位址。
    let ring = LogRing::new();
    ring.add_log(Entry::new(1, 4, &[0xAA, 0xBB, 0xCC, 0xDD]));
    ring.add_log(Entry::new(2, 4, &[0x11, 0x22, 0x33, 0x44]));
    let mut buf = [0u8; 1024];
    let n = ring.show_log(&mut buf);
    assert_eq!(n, 24); // (8+4) × 2
    assert_eq!(&buf[0..4], &2u32.to_be_bytes());
    assert_eq!(&buf[4..8], &4u32.to_be_bytes());
    assert_eq!(&buf[8..12], &[0x11, 0x22, 0x33, 0x44]);
    assert_eq!(&buf[12..16], &1u32.to_be_bytes());

    // ── smoke 2:淘汰 ── 264B × 4 = 1056 > 1024 → 最舊(id=10)被踢。
    let ring = LogRing::new();
    for id in 10u32..14 {
        ring.add_log(Entry::new(id, 256, &[id as u8; 256]));
    }
    let n = ring.show_log(&mut buf);
    assert_eq!(n, 3 * 264);
    assert_eq!(&buf[n - 264..n - 260], &11u32.to_be_bytes()); // 最舊倖存者在最高位址

    // ── smoke 3:小 buffer ── 只裝得下最新一筆。
    let mut small = [0u8; 300];
    let n = ring.show_log(&mut small);
    assert_eq!(n, 264);
    assert_eq!(&small[0..4], &13u32.to_be_bytes());

    println!("smoke 3/3 ✓");
}
