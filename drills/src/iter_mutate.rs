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

        let taken = std::mem::take(&mut self.events);
        self.events = taken
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
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
