//! drill:trie —— 填 insert(建路徑)與 walk(沿路徑)。
//!
//! 已給:arena 結構、idx、contains/starts_with/remove(全建立在 walk 上)。
//! 要填:`insert` / `walk`。
//! 關鍵之辨:contains 看 is_end,starts_with 只看路徑存在。

struct Node {
    children: [Option<usize>; 26],
    is_end: bool,
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
    nodes: Vec<Node>, // nodes[0] = root
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

    fn idx(c: char) -> usize {
        assert!(c.is_ascii_lowercase(), "trie only supports a-z");
        (c as u8 - b'a') as usize
    }

    /// spec:沿 word 的每個字元走;child 缺就在 arena 尾端 push 新節點
    /// 並接上(新索引 = push 前的 nodes.len())。走完把終點 is_end = true。
    /// O(L)。注意:空字串也合法(root 自己標 is_end)。
    pub fn insert(&mut self, word: &str) {
        // todo!("spec: 沿路徑走,缺就配;終點 is_end = true")
        let mut cur = 0;
        for ch in word.chars() {
            let idx = Self::idx(ch);
            if self.nodes[cur].children[idx].is_none() {
                let node_idx = self.nodes.len();
                self.nodes.push(Node::new());
                self.nodes[cur].children[idx] = Some(node_idx);
            }

            cur = self.nodes[cur].children[idx].expect("Invariant: we just insert this.");
        }
        self.nodes[cur].is_end = true;
    }

    /// spec:沿 s 走到底,回終點節點索引;任一步斷掉回 None。O(L)。
    /// 提示:`self.nodes[cur].children[Self::idx(c)]?`(? 對 Option 直接用)。
    fn walk(&self, s: &str) -> Option<usize> {
        // todo!("spec: 從 root(0) 沿字元走,斷掉 None")
        let mut cur = 0;
        for ch in s.chars() {
            let idx = Self::idx(ch);
            cur = self.nodes[cur].children[idx]?
        }
        Some(cur)
    }

    pub fn contains(&self, word: &str) -> bool {
        self.walk(word).is_some_and(|n| self.nodes[n].is_end)
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        self.walk(prefix).is_some()
    }

    pub fn remove(&mut self, word: &str) -> bool {
        match self.walk(word) {
            Some(n) if self.nodes[n].is_end => {
                self.nodes[n].is_end = false;
                true
            }
            _ => false,
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// boundary:重疊前綴共享節點;詞 vs 前綴之辨。
    #[test]
    // #[ignore = "填完 insert/walk 後移除"]
    fn overlap_and_word_vs_prefix() {
        let mut t = Trie::new();
        t.insert("app");
        assert_eq!(t.node_count(), 4);
        t.insert("apple");
        assert_eq!(t.node_count(), 6); // 只多 l、e 兩個節點
        assert!(t.contains("app"));
        assert!(!t.contains("appl")); // 路過的前綴不是詞
        assert!(t.starts_with("appl"));
        assert!(!t.starts_with("b"));
    }

    /// boundary:空字串與單字元。
    #[test]
    // #[ignore = "填完 insert/walk 後移除"]
    fn empty_string_and_single_char() {
        let mut t = Trie::new();
        assert!(!t.contains(""));
        assert!(t.starts_with("")); // root 永遠存在
        t.insert("");
        assert!(t.contains(""));
        t.insert("a");
        assert!(t.contains("a"));
    }
}
