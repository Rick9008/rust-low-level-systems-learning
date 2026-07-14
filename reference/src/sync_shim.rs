//! # sync_shim —— std / loom 的替換層
//!
//! library 本體 std-only,但 lock-free 程式碼要給 loom 窮舉驗證,
//! loom 要求被測程式用它的 atomic / UnsafeCell 型別(它靠這些型別攔截
//! 每一次記憶體存取來排程 interleaving)。
//!
//! 解法:核心演算法(`spsc_ring/core_impl.rs`、`arena_lockfree/core_impl.rs`)
//! 一律 `use crate::sync_shim as sync`。
//! - lib 編譯時:本檔案生效,全部是 std 型別 → **production 路徑零依賴**。
//! - loom 測試(`tests/loom_*.rs`)用 `#[path]` include 同一份演算法原始碼,
//!   並在測試 crate root 定義同名 `sync_shim`(re-export loom 型別)。
//!
//! 同一份演算法、兩套記憶體模型實例化——loom 驗過的就是 lib 跑的那份邏輯。

pub(crate) use std::sync::Arc;
pub(crate) use std::sync::atomic;

/// 模仿 `loom::cell::UnsafeCell` 的閉包式 API。
/// loom 版的 with/with_mut 會登記讀/寫存取供資料競爭偵測;
/// std 版就是裸的指標存取,零開銷(閉包必然內聯)。
#[derive(Debug)]
pub(crate) struct UnsafeCell<T>(std::cell::UnsafeCell<T>);

impl<T> UnsafeCell<T> {
    pub(crate) fn new(v: T) -> Self {
        Self(std::cell::UnsafeCell::new(v))
    }

    /// 唯讀存取:呼叫端保證此刻無人在寫(不變量寫在呼叫處的 SAFETY 註解)。
    pub(crate) fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
        f(self.0.get())
    }

    /// 可寫存取:呼叫端保證此刻獨佔(不變量寫在呼叫處的 SAFETY 註解)。
    pub(crate) fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
        f(self.0.get())
    }
}
