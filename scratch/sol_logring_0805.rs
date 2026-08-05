//! sol_logring_0805 —— 8/5 晚 mock 題(embedded telemetry LogRing)參考解。
//!
//! 對照原稿(當場手寫版)的修正課,五條:
//! 1. 【形狀】byte 預算下條目數變動,固定 1024 槽 Option 環買不到東西——
//!    VecDeque 本身就是環(push_back 進、pop_front 淘汰),head/tail/len/
//!    Option 槽/modulo 全部消失,E0507/E0502 那一族錯誤跟著消失。
//! 2. 【帳】total_bytes 進出都要記(原稿只出不進 → 永不淘汰,真觸發還會
//!    usize underflow)。「只出不進」= sim m tries 帳傷疤的鏡像。
//! 3. 【所有權】淘汰用 pop_front() 直接拿走所有權;槽形設計才需要
//!    as_ref()/take(),deque 連這個問題都沒有。
//! 4. 【欄位】C 端 struct 的 size 欄位不搬進 Rust——body.len() 是唯一真相,
//!    「size ≠ body.len()」的不一致直接做成不可表示(sim m ReqState 同課)。
//! 5. 【介面】C 的 `uint32_t &size` out-param 在 Rust 的形狀 = 回傳值。
//!
//! clarify 點(上場要問的,這裡取的預設):
//! - 輸出含不含 log_id?題面草圖「0 256 body : 4+4+256」→ 含,每筆 = 8 + body。
//! - endianness?原稿註解「0 0 0 4」= big-endian,照用。
//! - 「exceeds 1024」邊界?取 >(剛好 1024 不淘汰)。
//! - 單筆自身超預算?丟棄(body ≤ 256 下到不了,防禦性守住不無窮迴圈)。
//!
//! 驗證:rustc --edition 2024 編過 + main 內 smoke 3 段 assert 全過。

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// 題面:現存 log 總量超過 1024 bytes 就淘汰最舊(FIFO)。
const BYTE_BUDGET: usize = 1024;

struct Entry {
    log_id: u32,
    body: Vec<u8>,
}

impl Entry {
    fn new(log_id: u32, body: &[u8]) -> Self {
        // &[u8] 直接 .to_vec();.clone() 只會複製引用本身,是白做工。
        Self {
            log_id,
            body: body.to_vec(),
        }
    }

    /// 序列化後佔用:log_id(4) + size(4) + body。
    fn wire_size(&self) -> usize {
        8 + self.body.len()
    }
}

struct Inner {
    /// FIFO:push_back 進、pop_front 淘汰。
    entries: VecDeque<Entry>,
    /// 現存條目 wire_size 總和;add 時 +、evict 時 -,兩邊都要記帳。
    total_bytes: usize,
}

/// Clone 出去的是共享把手;Arc<Mutex<_>> 已自動 Send+Sync,
/// 不用(也不能)derive(Send)——auto trait 不可 derive。
#[derive(Clone)]
struct LogRing {
    inner: Arc<Mutex<Inner>>,
}

impl LogRing {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                entries: VecDeque::new(),
                total_bytes: 0,
            })),
        }
    }

    /// 對應 C 的 AddLog。共享把手 + 內部可變性 → &self,不是 &mut self。
    fn add_log(&self, entry: Entry) {
        // 之後要改欄位 → guard 綁定必須 mut(E0596 的處方)。
        let mut inner = self.inner.lock().unwrap();
        let es = entry.wire_size();
        // 借用課:淘汰拆成「先 pop 取值、再改帳」兩行——透過 MutexGuard 的
        // deref,同一句左改右讀會撞(E0502),拆行永遠安全。
        while inner.total_bytes + es > BYTE_BUDGET && !inner.entries.is_empty() {
            let dead = inner.entries.pop_front().unwrap();
            inner.total_bytes -= dead.wire_size();
        }
        if es > BYTE_BUDGET {
            return; // 單筆自身超預算:丟棄(clarify 預設,見檔頭)。
        }
        inner.total_bytes += es; // 原稿漏的那行:帳要有進。
        inner.entries.push_back(entry);
    }

    /// 對應 C 的 ShowLog(uint8_t*, uint32_t&)。最新的放最低位址 → 從 deque
    /// 尾端往前走;回傳實際寫入長度。塞不下就停(AddLog 已保證總量 ≤ 1024,
    /// 這是對 caller 給小 buffer 的防禦)。
    fn show_log(&self, dma_buffer: &mut [u8]) -> usize {
        let inner = self.inner.lock().unwrap(); // 只讀,不用 mut。
        let mut written = 0;
        for entry in inner.entries.iter().rev() {
            let es = entry.wire_size();
            if written + es > dma_buffer.len() {
                break;
            }
            // big-endian(檔頭 clarify);body.len() ≤ 256 → as u32 不截斷。
            dma_buffer[written..written + 4].copy_from_slice(&entry.log_id.to_be_bytes());
            dma_buffer[written + 4..written + 8]
                .copy_from_slice(&(entry.body.len() as u32).to_be_bytes());
            dma_buffer[written + 8..written + 8 + entry.body.len()].copy_from_slice(&entry.body);
            written += es;
        }
        written
    }
}

fn main() {
    // ── smoke 1:順序 + 精確 bytes ── 兩筆小 log,最新(id=2)在最低位址。
    let ring = LogRing::new();
    ring.add_log(Entry::new(1, &[0xAA, 0xBB, 0xCC, 0xDD]));
    ring.add_log(Entry::new(2, &[0x11, 0x22, 0x33, 0x44]));
    let mut buf = [0u8; 1024];
    let n = ring.show_log(&mut buf);
    assert_eq!(n, 24); // (8+4) × 2
    assert_eq!(&buf[0..4], &2u32.to_be_bytes());
    assert_eq!(&buf[4..8], &4u32.to_be_bytes());
    assert_eq!(&buf[8..12], &[0x11, 0x22, 0x33, 0x44]);
    assert_eq!(&buf[12..16], &1u32.to_be_bytes()); // 舊的排後面

    // ── smoke 2:淘汰 ── 264B × 4 = 1056 > 1024 → 最舊(id=10)被踢。
    let ring = LogRing::new();
    for id in 10u32..14 {
        ring.add_log(Entry::new(id, &[id as u8; 256]));
    }
    let n = ring.show_log(&mut buf);
    assert_eq!(n, 3 * 264);
    // 最舊倖存者 id=11 在最高位址那格。
    assert_eq!(&buf[n - 264..n - 260], &11u32.to_be_bytes());

    // ── smoke 3:小 buffer ── 只裝得下最新一筆,寫一筆就停。
    let mut small = [0u8; 300];
    let n = ring.show_log(&mut small);
    assert_eq!(n, 264);
    assert_eq!(&small[0..4], &13u32.to_be_bytes());

    println!("smoke 3/3 ✓");
}
