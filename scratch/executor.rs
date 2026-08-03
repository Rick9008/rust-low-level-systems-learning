use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread;

struct ThreadWaker(thread::Thread);

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

fn block_on<F: Future>(fut: F) -> F::Output {
    // todo!()
    // 考點:Arc → Waker 那一步怎麼轉、Context 怎麼建、
    // future 怎麼 pin、poll 迴圈的兩臂(Ready / Pending)各做什麼
    let mut pin_fut = std::pin::pin!(fut);
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));

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

fn main() {
    assert_eq!(block_on(async { 42 }), 42);
}
