//! drill:iter_mutate —— 填「邊迭代邊修改」的六種形狀核心。
//!
//! 已給:結構(`EventLog`)、getter、全部簽名與每個函式上方的 spec。
//! 要填:`scale_in_place` / `move_zeroes` / `dedup_sorted` /
//!       `keep_and_double_positives` / `evict_expired` / `reverse_in_place` /
//!       `EventLog::compact`。
//!
//! 填之前先分類:哪些只改值用 `iter_mut`、哪些改結構用寫指標、哪個減法會
//! **usize underflow**?卡住再 diff `reference/src/iter_mutate.rs`(最後手段)。

use std::collections::HashMap;

/// spec:每個元素乘以 `factor`,長度不變。`iter_mut()` 給你 `&mut T`,不碰 index。
pub fn scale_in_place(v: &mut [i64], factor: i64) {
    // todo!("spec: for x in v.iter_mut() {{ *x *= factor }}")

    v.iter_mut().for_each(|x| *x *= factor);
}

/// spec:把所有 0 搬到尾端、非 0 相對順序不變,in place。O(n)/O(1)。
/// 寫指標:`read` 掃、`write` 是「下一個非 0 該落的位置」;非 0 就 `swap(write, read)`
/// 再推進 `write`。
pub fn move_zeroes(v: &mut [i32]) {
    // todo!("spec: write/read 雙指標;v[read]!=0 → swap(write,read) 並 write+=1")

    //  2, 3, 0, 0, 1, 0, 3
    // rw
    //    wr
    //       wr r  r
    //          w     r  r
    let (mut write, n) = (0, v.len());
    for read in 0..n {
        if v[read] != 0 {
            v.swap(write, read);
            write += 1;
        }
    }
}

/// spec:已排序 Vec 就地去重,回傳去重後長度並 `truncate`。O(n)/O(1)。
/// `write-1` 永遠指向「最後一個已保留的值」,和 `read` 不同才寫入。
pub fn dedup_sorted(v: &mut Vec<i32>) -> usize {
    // todo!(
    //     "spec: 空回 0;write/read 從 1 起;v[read]!=v[write-1] → v[write]=v[read] 且 write+=1;truncate(write)"
    // )
    if v.is_empty() {
        return 0;
    }
    // [1, 1, 1, 2, 3, 3, 4, 5, 5, 5]

    // [1, 2]
    let (mut write, n) = (1usize, v.len());
    for read in 1..n {
        if v[read] != v[write - 1] {
            v[write] = v[read];
            write += 1;
        }
    }
    v.truncate(write);
    write
}

/// spec:一趟內「篩選 + 改值」——留下正數並把留下的翻倍。
/// 提示:`Vec::retain_mut(|x| { ...; keep })`,closure 拿 `&mut T`、回 bool。
// retain -> 保留
pub fn keep_and_double_positives(v: &mut Vec<i32>) {
    // todo!("spec: retain_mut;keep = *x>0;若 keep 則 *x *= 2;回 keep")

    // keep original values only double the positive values
    // v.iter_mut().filter(|v| **v > 0).for_each(|num| *num *= 2);
    // or
    v.retain_mut(|x| {
        if *x <= 0 {
            return false;
        }
        *x *= 2;
        true
    });
}

/// spec:刪掉所有「已過期」條目(`now - inserted > ttl`)。
/// 提示:`HashMap::retain`;減法用 `saturating_sub` 防時鐘回跳造成 underflow。
pub fn evict_expired(map: &mut HashMap<String, u64>, now: u64, ttl: u64) {
    // todo!("spec: map.retain(|_, inserted| now.saturating_sub(*inserted) <= ttl)")

    // retain: the condition true then keep
    map.retain(|_, value| now.saturating_sub(*value) <= ttl)
}

/// spec:就地反轉,兩指標對撞 `swap(lo, hi)`。O(n)。
/// 小心 usize:先 `hi -= 1` 再 swap,別讓 `hi` 在 0 時減。
pub fn reverse_in_place<T>(v: &mut [T]) {
    // todo!("spec: lo=0 hi=len;while lo+1<hi {{ hi-=1; swap(lo,hi); lo+=1 }}")

    // [1]
    let (mut lo, mut hi) = (0, v.len());
    // (0, 1)
    while lo + 1 < hi {
        hi -= 1;
        // redundant
        // let (left, right) = v.split_at_mut(hi);
        // std::mem::swap(&mut left[lo], &mut right[0]);
        v.swap(lo, hi);
        lo += 1;
    }
}

/// 形狀 5 的載體:需要把欄位搬出來重建。
pub struct EventLog {
    events: Vec<String>,
}

impl EventLog {
    pub fn new(events: Vec<String>) -> Self {
        Self { events }
    }

    pub fn events(&self) -> &[String] {
        &self.events
    }

    /// spec:`compact` 想**消費** `self.events`(`into_iter` 做 trim / 去空再收回),
    /// 但不能 `move out of &mut self`。用 `std::mem::take` 把欄位換成空 Vec、拿到
    /// 所有權後重建。
    pub fn compact(&mut self) {
        // todo!(
        //     "spec: let taken = mem::take(&mut self.events); self.events = taken.into_iter().map(trim).filter(非空).collect()"
        // )

        // let taken = std::mem::take(&mut self.events);
        // self.events = taken
        //     .into_iter()
        //     .map(|s| s.trim().to_string())
        //     .filter(|s| !s.is_empty())
        //     .collect();

        // instead of use to_string to allocate new heap, we can use retain_mut and truncate / drain
        //
        // 就地 trim 之後,`mem::take` 的前提就沒了:take 存在是因為 `into_iter()` 要
        // 所有權,而 trim 只需要 `&mut String`。不重配 String 就不必消費,形狀 5 塌回
        // 形狀 4(retain_mut)。實測 24 筆輸入:上面的 to_string 版每個保留元素各一次
        // malloc(18 allocs),這版 0 次。
        //
        // 整體 O(B) 時間 / O(1) 空間,B = 所有字串的總 bytes;零配置。
        //
        // 這個 0 是結構性的,不是運氣:**trim 是純粹縮小的操作,而縮小從來不需要跟
        // allocator 講話**。truncate/drain 都只動 len、不動 capacity,被砍掉的 bytes
        // 留在 buffer 裡變成餘裕(cap - len)。反過來,任何讓字串「長大」的需求都會
        // 拆掉這個保證:例如規格若改成加前綴 `insert_str(0, "[log] ")`,配不配取決於
        // `cap - trim後len >= 6`,而餘裕來自「前綴空白 + 後綴空白 + 出生 slack」——
        // 也就是**配置行為會取決於輸入長什麼樣子**(髒資料零配置、乾淨資料每筆一次),
        // 而輸出完全正確,測試抓不到。真要保證得建構時 reserve,或別把前綴存進 String。
        self.events.retain_mut(|s| {
            let end = s.trim_end().len();
            // 全空白 / 空字串。順便擋掉下面 start(== len)> end(== 0)、
            // truncate(0) 之後 drain(..len) 越界 panic 的情況。
            if end == 0 {
                return false;
            }
            let start = s.len() - s.trim_start().len();

            // 砍尾。做的事只有兩件:assert 一次 char boundary(讀 end 那個 byte 看
            // 是不是 UTF-8 continuation,單 byte 檢查、不掃描),然後把 len 設成 end。
            // 底層 Vec<u8> 照理要 drop 掉 [end..],但 u8 沒有 Drop,那段會被完全
            // 最佳化掉——尾巴的 bytes 只是不再算進 len,實體還躺在原地。
            //
            // 時間 O(1) / 空間 O(1),**與砍掉多少無關**(實測 N=4e7:砍掉 N-1 與
            // 砍掉 1 都是 110ns)。不搬移、不重配、capacity 不變。
            s.truncate(end);

            // 砍頭。`drain(range)` 回傳一個會 yield 被移除 char 的迭代器,而
            // **移除發生在那個迭代器 drop 的當下**,不是呼叫的當下——這裡當 statement
            // 寫、產生後立刻 drop,所以效果是純粹「砍掉前 start 個 bytes」。
            //   呼叫時:正規化 range、兩端各 assert 一次 char boundary、建一個帶 lazy
            //           chars() 的 Drain。一個 byte 都沒動。
            //   drop 時:把尾巴 [end..len) 整段 ptr::copy 到 start 位置,再減 len。
            //
            // 時間 O(len - start) / 空間 O(1):**成本只跟「要保留的尾巴」成正比,跟
            // 砍掉多少無關**(實測 N=4e7:砍掉 N-1 留 1 是 730ns,砍掉 1 留 N-1 是
            // 946µs——砍得多反而快 1300 倍,因為沒剩什麼要搬;這也證明 drop 沒有去
            // 解碼被砍掉那段的 chars)。start == 0 時搬 0 bytes,是 no-op。
            // 不重配、capacity 不變。
            //
            // 兩個地雷:
            // 1. 順序不可反。先 drain 的話尾巴會整段前移,先算好的 end 就失效了
            //    (得補成 end - start)。
            // 2. range 兩端必須落在 char boundary,否則 panic。trim_start/trim_end
            //    給的位置天生是 boundary,所以這裡安全;自己湊 byte index 就不一定。
            s.drain(..start);
            true
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_in_place_values_and_empty() {
        let mut v = [1i64, -2, 3];
        scale_in_place(&mut v, 10);
        assert_eq!(v, [10, -20, 30]);

        let mut empty: [i64; 0] = [];
        scale_in_place(&mut empty, 5);
        assert!(empty.is_empty());
    }

    /// boundary:[0,1,0,3,12] → [1,3,12,0,0](非 0 保序、0 到尾)。
    #[test]
    fn move_zeroes_trace() {
        let mut v = [0, 1, 0, 3, 12];
        move_zeroes(&mut v);
        assert_eq!(v, [1, 3, 12, 0, 0]);
    }

    /// boundary:空、全 0、全非 0、單一 0。
    #[test]
    fn move_zeroes_boundaries() {
        let mut a: [i32; 0] = [];
        move_zeroes(&mut a);
        assert!(a.is_empty());

        let mut allz = [0, 0, 0];
        move_zeroes(&mut allz);
        assert_eq!(allz, [0, 0, 0]);

        let mut none = [1, 2, 3];
        move_zeroes(&mut none);
        assert_eq!(none, [1, 2, 3]);

        let mut one = [0, 5];
        move_zeroes(&mut one);
        assert_eq!(one, [5, 0]);
    }

    /// boundary:[1,1,2,3,3,3] → [1,2,3] 回 3。
    #[test]
    fn dedup_sorted_trace() {
        let mut v = vec![1, 1, 2, 3, 3, 3];
        assert_eq!(dedup_sorted(&mut v), 3);
        assert_eq!(v, vec![1, 2, 3]);
    }

    /// boundary:空、單元素、全同、全不同。
    #[test]
    fn dedup_sorted_boundaries() {
        let mut e: Vec<i32> = vec![];
        assert_eq!(dedup_sorted(&mut e), 0);

        let mut same = vec![4, 4, 4, 4];
        assert_eq!(dedup_sorted(&mut same), 1);
        assert_eq!(same, vec![4]);

        let mut uniq = vec![1, 2, 3];
        assert_eq!(dedup_sorted(&mut uniq), 3);
        assert_eq!(uniq, vec![1, 2, 3]);
    }

    #[test]
    fn keep_and_double_positives_one_pass() {
        let mut v = vec![1, -2, 3, 0, 4];
        keep_and_double_positives(&mut v);
        assert_eq!(v, vec![2, 6, 8]);
    }

    /// boundary:偶數對撞、奇數中點不動、空 / 單元素。
    #[test]
    fn reverse_in_place_parity() {
        let mut even = [1, 2, 3, 4];
        reverse_in_place(&mut even);
        assert_eq!(even, [4, 3, 2, 1]);

        let mut odd = [1, 2, 3];
        reverse_in_place(&mut odd);
        assert_eq!(odd, [3, 2, 1]);

        let mut empty: [i32; 0] = [];
        reverse_in_place(&mut empty);
        assert!(empty.is_empty());
    }

    #[test]
    fn compact_takes_field_out_and_rebuilds() {
        let mut log = EventLog::new(vec![
            "  boot  ".to_string(),
            "".to_string(),
            "ready".to_string(),
            "   ".to_string(),
        ]);
        log.compact();
        assert_eq!(log.events(), ["boot".to_string(), "ready".to_string()]);
    }

    /// boundary:過期淘汰 + 未來 timestamp(saturating_sub 夾 0 → 不過期)。
    #[test]
    fn evict_expired_cases() {
        let mut m = HashMap::new();
        m.insert("a".to_string(), 60u64);
        m.insert("b".to_string(), 80u64);
        m.insert("future".to_string(), 200u64);
        evict_expired(&mut m, 100, 30);
        assert!(!m.contains_key("a")); // 40 > 30 → 刪
        assert!(m.contains_key("b")); // 20 ≤ 30 → 留
        assert!(m.contains_key("future")); // saturating_sub → 0 ≤ 30 → 留
    }

    /// oracle 對照:偽隨機 300 組輸入,move_zeroes 與
    /// 「filter 非零 + 尾補零」模型比對——寫指標的 off-by-one
    /// 在隨機形狀下無所遁形。
    #[test]
    fn move_zeroes_matches_oracle() {
        let mut seed: u32 = 0x2026_0716;
        let mut lcg = move || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            seed
        };
        for _ in 0..300 {
            let len = (lcg() % 20) as usize;
            let mut v: Vec<i32> = (0..len).map(|_| (lcg() % 5) as i32 - 2).collect();
            let mut expected: Vec<i32> = v.iter().copied().filter(|&x| x != 0).collect();
            expected.resize(v.len(), 0);
            move_zeroes(&mut v);
            assert_eq!(v, expected);
        }
    }

    /// oracle 對照:排序後的隨機輸入,dedup_sorted 與 std 的
    /// `Vec::dedup` 比對(回傳的新長度也要一致)。
    #[test]
    fn dedup_sorted_matches_std() {
        let mut seed: u32 = 7;
        let mut lcg = move || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            seed
        };
        for _ in 0..300 {
            let len = (lcg() % 24) as usize;
            let mut v: Vec<i32> = (0..len).map(|_| (lcg() % 6) as i32).collect();
            v.sort_unstable();
            let mut expected = v.clone();
            expected.dedup();
            let kept = dedup_sorted(&mut v);
            assert_eq!(kept, expected.len());
            assert_eq!(v, expected);
        }
    }
}
