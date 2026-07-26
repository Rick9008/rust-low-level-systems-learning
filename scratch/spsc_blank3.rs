// 12:11
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[repr(align(64))]
struct CachePadding<T>(T);

struct SpscRing<T> {
    buf: Box<[UnsafeCell<MaybeUninit<T>>]>,
    cap: usize,
    mask: usize,
    head: CachePadding<AtomicUsize>,
    tail: CachePadding<AtomicUsize>,
}

impl<T> SpscRing<T> {
    pub fn new(cap: usize) -> Self {
        assert!(cap >= 1);
        let cap = cap.next_power_of_two();
        Self {
            buf: (0..cap)
                .map(|_| UnsafeCell::new(MaybeUninit::uninit()))
                .collect(),
            cap,
            mask: cap - 1,
            head: CachePadding(AtomicUsize::new(0)),
            tail: CachePadding(AtomicUsize::new(0)),
        }
    }

    pub fn write_slot(&self, idx: usize, item: T) {
        unsafe {
            (*self.buf[idx & self.mask].get()).write(item);
        }
    }

    pub fn read_slot(&self, idx: usize) -> T {
        unsafe { (*self.buf[idx & self.mask].get()).assume_init_read() }
    }
}

impl<T> Drop for SpscRing<T> {
    fn drop(&mut self) {
        let mut tail = self.tail.0.load(Ordering::Relaxed);
        let head = self.head.0.load(Ordering::Relaxed);
        while tail != head {
            unsafe { (*self.buf[tail & self.mask].get()).assume_init_drop() }
            tail = tail.wrapping_add(1);
        }
    }
}

unsafe impl<T: Send> Sync for SpscRing<T> {}
unsafe impl<T: Send> Send for SpscRing<T> {}

struct Producer<T> {
    ring: Arc<SpscRing<T>>,
}

impl<T> Producer<T> {
    pub fn push(&self, item: T) -> Result<(), T> {
        let head = self.ring.head.0.load(Ordering::Relaxed);
        let tail = self.ring.tail.0.load(Ordering::Acquire);
        if head.wrapping_sub(tail) == self.ring.cap {
            return Err(item);
        }
        self.ring.write_slot(head, item);
        self.ring
            .head
            .0
            .store(head.wrapping_add(1), Ordering::Release);
        Ok(())
    }
}

struct Consumer<T> {
    ring: Arc<SpscRing<T>>,
}

impl<T> Consumer<T> {
    pub fn pop(&self) -> Option<T> {
        let tail = self.ring.tail.0.load(Ordering::Relaxed);
        let head = self.ring.head.0.load(Ordering::Acquire);
        if head == tail {
            return None;
        }
        let item = self.ring.read_slot(tail);
        self.ring
            .tail
            .0
            .store(tail.wrapping_add(1), Ordering::Release);
        Some(item)
    }
}

pub fn channel<T>(cap: usize) -> (Producer<T>, Consumer<T>) {
    let ring_arc = Arc::new(SpscRing::new(cap));
    (
        Producer {
            ring: ring_arc.clone(),
        },
        Consumer { ring: ring_arc },
    )
}

// 12:27
// 12:36

#[test]
fn smoke() {
    let (mut p, mut c) = channel(2);
    assert!(p.push(3).is_ok());
    assert!(p.push(2).is_ok());
    assert_eq!(p.push(60), Err(60));
    assert_eq!(c.pop(), Some(3));
    assert!(p.push(6).is_ok());
    assert_eq!(c.pop(), Some(2));
    assert_eq!(c.pop(), Some(6));
    assert_eq!(c.pop(), None);
}
