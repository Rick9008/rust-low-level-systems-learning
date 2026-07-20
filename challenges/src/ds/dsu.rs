//! challenge:union-find(DSU)
//!
//! 【題目】維護 `0..n` 元素的不相交集合:union 合併、find 查代表、
//! connected 判同集合、components 回集合總數。
//!
//! 【constraints】
//! - 一億元素、一億次操作要能跑:find/union 攤銷時間必須接近常數
//!   (提示考點:兩個標準優化,你要能說出名字與各自擋住什麼退化)
//! - find **禁止遞迴**(想想 10⁶ 長鏈的 call stack)
//! - components 查詢 O(1)
//!
//! 【clarify points——動手前先自答】
//! - 不做任何優化時,什麼操作序列讓 find 退化成 O(n)?
//! - 兩個優化各自的不變量是什麼?疊加後的攤銷界叫什麼?
//!
//! 【要實作】下方簽名。【驗收】tests/dsu.rs 轉綠。

pub struct Dsu {
    // ↓ 佔位:動手時整個換成你的設計。
    _todo: (),
}

impl Dsu {
    pub fn new(n: usize) -> Self {
        todo!("challenge: 從空白開始")
    }

    /// 回傳 x 所在集合的代表。攤銷近常數。
    pub fn find(&mut self, x: usize) -> usize {
        todo!("challenge")
    }

    /// 合併;回傳是否真的發生合併(原本就同集合 → false)。
    pub fn union(&mut self, x: usize, y: usize) -> bool {
        todo!("challenge")
    }

    pub fn connected(&mut self, x: usize, y: usize) -> bool {
        todo!("challenge")
    }

    /// O(1)。
    pub fn components(&self) -> usize {
        todo!("challenge")
    }
}
