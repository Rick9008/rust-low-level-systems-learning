// ═══ Warmup 8/4 — 題 2(建議 6m)═══
//
// Write `fn block_on<F: Future>(fut: F) -> F::Output` in pure std:
// runs the future to completion on the current thread. No busy-spinning,
// no channels, no condvar — use the cheapest thread-blocking primitive
// std offers. It must work for a future that gets woken from another
// thread.
//
// (簽名照 spec 立好;imports 與其餘全部自己來。)

use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

struct ThreadWaker(Thread);

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

fn block_on<F: Future>(fut: F) -> F::Output {
    // todo!()
    let mut pin_fut = std::pin::pin!(fut);
    let waker = Waker::from(Arc::new(ThreadWaker(std::thread::current())));

    let mut context = Context::from_waker(&waker);
    loop {
        match pin_fut.as_mut().poll(&mut context) {
            Poll::Ready(output) => break output,
            Poll::Pending => {
                thread::park();
            }
        }
    }
}

// ═══ 批改(8/4 冷重默;Claude)═══
//
// 合約層全對:pin 同顆 future(昨錯⑤概念洞)✓、park/unpark 詞組(昨錯②)✓、
// poll(&mut cx) 呼叫形(昨錯④)✓——概念洞 0。
//
// 6 個編譯錯全是「誰住哪個模組」一族(名字洞,rustc 一輪可回收):
// ✗ use std::future::{Future, Poll};
// ✗ use std::thread::{self, Thread, Waker};
// ✓ use std::sync::Arc;
// ✓ use std::task::{Context, Poll, Wake, Waker};  // 記法:poll 世界全家住 std::task
// ✓ use std::thread::{self, Thread};              // thread 只有 Thread/park
//   (Future 在 edition 2024 prelude,可不 import——pad 實測 2024)
//
// ⚠ Wake trait 名 + Waker::from(Arc) 本場 assisted(Claude 中途劇透)→ 8/5 taper 重驗
// ⚠ 修綠後補 3 行 smoke 跑一次(昨晚工作流洞:給了 rustc 沒跑)
