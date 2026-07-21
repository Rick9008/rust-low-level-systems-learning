//! # trie —— arena 上的 prefix tree
//!
//! ## [Clarify]
//! 解決:字串集合的 insert / 精確查詢 / 前綴查詢,時間都是 O(key 長度),
//! 與集合大小無關——autocomplete、路由表、字典的底層。
//! Constraints:字母表固定 `a-z`(26 個小寫;其他字元 panic,聲明在前);
//! key 長度 10⁰–10²;節點數上限 Σ|key|。
//!
//! ## [Abstract]
//! 節點不存 payload(純集合語意)。要 map 語意(key → value)就把
//! `is_end: bool` 換成 `value: Option<V>`——結構不變,面試先做集合往前走。
//!
//! ## [Iterate]
//! 指標式(`Box<Node>` 子節點)→ arena 式(本實作):節點全放一個 `Vec`,
//! child 存索引。好處:單次配置攤平、cache locality、無遞迴 Drop
//! (指標式深 trie drop 時遞迴析構可能爆 stack)。
//!
//! ## [Trade-offs]
//! - children 用 `[Option<usize>; 26]`(**424 bytes/node**,不是直覺的 208——
//!   `usize` 沒有 niche,所以 `size_of::<Option<usize>>() == 16` 而非 8;
//!   換 `Option<NonZeroUsize>` 是 216B,換 `u32` + sentinel 是 108B。
//!   見 docs/ds/trie.md):child 查找 O(1) 零 hash;
//!   代價是稀疏節點浪費——字母表大(Unicode)或極稀疏時換 `HashMap<char, usize>`
//!   (O(1) 期望 + heap 開銷)或排序 `Vec<(char, usize)>`(O(log deg) 二分)。
//! - arena 只長不縮:刪除詞只清 `is_end`,不回收節點(懶刪除)。
//!   回收要 free list + 世代標記——那是 [`crate::concurrency::arena_lockfree`] 的主題。
//! - insert/contains/starts_with 都是 O(L) 時間,L = key 長;空間 O(Σ L × 424B) 最壞。
//!
//! ## [Dry-Run]
//! 見測試:重疊前綴逐步 trace、空字串、單字 vs 前綴之辨、刪除保前綴詞。

struct Node {
    children: [Option<usize>; 26],
    is_end: bool, // 有詞在此結束(區分「路過的前綴」與「完整的詞」)
}

impl Node {
    fn new() -> Self {
        Self {
            children: [None; 26],
            is_end: false,
        }
    }
}

pub struct Trie {
    nodes: Vec<Node>, // nodes[0] = root(空字串的位置)
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}

impl Trie {
    pub fn new() -> Self {
        Self {
            nodes: vec![Node::new()],
        }
    }

    /// 字元 → 0..26 索引。字母表約束在單一入口把關。
    fn idx(c: char) -> usize {
        assert!(c.is_ascii_lowercase(), "trie only supports a-z, got {c:?}");
        (c as u8 - b'a') as usize
    }

    /// O(L)。沿路徑走,缺的節點現配(arena push,索引 = 新長度)。
    pub fn insert(&mut self, word: &str) {
        let mut cur = 0;
        for c in word.chars() {
            let i = Self::idx(c);
            cur = match self.nodes[cur].children[i] {
                Some(next) => next,
                None => {
                    let next = self.nodes.len();
                    self.nodes.push(Node::new());
                    self.nodes[cur].children[i] = Some(next);
                    next
                }
            };
        }
        self.nodes[cur].is_end = true;
    }

    /// 走到 prefix 末端的節點索引;任一步斷掉回 None。O(L)。
    fn walk(&self, s: &str) -> Option<usize> {
        let mut cur = 0;
        for c in s.chars() {
            cur = self.nodes[cur].children[Self::idx(c)]?;
        }
        Some(cur)
    }

    /// 精確查詢:路徑存在 **且** 末端標記為詞。O(L)。
    pub fn contains(&self, word: &str) -> bool {
        self.walk(word).is_some_and(|n| self.nodes[n].is_end)
    }

    /// 前綴查詢:路徑存在即可(不看 is_end)。O(L)。
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.walk(prefix).is_some()
    }

    /// 懶刪除:只清 is_end,節點留在 arena(見 Trade-offs)。O(L)。
    /// 回傳是否真的刪了一個詞。
    pub fn remove(&mut self, word: &str) -> bool {
        match self.walk(word) {
            Some(n) if self.nodes[n].is_end => {
                self.nodes[n].is_end = false;
                true
            }
            _ => false,
        }
    }

    /// arena 節點數(教學觀測用:共享前綴不重複配置)。
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 把 doc 裡的 424B 釘死在測試裡。
    ///
    /// 這個數字曾經寫錯成 208B——直覺是 26 × 8 + 1,但 `usize` 沒有 niche,
    /// 所以 `Option<usize>` 要多一個 word 放 discriminant(16B,不是 8B)。
    /// 26 × 16 + is_end + padding = 424B。
    ///
    /// 一併釘住 `Option<usize>` 的大小:哪天它變了(或有人把 children 換成
    /// `NonZeroUsize`),這裡會紅,doc 就不會繼續說謊。
    #[test]
    fn node_size_matches_the_documented_number() {
        assert_eq!(
            size_of::<Option<usize>>(),
            16,
            "usize 無 niche,Option 必須另配 discriminant word"
        );
        assert_eq!(size_of::<Node>(), 424, "docs/ds/trie.md 宣稱的每節點空間");
    }

    /// [Dry-Run] 重疊前綴 trace:
    ///   insert("app"): root→a(1)→p(2)→p(3),is_end[3]=true;node_count=4
    ///   insert("apple"): a,p,p 已在,只新增 l(4)、e(5);node_count=6 ← 共享前綴
    ///   contains("app")=true(3 是詞尾) contains("appl")=false(4 只是路過)
    ///   starts_with("appl")=true(路徑存在)
    #[test]
    fn overlapping_words_share_prefix_nodes() {
        let mut t = Trie::new();
        t.insert("app");
        assert_eq!(t.node_count(), 4);
        t.insert("apple");
        assert_eq!(t.node_count(), 6); // 只多 2 個節點
        assert!(t.contains("app"));
        assert!(t.contains("apple"));
        assert!(!t.contains("appl")); // boundary:路徑存在但非詞
        assert!(t.starts_with("appl"));
        assert!(!t.starts_with("apx"));
    }

    /// boundary:空字串是合法詞(root 的 is_end)。
    #[test]
    fn boundary_empty_string_word() {
        let mut t = Trie::new();
        assert!(!t.contains(""));
        assert!(t.starts_with("")); // 空前綴永遠成立(root 存在)
        t.insert("");
        assert!(t.contains(""));
    }

    /// boundary:查詢空 trie、單字元詞。
    #[test]
    fn boundary_empty_trie_and_single_char() {
        let mut t = Trie::new();
        assert!(!t.contains("a"));
        assert!(!t.starts_with("a"));
        t.insert("a");
        assert!(t.contains("a"));
        assert!(t.starts_with("a"));
    }

    /// remove 是懶刪除:清詞不清前綴詞、節點留在 arena。
    /// trace:insert app, apple → remove("apple") → app 仍在、appl 前綴仍可走。
    #[test]
    fn remove_keeps_prefix_words_intact() {
        let mut t = Trie::new();
        t.insert("app");
        t.insert("apple");
        assert!(t.remove("apple"));
        assert!(!t.contains("apple"));
        assert!(t.contains("app")); // 前綴詞不受影響
        assert!(t.starts_with("appl")); // 節點未回收(懶刪除)
        assert!(!t.remove("apple")); // 重複刪 → false
        assert!(!t.remove("ap")); // 非詞(只是前綴)→ false
    }
}
