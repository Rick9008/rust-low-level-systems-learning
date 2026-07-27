// 7/27 骨架默寫抽查(15m,taper 豁免項)——白紙規則:不翻任何檔案,寫完才對答案
// 七格,每格默完往下一格,卡住標 ??? 繼續走,不深挖:
//
// ══ 批改(14:55,1✓/5⚠/1✗)══════════════════════════════════════════════
// ✗ 5 length-prefix【真洞】usize::from_be_bytes → try_into 推 [u8;8],4-byte slice
//    每次 runtime Err。正解 u32::from_be_bytes(...) as usize(wire 型別解,再 as host)。
//    → 7/28 早暖手重默這 3 行。
// ⚠ 1 spsc:write/read_slot 少 &self(7/19 老傷疤回鍋);stray use VecDeque。
// ⚠ 2 pool:睡眠條件語意 ✓;退出條件沒寫;closure 多分號回 ()、少 .unwrap()。
// ⚠ 3 framer:退回 c#1 形狀——c#2 定版 = feed(&mut self)->Vec<Frame> + parse(&mut self)->Option<Frame>。
// ⚠ 4 TCP:多套 loop、read 少 &mut、write_all 少 &、忘 use std::io::{Read,Write};accept Err = continue 不是 break。
// ✓ 6 token:零洞,mask 傷疤癒合(as u32 截斷天生足 32 bit)。
// ⚠ 7 channel:Receiver 三步 ✓;Sender 正解 = if fetch_sub(1, Release) == 1(用回傳值,非 sub 後再 load)。
// ═══════════════════════════════════════════════════════════════════════
//
// ── 1. spsc:use 塊 + struct/impl<T> 簽名(含 UnsafeCell/&self)──────────────
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
unsafe impl<T: Send> Sync for SpscRing<T> {}
unsafe impl<T: Send> Send for SpscRing<T> {}

impl<T> SpscRing<T> {
    fn new(cap: usize) -> Self {
        todo!()
    }
    fn write_slot(&self, idx: usize, item: T) {
        todo!()
    }
    fn read_slot(&self, idx: usize) -> T {
        todo!()
    }
}

struct Producer<T> {
    ring: Arc<SpscRing<T>>,
}
struct Consumer<T> {
    ring: Arc<SpscRing<T>>,
}

// ── 2. thread pool:worker 兩條件(退出條件 + 睡眠條件,正面寫法)──────────

fn worker_loop {
    let mut jobs_guard = self.shared.jobs.lock().unwrap();
    if jobs_guard.queue.is_empty() && self.shared.shutdown.load(Ordering::Acquire) {
        break;
    }
    jobs_guard = wait_job.wait_while(jobs_guard, |s| {
        s.queue.is_empty() && !self.shared.shutdown.load(Ordering::Acquire)
    }).unwrap();
}


// ── 3. framer:parse 函式簽名(輸入/回傳/Err 合約)──────────────────────────


impl framer {
    fn parse(&mut self) -> Option<Vec<Frame>> {
        todo!()
    }
    fn feed(&mut self, buf: &[u8]) -> Vec<Frame> {
        todo!()
    }
}


// ── 4. TCP accept-loop 六行(std)───────────────────────────────────────────
use std::net::TcpListener;
use std::io::{Read, Write};

fn serve(listener: TcpListener) {
        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(stream) => stream,
                Err(_err) => continue,
            };
            std::thread::spawn(move || {
                loop {
                    let mut buf = [0;4096];
                    match stream.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if stream.write_all(&buf[0..n]).is_err() {
                                break;
                            }
                        },
                        Err(_) => break,
                    }
                }
            });
        }
}


// ── 5. length-prefix 解析 3 行(checked_add → get → from_be_bytes + try_into)─

fn length_prefix(buf: &[u8], ptr: usize) -> Option<()> {
    let end = ptr.checked_add(4)?;
    let slice = buf.get(ptr..end)?;
    let len = u32::from_be_bytes(slice.try_into()?) as usize;
}


// ── 6. token pack/unpack(mask 足 32 bit)───────────────────────────────────

fn pack(generation: u32, fd: u32) -> u64 {
    ((generation as u64) << 32) | (fd as u64)
}

fn unpack(token: u64) -> (u32, u32) {
    (((token >> 32) as u32), token as u32)
}


// ── 7. bounded_channel 雙 Drop 六行(store/sub → 拿鎖放鎖 → notify_all)─────


impl<T> Drop for Consumer<T> {
    fn drop(&mut self) {
    self.has_consumer.store(false, Ordering::Release);
    let _guard = self.deque.lock().unwrap();
    self.wait_not_full.notify_all();
    }
}

impl<T> Drop for Producer<T> {
    fn drop(&mut self) {
    if self.producer_cnt.fetch_sub(1, Ordering::Release) > 1 {
        return;
    }
    let _gaurd = self.deque.lock().unwrap();
    self.wait_job.notify_one();
    }
}




