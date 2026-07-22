/*
▎ Implement a bounded single-producer single-consumer (SPSC) queue.
▎ try_push returns the item back when full; try_pop returns None when empty. Non-blocking, no locks.
▎ The producer and consumer handles must be usable from two different threads.
*/

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[repr(align(64))]
struct CachePadding<T>(T);

struct SpscRing<T> {
    buf: Box<[UnsafeCell<MaybeUninit<T>>]>,
    head: CachePadding<AtomicUsize>,
    tail: CachePadding<AtomicUsize>,
    cap: usize,
    mask: usize,
}

impl<T> SpscRing<T> {
    fn new(cap: usize) -> Self {
        assert!(cap >= 1);
        let cap = cap.next_power_of_two();
        Self {
            buf: (0..cap)
                .map(|_| UnsafeCell::new(MaybeUninit::uninit()))
                .collect(),
            head: CachePadding(AtomicUsize::new(0)),
            tail: CachePadding(AtomicUsize::new(0)),
            cap,
            mask: cap - 1,
        }
    }
    fn write_slot(&self, idx: usize, item: T) {
        unsafe {
            (*self.buf[idx & self.mask].get()).write(item);
        }
    }

    fn read_slot(&self, idx: usize) -> T {
        unsafe { (*self.buf[idx & self.mask].get()).assume_init_read() }
    }
}

unsafe impl<T: Send> Send for SpscRing<T> {}
unsafe impl<T: Send> Sync for SpscRing<T> {}

impl<T> Drop for SpscRing<T> {
    fn drop(&mut self) {
        let tail = self.tail.0.load(Ordering::Relaxed);
        let mut head = self.head.0.load(Ordering::Relaxed);
        while head != tail {
            unsafe {
                (*self.buf[head].get()).assume_init_drop();
            }
            head = head.wrapping_add(1);
        }
    }
}

struct Producer<T> {
    ring: Arc<SpscRing<T>>,
}

struct Consumer<T> {
    ring: Arc<SpscRing<T>>,
}

impl<T> Producer<T> {
    pub fn push(&mut self, item: T) -> Result<(), T> {
        let tail = self.ring.tail.0.load(Ordering::Relaxed);
        let head = self.ring.head.0.load(Ordering::Acquire);

        if tail.wrapping_sub(head) == self.ring.cap {
            return Err(item);
        }

        self.ring.write_slot(tail, item);
        self.ring
            .tail
            .0
            .store(tail.wrapping_add(1), Ordering::Release);
        Ok(())
    }
}

impl<T> Consumer<T> {
    pub fn pop(&mut self) -> Option<T> {
        let head = self.ring.head.0.load(Ordering::Relaxed);
        let tail = self.ring.tail.0.load(Ordering::Acquire);

        if head == tail {
            return None;
        }

        let item = self.ring.read_slot(head);
        self.ring
            .head
            .0
            .store(head.wrapping_add(1), Ordering::Release);
        Some(item)
    }
}

pub fn Channel<T>(cap: usize) -> (Producer<T>, Consumer<T>) {
    let spsc = Arc::new(SpscRing::<T>::new(cap));
    (Producer { ring: spsc.clone() }, Consumer { ring: spsc })
}

#[test]
fn smoke_single_thread() {
    let (mut p, mut c) = Channel::<i32>(2);
    assert_eq!(p.push(2), Ok(()));
    assert_eq!(p.push(3), Ok(()));
    assert_eq!(p.push(1), Err(1));
    assert_eq!(c.pop(), Some(2));
    assert_eq!(c.pop(), Some(3));
    assert_eq!(c.pop(), None);
}
