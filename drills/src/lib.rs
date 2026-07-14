//! # drills —— 填空層
//!
//! 與 `reference` 相同的模組樹,但核心函式的函式體被挖空成
//! `todo!("spec: ...")`。骨架、helper、資料結構定義都給了,
//! 只填會被面試考的那幾行核心邏輯。
//!
//! 使用法:
//! 1. `cargo test -p drills -- --include-ignored` 看哪些測試紅。
//! 2. 打開對應檔案,讀函式上方的 spec doc comment。
//! 3. 先在紙上 dry-run 測試裡標注的 boundary(空、滿、wrap、overflow)。
//! 4. 填掉 `todo!()`,移除測試上的 `#[ignore]`,轉綠。
//! 5. 卡住就 diff `reference/` 的同名模組(最後手段)。

// 挖空的函式參數/為你準備的 import 在你填完之前用不到;
// 骨架 helper 也可能暫時沒人呼叫。這些 allow 讓 drills 在「全部挖空」
// 狀態下依然過 -D warnings 閘門。填完所有 todo 後可拿掉自查。
#![allow(unused_variables, dead_code, unused_imports)]

pub mod bounded_queue;
pub mod dsu;
pub mod graph;
pub mod hw_bridge;
pub mod lru;
pub mod ring_buffer;
pub mod sharded_map;
pub mod spsc_ring;
pub mod thread_pool;
pub mod tree;
pub mod trie;

pub mod arena_lockfree;
pub mod epoll_sys;
pub mod event_loop;
pub mod executor;
pub mod file_io_offload;
pub mod tcp_echo;
