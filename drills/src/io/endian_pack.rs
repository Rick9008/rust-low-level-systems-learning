//! drill:endian_pack —— 位元組序與位元打包(c 題 framer + e2 token 的共用肌肉)。
//!
//! 為什麼獨立成 drill(2026-07-23 凌晨新增):BE/LE、`from_*_bytes` 家族、
//! pack/unpack 的 mask 寬度,三者都是「懂了但手不熟」的高頻手滑區——
//! c#1 的 `u32 len (BE)`、e2 的 `(gen << 32) | idx` 都踩在上面
//! (e2#1 實傷:mask 寫 `(1 << 31) - 1` 少一 bit,fd ≥ 2³¹ alias 到低位)。
//!
//! 記法(先背這三句,再動手):
//! 1. **BE = big end first,高位 byte 在前** = network byte order(hw_bridge 協定用它);
//!    LE = 低位在前 = x86 本機序。
//! 2. `from_be_bytes` 的 be 說的是 **bytes 在 wire 上的序**,不是機器的序——
//!    同一行 code 在任何機器上都對,swap 是編譯器的事。
//! 3. byte slice 沒有對齊問題:`&buf[3..7]` 照樣 `from_be_bytes`,
//!    這正是不能把 wire bytes 直接 transmute 成 struct 的理由之一
//!    (另一個是 padding/endianness 不可攜)。
//!
//! 已給:`Header` struct 與測試。要填:八個函式。
//! 填綠(移光 `#[ignore]`)後,7/27 骨架默寫抽查會抽「length-prefix 解析 3 行
//! + pack/unpack + mask」——目標是那時全部默得出來。

/// 混合欄位 wire header(佈局見 `parse_header` 的 spec;全部 BE)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub magic: u16,
    pub flags: u8,
    pub len: u32,
}

/// spec:從 `buf[at..at+4]` 讀一個 big-endian u32。
///
/// 1. `at + 4` 可能 overflow:`let end = at.checked_add(4)?;`
/// 2. 範圍取 slice 用 `buf.get(at..end)?`——出界回 `None`,不 panic。
/// 3. `u32::from_be_bytes(bytes.try_into().expect("4 bytes"))`——
///    slice → `[u8; 4]` 用 `try_into`(pad 是 1.92,`as_array` 還不穩定,
///    7/20 已踩過本機 1.91 的坑;`expect` 合法:長度剛驗過)。
///
/// 這三行就是 c 題 framer 的第一塊肌肉。
pub fn read_u32_be(buf: &[u8], at: usize) -> Option<u32> {
    let _ = (buf, at);
    todo!("spec: checked_add → get(range) → from_be_bytes(try_into)")
}

/// spec:同 `read_u32_be`,但 little-endian(`from_le_bytes`)。
pub fn read_u32_le(buf: &[u8], at: usize) -> Option<u32> {
    let _ = (buf, at);
    todo!("spec: 只差一個字:from_le_bytes")
}

/// spec:**不准用 `from_be_bytes`**,手動 shift 拼出 BE u32——證明你懂 API 在做什麼。
///
/// BE = 高位在前:`b[0]` 是最高位 byte。
/// `(b[0] as u32) << 24 | (b[1] as u32) << 16 | (b[2] as u32) << 8 | (b[3] as u32)`
/// 陷阱:先 shift 再 cast(`(b[0] << 24) as u32`)是錯的——u8 shift 24 直接歸零/panic。
pub fn u32_be_manual(b: [u8; 4]) -> u32 {
    let _ = b;
    todo!("spec: 每個 byte 先 as u32 再 shift,高位 byte shift 最多")
}

/// spec:把 `v` 以 big-endian 附加到 `out` 尾端。
/// `out.extend_from_slice(&v.to_be_bytes())`——encode 側的一行肌肉
/// (hw_bridge `encode` 寫 len 欄位就是它)。
pub fn write_u32_be(out: &mut Vec<u8>, v: u32) {
    let _ = (out, v);
    todo!("spec: extend_from_slice(&v.to_be_bytes())")
}

/// spec:從 `buf[at..at+2]` 讀 little-endian **i16**(有號!)。
///
/// 正解:`i16::from_le_bytes`——符號擴展是免費的,bytes 進來就是補數。
/// 陷阱(這題存在的理由):自己拼 `(b[1] as i16) << 8 | b[0] as i16` 時,
/// 若 `b[0] as i16` 走過 `as i8` 會被符號擴展成 0xFF__ 污染高位;
/// 直接 u8→i16 則安全(零擴展)。用 API 就沒這些心智稅。
pub fn read_i16_le(buf: &[u8], at: usize) -> Option<i16> {
    let _ = (buf, at);
    todo!("spec: 同 read_u32_le 的三行,型別換 i16、寬度換 2")
}

/// spec:把 `(generation, idx)` 打包進一個 u64:generation 佔高 32 bit、idx 佔低 32 bit。
/// `((generation as u64) << 32) | (idx as u64)`——e2 token 的本體
/// (kernel 只還你一個 u64,兩個欄位得自己擠)。
///
/// 順帶的 pad 坑:**`gen` 是 edition 2024 的保留字**(gen block),
/// 參數名寫 `gen` 直接 syntax error——e2 場上請用 `generation` 或 `slot_gen`。
pub fn pack_token(generation: u32, idx: u32) -> u64 {
    let _ = (generation, idx);
    todo!("spec: 高位 shift 32,低位 or 進來;兩邊都先 as u64")
}

/// spec:`pack_token` 的逆:回 `(gen, idx)`。
///
/// gen = `(tok >> 32) as u32`;idx 兩種寫法擇一:
/// - `(tok & 0xFFFF_FFFF) as u32`——mask 必須**足 32 bit**
///   (e2#1 實傷:寫 `(1 << 31) - 1` 少一 bit → fd = 0x8000_0000 被 alias 成 0);
/// - `tok as u32`——截斷 cast 本身就是 mask,少一個出錯點(推薦寫這個,
///   但上面那行的教訓要講得出來)。
pub fn unpack_token(tok: u64) -> (u32, u32) {
    let _ = tok;
    todo!("spec: >> 32 取高、as u32 截低;mask 寫法的一 bit 教訓見上")
}

/// spec:解析 7-byte 混合 header(全 BE):
///
/// ```text
/// offset 0..2  magic: u16 BE
/// offset 2     flags: u8
/// offset 3..7  len:   u32 BE   ← 故意不對齊(offset 3),slice 不在乎
/// ```
///
/// 不足 7 bytes → `None`。`u16::from_be_bytes` 同家族,寬度換 2。
/// 這就是「多欄位 header 逐欄切」的完整形:c/d 題若 header 長胖,照這個模子。
pub fn parse_header(buf: &[u8]) -> Option<Header> {
    let _ = buf;
    todo!("spec: get(..7)? 先擋短;再逐欄 from_be_bytes / 直取")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// boundary:BE vs LE 同四 bytes 讀出不同值——背這組數就背住方向。
    /// trace:bytes [0x12,0x34,0x56,0x78]:BE 高位在前 → 0x1234_5678;
    /// LE 低位在前 → 0x7856_3412。offset 1 起讀、長度不足 → None。
    #[test]
    #[ignore = "填完 read_u32_be/le 後移除"]
    fn be_le_read_and_bounds() {
        let buf = [0x12u8, 0x34, 0x56, 0x78, 0x9A];
        assert_eq!(read_u32_be(&buf, 0), Some(0x1234_5678));
        assert_eq!(read_u32_le(&buf, 0), Some(0x7856_3412));
        assert_eq!(read_u32_be(&buf, 1), Some(0x3456_789A));
        assert_eq!(read_u32_be(&buf, 2), None); // 只剩 3 bytes
        assert_eq!(read_u32_be(&buf, usize::MAX), None); // checked_add 擋 overflow
    }

    /// 手動 shift 必須跟 API 逐值一致(含最高位 byte ≥ 0x80 的 case)。
    #[test]
    #[ignore = "填完 u32_be_manual 後移除"]
    fn manual_matches_api() {
        for bytes in [
            [0x00, 0x00, 0x00, 0x01],
            [0x12, 0x34, 0x56, 0x78],
            [0x80, 0x00, 0x00, 0x00], // 最高 bit:先 cast 再 shift 才活得下來
            [0xFF, 0xFF, 0xFF, 0xFF],
        ] {
            assert_eq!(u32_be_manual(bytes), u32::from_be_bytes(bytes));
        }
    }

    /// encode → decode roundtrip;寫出的 bytes 高位在前。
    /// trace:0x0000_0105 → [0x00,0x00,0x01,0x05](hw_bridge len 欄位同款)。
    #[test]
    #[ignore = "填完 write_u32_be 後移除"]
    fn write_be_roundtrip() {
        let mut out = Vec::new();
        write_u32_be(&mut out, 0x0000_0105);
        assert_eq!(out, [0x00, 0x00, 0x01, 0x05]);
        write_u32_be(&mut out, u32::MAX);
        assert_eq!(read_u32_be(&out, 4), Some(u32::MAX));
    }

    /// 符號擴展:0xFFFF → -1;0x8000 → i16::MIN。LE = 低位 byte 在前。
    /// trace:[0x00, 0x80] LE → 0x8000 → 補數解讀 = -32768。
    #[test]
    #[ignore = "填完 read_i16_le 後移除"]
    fn i16_sign_extension() {
        assert_eq!(read_i16_le(&[0xFF, 0xFF], 0), Some(-1));
        assert_eq!(read_i16_le(&[0x00, 0x80], 0), Some(i16::MIN));
        assert_eq!(read_i16_le(&[0xFE, 0xFF], 0), Some(-2));
        assert_eq!(read_i16_le(&[0xFF], 0), None);
    }

    /// e2 傷疤直測:idx 最高 bit 有值(0x8000_0000)必須 roundtrip 不 alias。
    /// `(1<<31)-1` 那種少一 bit 的 mask 在這條測試上必死。
    #[test]
    #[ignore = "填完 pack/unpack_token 後移除"]
    fn token_roundtrip_high_bit() {
        for (g, i) in [
            (0u32, 0u32),
            (7, 0x8000_0000), // e2#1 的 alias case 本尊
            (u32::MAX, u32::MAX),
            (1, 1),
        ] {
            let tok = pack_token(g, i);
            assert_eq!(unpack_token(tok), (g, i));
        }
        // 兩欄位互不滲漏:gen 全 1 時 idx 讀出來必須還是 0。
        assert_eq!(unpack_token(pack_token(u32::MAX, 0)), (u32::MAX, 0));
    }

    /// 混合 header 逐欄切(len 落在 offset 3,不對齊照讀)。
    /// trace:magic 0xCAFE → [0xCA,0xFE];len 0x0000_0105 → [0x00,0x00,0x01,0x05]。
    #[test]
    #[ignore = "填完 parse_header 後移除"]
    fn header_parse_and_short_input() {
        let wire = [0xCAu8, 0xFE, 0x01, 0x00, 0x00, 0x01, 0x05];
        assert_eq!(
            parse_header(&wire),
            Some(Header {
                magic: 0xCAFE,
                flags: 0x01,
                len: 0x0000_0105,
            })
        );
        assert_eq!(parse_header(&wire[..6]), None); // 差一 byte 就是 None
    }
}
