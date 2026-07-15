//! # iter_mutate —— 邊迭代邊修改(std / slice 慣用法)
//!
//! ## [Clarify]
//! 解決:面試高頻的「我想一邊走過 Vec / slice,一邊改它」在 Rust 為什麼常常
//! 編不過,以及六種對應形狀怎麼選。核心規則一句話:**借用檢查器不准你在持有
//! 一個(共享或迭代)借用的同時,再拿一個會使前者失效的可變借用。**
//! `for x in &v { v.push(..) }` 編不過——`push` 可能 realloc,讓 `x` 指向的
//! 記憶體失效。這不是龜毛:它把 C++ 的 iterator invalidation(UB)提到編譯期。
//! Constraints:std-only、單執行緒;只談「同一個容器邊走邊改」,跨容器不在此。
//!
//! ## [Abstract]
//! 動手前先問兩題:(1)你改的是**值**、還是**結構(長度 / 容量)**?
//! (2)你需不需要**同時**碰到多個元素?兩個答案就決定用哪個工具。
//!
//! ## [Iterate]
//! 六種形狀,由最常見排到最進階:
//! 1. 只改值、長度不變 → [`iter_mut`]:`for x in v.iter_mut() { *x = ... }`
//!    (見 [`scale_in_place`])
//! 2. 改結構(移除 / 搬移)、單向掃描 → **寫指標 two-pointer**:`read` 掃描、
//!    `write` 標下一個該落的位置,in place、O(1) 空間
//!    (見 [`move_zeroes`] / [`dedup_sorted`])
//! 3. 篩選 + 同時改值、只走一趟 → [`Vec::retain_mut`](給 `&mut T`,回 `bool`
//!    決定去留;見 [`keep_and_double_positives`])
//! 4. 邊遍歷邊刪(Vec / HashMap / HashSet) → `retain`;更複雜條件的 fallback 是
//!    **先 collect 要動的 index / key,再第二趟動它**,繞開借用衝突
//!    (見 [`evict_expired`])
//! 5. 需要「消費 / 拿所有權」卻卡在 `&mut self` 後面 → [`std::mem::take`] /
//!    [`std::mem::replace`] 把欄位換成 Default 值再重建(見 [`EventLog::compact`])
//! 6. 需要**同時**拿兩個 `&mut` 到同一 slice 的不同位置 → [`slice::split_at_mut`] /
//!    `split_first_mut`:借用檢查器無法證明 `&mut v[i]` 與 `&mut v[j]` 不重疊,
//!    這些方法在內部用 unsafe 把 slice 切成兩段互不重疊的可變 slice 還你
//!    (`slice::swap` 底層就是這樣;見 [`reverse_in_place`])
//!
//! ## [Trade-offs]
//! - `iter_mut()` vs 索引迴圈 `for i in 0..len`:能用 `iter_mut` 就用——不必碰
//!   index、天生無越界,也躲開 **usize underflow**(`i - 1` 在 `i == 0` 時,
//!   debug build panic、release build 靜默 wrap 成 `usize::MAX`)。只有「要看
//!   鄰居 `v[i±1]`」或「要 `v[i]` 與 `v[j]`」時才退回索引 / `split_at_mut`。
//! - 寫指標 in place(O(1) 空間) vs `filter().collect()` 到新 Vec(O(n) 空間、
//!   但程式更短):面試若強調 in place / O(1) space,寫指標;否則 collect 更清楚。
//! - `retain` 是 O(n) 單趟;「在 `for i in 0..len` 裡 `remove(i)`」則是 O(n²) 且
//!   會 index 錯位(刪一個後面全左移)——別這樣做。
//!
//! ## [Dry-Run]
//! 見測試:[`move_zeroes`] 逐格 trace read / write 指標、[`dedup_sorted`] 相鄰
//! 重複與空 / 全同邊界、[`keep_and_double_positives`] 篩選+改值一趟、
//! [`EventLog::compact`] 取走後原欄位為空、[`evict_expired`] HashMap 過期淘汰,
//! 以及 proptest 對照 clone 版 oracle。

use std::collections::HashMap;

/// 形狀 1:只改值、長度不變。`iter_mut()` 是「邊迭代邊改值」最直接的形狀——
/// 拿到的是 `&mut T`,不碰 index。O(n)。
pub fn scale_in_place(v: &mut [i64], factor: i64) {
    for x in v.iter_mut() {
        *x *= factor;
    }
}

/// 形狀 2a:把所有 0 搬到尾端,非 0 的相對順序不變,in place。O(n) 時間、
/// O(1) 空間。寫指標法:`read` 掃描,`write` 是「下一個非 0 該落的位置」;
/// 遇到非 0 就 `swap(write, read)` 再推進 `write`。
///
/// 用 `while` 而非 `for read in 0..len`:後者除了會被 clippy 盯上,更重要的是
/// 提醒你這是**雙指標各自推進**,不是單純走訪。
pub fn move_zeroes(v: &mut [i32]) {
    let mut write = 0usize;
    let mut read = 0usize;
    while read < v.len() {
        if v[read] != 0 {
            v.swap(write, read);
            write += 1;
        }
        read += 1;
    }
}

/// 形狀 2b:已排序 Vec 就地去重,回傳去重後長度並 `truncate`。O(n) / O(1)。
/// 對照 [`Vec::dedup`](std 就有這功能);這裡手寫是為了看清寫指標邏輯:
/// `write - 1` 永遠指向「最後一個已保留的值」,和 `read` 不同才寫入。
pub fn dedup_sorted(v: &mut Vec<i32>) -> usize {
    if v.is_empty() {
        return 0;
    }
    let mut write = 1usize; // v[0] 一定保留
    let mut read = 1usize;
    while read < v.len() {
        if v[read] != v[write - 1] {
            v[write] = v[read];
            write += 1;
        }
        read += 1;
    }
    v.truncate(write);
    write
}

/// 形狀 3:一趟內同時「篩選 + 改值」——留下正數,並把留下的翻倍。
/// [`Vec::retain_mut`](Rust 1.61+)給你 `&mut T`、回 `bool` 決定去留,
/// 比「先 `retain` 再 `iter_mut`」少走一趟。O(n)。
pub fn keep_and_double_positives(v: &mut Vec<i32>) {
    v.retain_mut(|x| {
        let keep = *x > 0;
        if keep {
            *x *= 2;
        }
        keep
    });
}

/// 形狀 4:邊遍歷邊刪。刪掉所有「已過期」的條目(`now - inserted > ttl`)。
/// [`HashMap::retain`] 是「不能在 `for` 迴圈裡 `remove`」的正解——單趟 O(n)。
/// 用 `saturating_sub` 而非 `-`:`inserted` 若大於 `now`(時鐘回跳 / 測資亂序),
/// 裸減會 usize/u64 underflow;`saturating_sub` 夾到 0,語意上「還沒過期」。
///
/// Fallback(舊 Rust,或條件複雜到 retain closure 塞不下):先把要刪的 key
/// `collect` 成 `Vec`,再第二趟 `for k in keys { map.remove(&k); }`——這樣刪除
/// 就不再與遍歷借用衝突。
pub fn evict_expired(map: &mut HashMap<String, u64>, now: u64, ttl: u64) {
    map.retain(|_key, inserted| now.saturating_sub(*inserted) <= ttl);
}

/// 形狀 6:需要**同時**兩個 `&mut`。就地反轉,兩指標對撞。
/// `slice::swap(i, j)` 底層就是 `split_at_mut` 的封裝——借用檢查器不讓你直接
/// 同時持有 `&mut v[i]` 與 `&mut v[j]`(無法證明不重疊),`swap` / `split_at_mut`
/// 用 unsafe 在內部保證兩段不重疊,把安全介面還你。O(n)。
pub fn reverse_in_place<T>(v: &mut [T]) {
    let mut lo = 0usize;
    let mut hi = v.len();
    while lo + 1 < hi {
        hi -= 1;
        v.swap(lo, hi);
        lo += 1;
    }
}

/// 形狀 5 的載體:一個「需要把欄位搬出來重建」的典型場景。
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

    /// 形狀 5:`compact` 想 **消費** `self.events`(`into_iter` 做 trim / filter
    /// 再收集回去),但你不能 `move out of &mut self`。[`std::mem::take`] 用
    /// Default 值(空 `Vec`)換走原值,拿到所有權後安心重建。O(n)。
    ///
    /// 替代法:`self.events.drain(..)` 也能拿到元素所有權;`mem::take` 勝在
    /// 「整個欄位換出去」語意最清楚,且中途 `self` 一直處於合法(空)狀態。
    pub fn compact(&mut self) {
        let taken = std::mem::take(&mut self.events); // self.events 現在是空 Vec
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
    use proptest::prelude::*;

    /// boundary:空 / 單元素不 panic;負數與長度不變。
    #[test]
    fn scale_in_place_values_and_empty() {
        let mut v = [1i64, -2, 3];
        scale_in_place(&mut v, 10);
        assert_eq!(v, [10, -20, 30]);

        let mut empty: [i64; 0] = [];
        scale_in_place(&mut empty, 5);
        assert!(empty.is_empty());
    }

    /// [Dry-Run] move_zeroes 逐格 trace(v = [0,1,0,3,12]):
    ///   read=0 v[0]=0  跳過                     write=0
    ///   read=1 v[1]=1≠0  swap(0,1)→[1,0,0,3,12]  write=1
    ///   read=2 v[2]=0  跳過                     write=1
    ///   read=3 v[3]=3≠0  swap(1,3)→[1,3,0,0,12]  write=2
    ///   read=4 v[4]=12≠0 swap(2,4)→[1,3,12,0,0]  write=3
    ///   結果 [1,3,12,0,0]:非 0 保持相對順序,0 全到尾端。
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

    /// [Dry-Run] dedup_sorted trace(v = [1,1,2,3,3,3]):
    ///   write=1(v[0] 一定留)
    ///   read=1 v[1]=1 == v[0]  跳過          write=1
    ///   read=2 v[2]=2 ≠  v[1]  v[1]=2        write=2
    ///   read=3 v[3]=3 ≠  v[1]  v[2]=3        write=3
    ///   read=4 v[4]=3 == v[2]  跳過          write=3
    ///   read=5 v[5]=3 == v[2]  跳過          write=3
    ///   truncate(3) → [1,2,3],回 3。
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

        let mut one = vec![7];
        assert_eq!(dedup_sorted(&mut one), 1);
        assert_eq!(one, vec![7]);

        let mut same = vec![4, 4, 4, 4];
        assert_eq!(dedup_sorted(&mut same), 1);
        assert_eq!(same, vec![4]);

        let mut uniq = vec![1, 2, 3];
        assert_eq!(dedup_sorted(&mut uniq), 3);
        assert_eq!(uniq, vec![1, 2, 3]);
    }

    /// retain_mut:一趟丟掉 ≤0、留下的翻倍。
    #[test]
    fn keep_and_double_positives_one_pass() {
        let mut v = vec![1, -2, 3, 0, 4];
        keep_and_double_positives(&mut v);
        assert_eq!(v, vec![2, 6, 8]);
    }

    /// reverse_in_place:偶數長度對撞、奇數長度中點不動、空 / 單元素安全。
    #[test]
    fn reverse_in_place_parity() {
        let mut even = [1, 2, 3, 4];
        reverse_in_place(&mut even);
        assert_eq!(even, [4, 3, 2, 1]);

        let mut odd = [1, 2, 3];
        reverse_in_place(&mut odd);
        assert_eq!(odd, [3, 2, 1]);

        let mut one = [9];
        reverse_in_place(&mut one);
        assert_eq!(one, [9]);

        let mut empty: [i32; 0] = [];
        reverse_in_place(&mut empty);
        assert!(empty.is_empty());
    }

    /// mem::take:compact 把欄位搬出、trim + 去空、重建。
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

    /// [Dry-Run] evict_expired(now=100, ttl=30):保留 `100 - inserted ≤ 30`,
    /// 即 inserted ≥ 70。
    ///   "a":60  → 100-60=40 > 30 → 刪
    ///   "b":80  → 100-80=20 ≤ 30 → 留
    ///   "c":100 → 100-100=0 ≤ 30 → 留
    #[test]
    fn evict_expired_drops_stale() {
        let mut m = HashMap::new();
        m.insert("a".to_string(), 60u64);
        m.insert("b".to_string(), 80u64);
        m.insert("c".to_string(), 100u64);
        evict_expired(&mut m, 100, 30);
        assert_eq!(m.len(), 2);
        assert!(!m.contains_key("a"));
        assert!(m.contains_key("b"));
        assert!(m.contains_key("c"));
    }

    /// clock skew:inserted 在未來(> now),saturating_sub 夾到 0 → 視為未過期。
    #[test]
    fn evict_expired_future_timestamp_is_not_stale() {
        let mut m = HashMap::new();
        m.insert("future".to_string(), 200u64);
        evict_expired(&mut m, 100, 30);
        assert!(m.contains_key("future"));
    }

    proptest! {
        /// property:move_zeroes 後,非 0 元素 == 原序列的非 0 子序列(順序保留),
        /// 後面補滿 0。oracle:filter 非 0 再 resize 補 0。
        #[test]
        fn prop_move_zeroes_matches_oracle(v in proptest::collection::vec(-5i32..5, 0..50)) {
            let mut expected: Vec<i32> = v.iter().copied().filter(|&x| x != 0).collect();
            expected.resize(v.len(), 0); // 補回原本的 0 數量
            let mut got = v.clone();
            move_zeroes(&mut got);
            prop_assert_eq!(got, expected);
        }

        /// property:dedup_sorted 對已排序輸入,結果與 std 的 `Vec::dedup` 一致。
        #[test]
        fn prop_dedup_sorted_matches_std(mut v in proptest::collection::vec(0i32..8, 0..50)) {
            v.sort_unstable();
            let mut expected = v.clone();
            expected.dedup();
            let mut got = v.clone();
            let n = dedup_sorted(&mut got);
            prop_assert_eq!(n, expected.len());
            prop_assert_eq!(got, expected);
        }
    }
}
