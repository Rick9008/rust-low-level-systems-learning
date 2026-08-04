// ═══ Warmup 8/4 — 題 3(建議 2m)═══
//
// Given `buf: &[u8]` and `ptr: usize`, return the 4-byte big-endian
// length field starting at `ptr` as `Option<usize>`. Any input —
// `ptr` past the end, near `usize::MAX`, buffer too short — must
// return `None`, never panic.

fn read_len(buf: &[u8], ptr: usize) -> Option<usize> {
    let end = ptr.checked_add(4)?;
    let slice = buf.get(ptr..end)?;
    Some(u32::from_be_bytes(slice.try_into().ok()?) as usize)
}

// ═══ Warmup 8/4 — 題 4(建議 1m)═══
//
// You're about to implement a lock-free SPSC ring on CoderPad (std only):
// fixed-capacity slot array writable through shared references, slots
// start uninitialized, producer and consumer are two handles owned by
// two threads, indices are atomic counters. Write the `use` block this
// design needs.

// (題 4 的 use 塊直接寫在這行下面)

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// ═══ 批改(8/4;Claude)═══
//
// 題 3:**7/27 真洞(wire 型別 u32 解、再 as usize)未復發,結案。** 1 錯:
// ✗ slice.try_into()?        // ? 在回 Option 的 fn 裡不能拆 Result(E0277)
// ✓ slice.try_into().ok()?   // Result 先 .ok() 降級再 ?
//
// 題 4:0 錯 PASS。四行全中——「use 塊衰退」只在 tokio 側,std 側肌肉完好。
