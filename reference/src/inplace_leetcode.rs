//! # inplace_leetcode —— 高頻 in-place 題示範(接 [`crate::iter_mutate`])
//!
//! ## [Clarify]
//! 解決:把 [`crate::iter_mutate`] 的「邊迭代邊修改」pattern 落到五道最常出現的
//! LeetCode in-place 題,示範面試現場怎麼把「O(1) space、就地改」講清楚並寫對。
//! Constraints:std-only;每題都要求 **O(1) 額外空間**(不能開新陣列)——這正是
//! 「邊走邊改」被逼出來的場景。
//!
//! ## [Abstract]
//! 五題三種骨架:寫指標(27 / 80)、多指標分割(75)、反向填避免覆蓋(88)、
//! 反轉三部曲(189)。
//!
//! ## [Iterate]
//! - **27 Remove Element**:寫指標最單純版——`read` 掃、`write` 收留下的。
//! - **75 Sort Colors**:Dutch national flag,`lo / hi / i` 三指標一趟三分割;
//!   從尾端換過來的元素**還沒看過,`i` 不能前進**(最容易錯的一格)。
//! - **80 Remove Duplicates II**:寫指標 + 回看 `v[write-2]`,每個值最多留 2 個。
//! - **88 Merge Sorted Array**:**從尾端往前填**。從前面填會蓋掉 `a` 還沒讀的段;
//!   反向填時「寫入位置永遠在讀取位置的右邊」,天然不覆蓋。
//! - **189 Rotate Array**:反轉三部曲(全反 → 反前 k → 反後 n−k),O(1) 空間旋轉,
//!   直接複用 [`crate::iter_mutate::reverse_in_place`]。
//!
//! ## [Trade-offs]
//! - 這些題「開一個新 Vec 收結果」都能過,但面試問的就是 O(1) space;寫指標 /
//!   反向填是把空間壓到 O(1) 的標準手法。
//! - 88 的方向、80 的 `write-2`、75 的「換過來不動 `i`」是三個經典覆蓋 / off-by-one
//!   坑,而且全繞不開 **usize underflow**(`write-2`、`i-1`)——邊界 guard 要先想
//!   (見 [`crate::iter_mutate`] 的 [Trade-offs])。
//!
//! ## [Dry-Run]
//! 見測試:每題 trace 一個代表性輸入 + 空 / 單元素 / 全同邊界;75 / 88 另有 proptest
//! 對照 sort 版 oracle。

use crate::iter_mutate::reverse_in_place;

/// LeetCode 27 — Remove Element。移除所有等於 `val` 的元素,回傳新長度,
/// 前 `k` 個即結果(順序保留)。寫指標:`read` 掃、`write` 收非 `val`。O(n) / O(1)。
pub fn remove_element(v: &mut Vec<i32>, val: i32) -> usize {
    let mut write = 0usize;
    let mut read = 0usize;
    while read < v.len() {
        if v[read] != val {
            v[write] = v[read];
            write += 1;
        }
        read += 1;
    }
    v.truncate(write);
    write
}

/// LeetCode 75 — Sort Colors(Dutch National Flag)。0 / 1 / 2 三色就地排序,一趟。
/// 三指標:`lo`=下一個 0 的落點、`hi`=下一個 2 的落點(exclusive)、`i`=游標。
/// O(n) / O(1)。
///
/// 關鍵坑:碰到 2 時 `swap(i, hi)` 換過來的元素**還沒檢查**,所以 `i` 不能前進;
/// 碰到 0 時 `swap(lo, i)` 換過來的必是已掃過的 1(或 `i==lo`),所以 `i` 可以前進。
pub fn sort_colors(v: &mut [u8]) {
    let mut lo = 0usize;
    let mut hi = v.len();
    let mut i = 0usize;
    while i < hi {
        match v[i] {
            0 => {
                v.swap(lo, i);
                lo += 1;
                i += 1;
            }
            2 => {
                hi -= 1;
                v.swap(i, hi); // 換來的還沒看,i 不動
            }
            _ => i += 1, // 1 就地
        }
    }
}

/// LeetCode 80 — Remove Duplicates from Sorted Array II。已排序陣列,每個值最多
/// 保留 2 個,回傳新長度。O(n) / O(1)。
///
/// 寫指標 + 回看:新元素只要 `!= v[write-2]` 就能放——因為已排序,`v[write-2]`
/// 是「目前這個值已保留的第一個」,不同就代表該值還沒滿 2 個。`write < 2` 時
/// 前兩格無條件收(也順手避開 `write-2` 的 usize underflow)。
pub fn dedup_at_most_two(v: &mut Vec<i32>) -> usize {
    let mut write = 0usize;
    let mut read = 0usize;
    while read < v.len() {
        if write < 2 || v[read] != v[write - 2] {
            v[write] = v[read];
            write += 1;
        }
        read += 1;
    }
    v.truncate(write);
    write
}

/// LeetCode 88 — Merge Sorted Array。`a` 的前 `m` 個有效、後 `n` 個是佔位;`b` 有
/// `n` 個。就地合併成排序,結果留在 `a`。O(m+n) / O(1)。
///
/// 關鍵:**從尾端往前填**。若從前面填,會蓋掉 `a` 還沒讀的有效段;反向填時寫入位置
/// `w` 永遠在兩個讀游標的右邊,天然不覆蓋。所有減法都先 guard,避開 usize underflow。
pub fn merge_sorted(a: &mut [i32], m: usize, b: &[i32]) {
    let n = b.len();
    assert_eq!(a.len(), m + n, "a 的長度必須是 m + n(後 n 個為佔位)");
    let mut i = m; // a 有效段游標(往前)
    let mut j = n; // b 游標(往前)
    let mut w = m + n; // 寫入位置(往前)
    while j > 0 {
        w -= 1;
        // a 還有貨、且它更大 → 放 a;否則(含 a 已空)放 b
        if i > 0 && a[i - 1] > b[j - 1] {
            i -= 1;
            a[w] = a[i];
        } else {
            j -= 1;
            a[w] = b[j];
        }
    }
    // b 空了就結束:a 剩下的有效段本來就在正確位置,不用動。
}

/// LeetCode 189 — Rotate Array。向右旋轉 `k` 步,in place、O(1) 空間。
/// 反轉三部曲:全反 → 反前 `k` → 反後 `n−k`。O(n)。複用
/// [`crate::iter_mutate::reverse_in_place`]。
pub fn rotate_right(v: &mut [i32], k: usize) {
    let n = v.len();
    if n == 0 {
        return;
    }
    let k = k % n; // k 可能 ≥ n
    reverse_in_place(v);
    reverse_in_place(&mut v[..k]);
    reverse_in_place(&mut v[k..]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// 27:trace [3,2,2,3] 移除 3 → [2,2] 回 2;無命中 / 全命中邊界。
    #[test]
    fn remove_element_cases() {
        let mut v = vec![3, 2, 2, 3];
        assert_eq!(remove_element(&mut v, 3), 2);
        assert_eq!(v, vec![2, 2]);

        let mut none = vec![1, 2, 3];
        assert_eq!(remove_element(&mut none, 9), 3);
        assert_eq!(none, vec![1, 2, 3]);

        let mut all = vec![7, 7, 7];
        assert_eq!(remove_element(&mut all, 7), 0);
        assert!(all.is_empty());
    }

    /// [Dry-Run] 75 sort_colors trace(v=[2,0,2,1,1,0])lo/hi/i:
    ///   i=0 v=2 → hi=5 swap(0,5)→[0,0,2,1,1,2]        i=0 (不動)
    ///   i=0 v=0 → swap(0,0) lo=1 i=1
    ///   i=1 v=0 → swap(1,1) lo=2 i=2
    ///   i=2 v=2 → hi=4 swap(2,4)→[0,0,1,1,2,2]        i=2 (不動)
    ///   i=2 v=1 → i=3     i=3 v=1 → i=4     i=4==hi 停
    ///   結果 [0,0,1,1,2,2]。
    #[test]
    fn sort_colors_trace() {
        let mut v = [2u8, 0, 2, 1, 1, 0];
        sort_colors(&mut v);
        assert_eq!(v, [0, 0, 1, 1, 2, 2]);
    }

    #[test]
    fn sort_colors_boundaries() {
        let mut empty: [u8; 0] = [];
        sort_colors(&mut empty);
        assert!(empty.is_empty());

        let mut one = [1u8];
        sort_colors(&mut one);
        assert_eq!(one, [1]);

        let mut allsame = [2u8, 2, 2];
        sort_colors(&mut allsame);
        assert_eq!(allsame, [2, 2, 2]);
    }

    /// [Dry-Run] 80 dedup_at_most_two(v=[1,1,1,2,2,3])→[1,1,2,2,3],回 5:
    ///   read 0/1:write<2 無條件收 → [1,1..] write=2
    ///   read 2 v=1 == v[0]=1 → 跳過(已 2 個)      write=2
    ///   read 3 v=2 != v[0]=1 → 收 v[2]=2           write=3
    ///   read 4 v=2 != v[1]=1 → 收 v[3]=2           write=4
    ///   read 5 v=3 != v[2]=2 → 收 v[4]=3           write=5
    #[test]
    fn dedup_at_most_two_trace() {
        let mut v = vec![1, 1, 1, 2, 2, 3];
        assert_eq!(dedup_at_most_two(&mut v), 5);
        assert_eq!(v, vec![1, 1, 2, 2, 3]);
    }

    #[test]
    fn dedup_at_most_two_boundaries() {
        let mut e: Vec<i32> = vec![];
        assert_eq!(dedup_at_most_two(&mut e), 0);

        let mut one = vec![5];
        assert_eq!(dedup_at_most_two(&mut one), 1);

        let mut two = vec![5, 5];
        assert_eq!(dedup_at_most_two(&mut two), 2);
        assert_eq!(two, vec![5, 5]);

        let mut many = vec![5, 5, 5, 5];
        assert_eq!(dedup_at_most_two(&mut many), 2);
        assert_eq!(many, vec![5, 5]);
    }

    /// [Dry-Run] 88 merge_sorted(a=[1,2,3,_,_,_], m=3, b=[2,5,6])反向填:
    ///   w=5: a[2]=3 vs b[2]=6 → 6 大,放 b → a[5]=6  j=2
    ///   w=4: a[2]=3 vs b[1]=5 → 5 大,放 b → a[4]=5  j=1
    ///   w=3: a[2]=3 vs b[0]=2 → 3 大,放 a → a[3]=3  i=2
    ///   w=2: a[1]=2 vs b[0]=2 → 不 >,放 b → a[2]=2  j=0 停
    ///   a 剩 [1,2] 已在位 → [1,2,2,3,5,6]。
    #[test]
    fn merge_sorted_trace() {
        let mut a = [1, 2, 3, 0, 0, 0];
        merge_sorted(&mut a, 3, &[2, 5, 6]);
        assert_eq!(a, [1, 2, 2, 3, 5, 6]);
    }

    #[test]
    fn merge_sorted_boundaries() {
        let mut only_b = [0, 0, 0];
        merge_sorted(&mut only_b, 0, &[1, 2, 3]);
        assert_eq!(only_b, [1, 2, 3]);

        let mut only_a = [1, 2, 3];
        merge_sorted(&mut only_a, 3, &[]);
        assert_eq!(only_a, [1, 2, 3]);

        let mut empty: [i32; 0] = [];
        merge_sorted(&mut empty, 0, &[]);
        assert!(empty.is_empty());
    }

    /// [Dry-Run] 189 rotate_right([1,2,3,4,5,6,7], 3):
    ///   全反   → [7,6,5,4,3,2,1]
    ///   反 [..3] → [5,6,7,4,3,2,1]
    ///   反 [3..] → [5,6,7,1,2,3,4]
    #[test]
    fn rotate_right_trace() {
        let mut v = [1, 2, 3, 4, 5, 6, 7];
        rotate_right(&mut v, 3);
        assert_eq!(v, [5, 6, 7, 1, 2, 3, 4]);
    }

    #[test]
    fn rotate_right_boundaries() {
        let mut v = [1, 2, 3];
        rotate_right(&mut v, 0); // k=0 → 不變
        assert_eq!(v, [1, 2, 3]);
        rotate_right(&mut v, 3); // k==n → 不變
        assert_eq!(v, [1, 2, 3]);
        rotate_right(&mut v, 4); // k>n → k%n=1
        assert_eq!(v, [3, 1, 2]);

        let mut empty: [i32; 0] = [];
        rotate_right(&mut empty, 2);
        assert!(empty.is_empty());
    }

    proptest! {
        /// 75:sort_colors 的結果必須 == 直接 sort。
        #[test]
        fn prop_sort_colors_matches_sort(v in proptest::collection::vec(0u8..3, 0..40)) {
            let mut expected = v.clone();
            expected.sort_unstable();
            let mut got = v.clone();
            sort_colors(&mut got);
            prop_assert_eq!(got, expected);
        }

        /// 88:merge_sorted 的結果 == 兩段合起來 sort。
        #[test]
        fn prop_merge_sorted_matches_oracle(
            mut aa in proptest::collection::vec(-5i32..5, 0..20),
            mut bb in proptest::collection::vec(-5i32..5, 0..20),
        ) {
            aa.sort_unstable();
            bb.sort_unstable();
            let m = aa.len();
            let mut expected: Vec<i32> = aa.iter().chain(bb.iter()).copied().collect();
            expected.sort_unstable();
            let mut a = aa.clone();
            a.resize(m + bb.len(), 0); // 佔位
            merge_sorted(&mut a, m, &bb);
            prop_assert_eq!(a, expected);
        }
    }
}
