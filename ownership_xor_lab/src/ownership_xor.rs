//! 第 0 關：所有權與借用暖身。
//!
//! 一次只完成一個函式，再執行 `cargo test ownership_xor`。

/// 取得 Vec 的所有權並計算總和。
///
/// 提示：可以使用迴圈，也可以使用 iterator 的 `sum`。
pub fn consume_and_sum(numbers: Vec<i32>) -> i32 {
    //todo!("走訪 numbers 並回傳總和")
    let mut total = 0;
    for number in numbers {
        total += number;
    }
    total
}

/// 借用 Vec 並加入一個元素，不取得 Vec 的所有權。
pub fn append_number(numbers: &mut Vec<i32>, value: i32) {
    //todo!("透過 mutable borrow 加入 value")
    numbers.push(value);
}

/// 借用 slice，回傳其中最大值的引用；空 slice 回傳 None。
///
/// 注意：回傳的是 `&i32`，不是複製一份 `i32`。
pub fn largest(numbers: &[i32]) -> Option<&i32> {
    //todo!("走訪 numbers 並保留最大值的引用")
    if numbers.is_empty(){
        return None;
    }
    let mut max_index = &numbers[0];
    for number in &numbers[1..]{
        if *max_index < *number{
            max_index = number;
        }
    }
    Some(max_index)
}

/// 同時修改 slice 中兩個不同的位置。
///
/// 提示：`slice.swap(a, b)` 已經替你封裝好不重疊的 mutable borrow。
pub fn swap_positions(numbers: &mut [i32], a: usize, b: usize) {
    //todo!("交換索引 a 與 b 的值")
    numbers.swap(a,b);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ownership_xor_consume_and_sum() {
        let numbers = vec![2, 4, 6];
        assert_eq!(consume_and_sum(numbers), 12);
        // numbers 的所有權已移入函式，因此這裡不能再使用 numbers。
    }

    #[test]
    fn ownership_xor_append_through_mutable_borrow() {
        let mut numbers = vec![1, 2];
        append_number(&mut numbers, 3);
        assert_eq!(numbers, vec![1, 2, 3]);
    }

    #[test]
    fn ownership_xor_largest_returns_a_borrow() {
        let numbers = vec![8, 3, 13, 5];
        assert_eq!(largest(&numbers), Some(&13));
        assert_eq!(largest(&[]), None);
    }

    #[test]
    fn ownership_xor_swap_two_positions() {
        let mut numbers = vec![10, 20, 30];
        swap_positions(&mut numbers, 0, 2);
        assert_eq!(numbers, vec![30, 20, 10]);
    }
}

