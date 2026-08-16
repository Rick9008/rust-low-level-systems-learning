fn take_ownership(numbers: Vec<i32>) {
    println!("函式取得所有權：{numbers:?}");
}

fn read_only(numbers: &[i32]) {
    println!("不可變借用，元素數量：{}", numbers.len());
}

fn append_number(numbers: &mut Vec<i32>, value: i32) {
    numbers.push(value);
}

fn main() {
    let moved_numbers = vec![1, 2, 3];
    take_ownership(moved_numbers);
    // 取消下一行的註解會產生編譯錯誤，因為所有權已被移走。
    // println!("{moved_numbers:?}");

    let mut borrowed_numbers = vec![10, 20];

    read_only(&borrowed_numbers);
    println!("唯讀借用結束後仍可使用：{borrowed_numbers:?}");

    append_number(&mut borrowed_numbers, 30);
    println!("可變借用修改了原資料：{borrowed_numbers:?}");

    let first_reader = &borrowed_numbers;
    let second_reader = &borrowed_numbers;
    println!("兩個唯讀引用可以並存：{first_reader:?} / {second_reader:?}");

    // 如果後面仍會使用 first_reader，就不能在這裡同時建立可變引用。
    // let writer = &mut borrowed_numbers;
    // println!("{first_reader:?} / {writer:?}");
}
