//! 教學範例:最小 intrusive 等待鏈(stack-pinned self-referential 節點)。
//!
//! waiter 節點活在「呼叫 `wait()` 的那條 thread」的 stack 上,用 `pin!` 釘住
//! (`!Unpin` 靠 `PhantomPinned`),把「指向自己」的裸指標掛進共享 list;喚醒者
//! 持鎖把它摘掉並 `unpark`。這是 `tokio::sync::Notify` / 各種 waiter queue 底層
//! pattern 的縮影,也是「future 被 stack-pin 不只為 future 自己,也讓內部節點能被
//! 外界安全指向」的具體實例。
//!
//! 跑:`cargo run -p reference --example intrusive_wait_queue`
//!
//! ## 關鍵不變式(為什麼 sound)
//! 一個節點,只在它的 `wait()` frame 正在 park(還活著)期間才留在 list 裡;
//! `notify` 在喚醒之前先 unlink,而 `wait()` 只有在被 unlink 之後才返回。
//! 所以 queue 永遠不會握著指向「已死 frame」的指標。
//!
//! ## 教學用、非 production —— 刻意省掉:
//! - **cancellation / timeout**(最硬的一塊):若 `wait()` 能提早返回(逾時、或
//!   async 下 future 被 drop),節點可能還掛在 list 上 frame 就死了 → 懸空;真實版
//!   要一個 Drop guard,drop 時鎖住 queue 把自己 unlink。這版靠「`wait()` 只在
//!   `notify` unlink 後才返回」繞過。
//! - **Waker**:這裡用 sync 的 `park`/`unpark`;async 版節點裡存 `Waker`,`poll`
//!   註冊後回 `Pending`,`notify` 呼 `waker.wake()`——同一個「節點住在被 pin 的
//!   future 裡」pattern。
//! - **lock-free**(單一 `Mutex`)、**FIFO**(這裡頭插頭出 = LIFO)。
//! - **aliasing / provenance 未過 Miri 驗證**。
//!
//! production 級請讀 `tokio::sync::Notify` 與 `intrusive-collections` crate。

use std::marker::PhantomPinned;
use std::pin::pin;
use std::sync::{Arc, Mutex};
use std::thread::{self, Thread};

struct Waiter {
    next: *mut Waiter,   // intrusive link(null = 尾端);排隊時由 queue 維護
    thread: Thread,      // 要喚醒誰
    notified: bool,      // 被喚醒了嗎
    _pin: PhantomPinned, // !Unpin:一旦掛進 list 就不能移動
}

pub struct WaitQueue {
    head: Mutex<*mut Waiter>, // 一條 intrusive 單向鏈,指標指進各 waiter 的 stack
}

// SAFETY: head 內的裸指標指向各 thread 的 stack 節點,對節點的所有存取一律持
// `head` 鎖 → 互斥;節點只在其 `wait()` frame 存活(park 中)期間留在鏈上。
// 跨執行緒共享安全由「持鎖存取 + 上述不變式」保證。
unsafe impl Send for WaitQueue {}
unsafe impl Sync for WaitQueue {}

impl WaitQueue {
    pub fn new() -> Self {
        WaitQueue {
            head: Mutex::new(std::ptr::null_mut()),
        }
    }

    /// 阻塞直到被 `notify_one` 喚醒。節點在本 frame 的 stack 上 —— 零 heap alloc。
    pub fn wait(&self) {
        // 1. stack 上建節點,pin! 釘住(保證這 frame 內它不會 move)
        let mut waiter = pin!(Waiter {
            next: std::ptr::null_mut(),
            thread: thread::current(),
            notified: false,
            _pin: PhantomPinned,
        });
        // SAFETY: 取 pin 後節點的 *mut;直到它被 unlink 前,frame 不結束、節點不 move,
        // 且對它的所有存取都在 queue 的鎖底下,不與別人同時碰。
        let node: *mut Waiter = unsafe { waiter.as_mut().get_unchecked_mut() as *mut Waiter };

        // 2. 把「指向自己 stack 的指標」推進共享 list 頭
        {
            let mut head = self.head.lock().unwrap();
            // SAFETY: 持鎖 → 對 node 與鏈結構互斥;node 目前只有本執行緒在寫。
            unsafe {
                (*node).next = *head;
            }
            *head = node;
        }

        // 3. park 直到喚醒者把 notified 設 true(且已把我從 list 摘掉)
        loop {
            {
                let _g = self.head.lock().unwrap(); // 持鎖才讀 notified,擋與 notify 的競態
                // SAFETY: 持鎖 → 對 node 互斥。
                if unsafe { (*node).notified } {
                    break;
                }
            }
            thread::park();
        }
        // 走到這:notify_one 已把我 unlink,node 不再被任何指標指著,frame 可安全結束。
    }

    /// 喚醒鏈上一個等待者(頭出)。
    pub fn notify_one(&self) {
        let mut head = self.head.lock().unwrap();
        let node = *head;
        if !node.is_null() {
            // SAFETY: 持鎖 → 對 node 互斥;node 指向仍在 `wait()` park 的某 thread 的
            // stack(那個 frame 還沒返回,所以還活著)。
            unsafe {
                *head = (*node).next; // 從 list 摘掉(必在對方 frame 死之前)
                (*node).notified = true;
                (*node).thread.unpark();
            }
        }
    }
}

impl Default for WaitQueue {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let q = Arc::new(WaitQueue::new());

    let mut handles = Vec::new();
    for id in 0..3 {
        let q = q.clone();
        handles.push(thread::spawn(move || {
            println!("waiter {id}: 掛上等待鏈(節點在我自己的 stack)");
            q.wait();
            println!("waiter {id}: 被喚醒,frame 結束");
        }));
    }

    // 給 waiters 一點時間把節點掛上
    thread::sleep(std::time::Duration::from_millis(80));
    for _ in 0..3 {
        q.notify_one();
        thread::sleep(std::time::Duration::from_millis(20));
    }

    for h in handles {
        h.join().unwrap();
    }
    println!("main: 全部喚醒完畢,所有 waiter 的 stack 節點都已安全回收");
}
