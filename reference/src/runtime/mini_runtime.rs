//! # mini_runtime —— executor × reactor 縫起來:兩階 reactor 的 mini-tokio
//!
//! ## [Clarify]
//! 解決:[`crate::runtime::executor`] 只能 `block_on` 一個 future、
//! [`crate::io::event_loop`] 只會回報 readiness——兩者之間缺的就是
//! 「readiness 怎麼變成 wake、wake 怎麼變成 re-poll」那條線。
//! 本模組把線接上:**多 task run queue + IO futures + 可抽換的 Poller**。
//! Constraints:std-only、單執行緒 runtime(worker 一條)、
//! 一個 fd 同一時刻最多一個等待中的 future(見誠實邊界)。
//!
//! ## [Abstract]
//! reactor 的等待原語抽成三個方法的 [`Poller`] trait——
//! 正是 coderpad-constraints「Abstract the Noise」那個 stub,
//! 但這裡給它**兩個真實作**:
//! - [`ScanPoller`](V0):不偵測任何東西,每個 tick 把所有 armed token
//!   全數回報(spurious by design)。被喚醒的 task 自己 re-try 一次
//!   nonblocking op——每 tick 成本 O(n_armed) 次 syscall + tick 延遲。
//!   **CoderPad 上寫得出來**(純 std)。
//! - [`EpollPoller`](V1):包 [`crate::io::event_loop::EventLoop`],
//!   O(n_ready) 喚醒、零 tick 延遲。executor 與 future 的程式碼
//!   **一行不改**——這就是換 reactor 不動 runtime 的示範,
//!   也是 cost-model「poll vs epoll」那張表的可執行版。
//!
//! ## [Iterate]
//! 1. run queue:`VecDeque<Arc<Task>>`,`Wake` = 把自己 push 回去
//!    (thread pool 骨架換 payload——rehearsals escalation ladder 階段 2)。
//! 2. IO future:nonblocking op → `WouldBlock` → 把 waker 登記進
//!    interest table、`Poller::arm` → `Pending`;readiness 回來 → wake →
//!    re-poll → 再 try 一次。**虛假喚醒是協定的一部分**:醒了不代表好了,
//!    re-try 才算數(與 condvar 的 predicate-wait 同構)。
//! 3. interest table 就是 [`crate::io::fd_registry::FdRegistry`]`<Waker>`:
//!    poller 回報的 token 若已 stale(連線關了、fd 被新連線重用),
//!    `get` 自然回 `None`,過期事件被丟棄——generation 防 stale dispatch
//!    在 runtime 裡的實戰位。
//!
//! ## [Trade-offs]
//! - ScanPoller vs EpollPoller:N=10,000、ready=10 時,前者每 tick 燒
//!   10,000 次 re-try syscall、延遲上限 = tick;後者每次喚醒只碰 10。
//!   同一支測試跑兩個 poller,差的只是效率,不是正確性。
//! - 單執行緒 runtime:無鎖競爭、好推理;代價是 CPU-bound task 會凍住
//!   一切(對,跟 server_evented_inline 同一個病——所以 tokio 有
//!   spawn_blocking,本 repo 的 [`crate::io::file_io_offload`])。
//! - idle 睡覺帶 20ms 上限而非 `None`:跨執行緒的 wake(timer thread 等)
//!   沒有 eventfd 通知路徑,靠這個上限保底。production 的做法是
//!   eventfd 自我喚醒(event_loop 的 `WakeHandle` 就是),聲明後往前走。
//! - **誠實邊界**:一個 fd 同一時刻只存一個 waker——同一條連線同時
//!   讀又寫要兩個 waker 槽(tokio 的 reader/writer 對),本模組不做。
//!
//! ## [Dry-Run]
//! 測試對兩個 poller 各跑一輪同一套:echo roundtrip(readiness 環路走通)、
//! 單執行緒雙 task 交錯(B 不等 A);另有 write-WouldBlock 欠帳路徑說明。

use crate::io::event_loop::{EventLoop, Events, Interest, Token as ElToken, Trigger};
use crate::io::fd_registry::{FdRegistry, Token};
use std::collections::HashSet;
use std::collections::VecDeque;
use std::future::Future;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::fd::{AsRawFd, RawFd};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::time::Duration;

/// reactor 的等待原語。面試裡它以三行 stub 出現(Abstract the Noise);
/// 這裡有兩個真實作:[`ScanPoller`]、[`EpollPoller`]。
///
/// 契約:`wait` 允許 spurious(多報)——被喚醒的一方必須 re-try 判斷
/// 「真的好了嗎」。這讓 V0「全員叫醒」成為合法實作。
pub trait Poller {
    /// 登記/更新 fd 的 interest(upsert 語意,重複 arm 同一 fd 合法)。
    fn arm(&mut self, fd: RawFd, token: u64, interest: Interest) -> io::Result<()>;
    /// fd 關閉前拆掉登記。
    fn disarm(&mut self, fd: RawFd) -> io::Result<()>;
    /// 等 readiness,把 ready 的 token 填進 `out`(先清空)。
    /// 最多睡 `timeout`。
    fn wait(&mut self, out: &mut Vec<u64>, timeout: Duration) -> io::Result<()>;
}

/// V0:O(n) 輪詢 reactor——什麼都不偵測,睡一個 tick 後把所有 armed token
/// 全數回報。正確性靠「spurious 合法 + task 自己 re-try」的契約撐住;
/// 效率是它教學的反面示範(每 tick O(n) 次 re-try + tick 延遲)。
/// std-only、無 epoll——**CoderPad 上寫得出來的那一階**。
pub struct ScanPoller {
    tick: Duration,
    armed: Vec<(RawFd, u64)>,
}

impl ScanPoller {
    pub fn new(tick: Duration) -> Self {
        Self {
            tick,
            armed: Vec::new(),
        }
    }
}

impl Poller for ScanPoller {
    fn arm(&mut self, fd: RawFd, token: u64, _interest: Interest) -> io::Result<()> {
        // interest 不重要:反正全員叫醒,分不分讀寫沒有差別——
        // 這個「不重要」本身就是 O(n) 輪詢粗糙之處的展品。
        match self.armed.iter_mut().find(|(f, _)| *f == fd) {
            Some(slot) => slot.1 = token,
            None => self.armed.push((fd, token)),
        }
        Ok(())
    }

    fn disarm(&mut self, fd: RawFd) -> io::Result<()> {
        self.armed.retain(|(f, _)| *f != fd);
        Ok(())
    }

    fn wait(&mut self, out: &mut Vec<u64>, timeout: Duration) -> io::Result<()> {
        out.clear();
        std::thread::sleep(self.tick.min(timeout));
        out.extend(self.armed.iter().map(|(_, t)| *t)); // 全員叫醒(spurious)
        Ok(())
    }
}

/// V1:epoll reactor——包 [`EventLoop`],O(n_ready) 喚醒、零 tick 延遲。
/// 對 runtime 與 future 而言,它與 ScanPoller 唯一的差別是效率。
pub struct EpollPoller {
    el: EventLoop,
    events: Events,
    /// epoll_ctl ADD 對已註冊 fd 回 EEXIST——記住誰註冊過,upsert 走 MOD。
    registered: HashSet<RawFd>,
}

impl EpollPoller {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            el: EventLoop::new()?,
            events: Events::with_capacity(64),
            registered: HashSet::new(),
        })
    }
}

impl Poller for EpollPoller {
    fn arm(&mut self, fd: RawFd, token: u64, interest: Interest) -> io::Result<()> {
        if self.registered.contains(&fd) {
            self.el
                .reregister(fd, ElToken(token), interest, Trigger::Level)
        } else {
            self.el
                .register(fd, ElToken(token), interest, Trigger::Level)?;
            self.registered.insert(fd);
            Ok(())
        }
    }

    fn disarm(&mut self, fd: RawFd) -> io::Result<()> {
        if self.registered.remove(&fd) {
            self.el.deregister(fd)?;
        }
        Ok(())
    }

    fn wait(&mut self, out: &mut Vec<u64>, timeout: Duration) -> io::Result<()> {
        out.clear();
        self.el.poll(&mut self.events, Some(timeout))?;
        out.extend(self.events.iter().map(|ev| ev.token.0));
        Ok(())
    }
}

/// idle 睡覺的上限:跨執行緒 wake 沒有 eventfd 通知路徑,靠它保底
/// (見模組 doc [Trade-offs])。
const SLEEP_CAP: Duration = Duration::from_millis(20);

struct Reactor {
    poller: Box<dyn Poller + Send>,
    /// interest table:token → waker。generation 讓「連線關了、fd 被重用、
    /// 舊 readiness 事件晚到」的 token 自然查無此人。
    wakers: FdRegistry<Waker>,
}

struct Inner {
    queue: Mutex<VecDeque<Arc<Task>>>,
    reactor: Mutex<Reactor>,
}

struct Task {
    /// None = 已完成(之後的 spurious wake 變 no-op)。
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send>>>>,
    inner: Arc<Inner>,
}

impl Wake for Task {
    /// wake = 把自己 push 回 run queue——thread pool 的骨架,payload 換成
    /// 「re-poll 這個 future」。
    fn wake(self: Arc<Self>) {
        let inner = Arc::clone(&self.inner);
        inner.queue.lock().unwrap().push_back(self);
    }
}

/// 給 IO 物件與 spawn 用的把手(可自由 clone)。
#[derive(Clone)]
pub struct Handle {
    inner: Arc<Inner>,
}

impl Handle {
    /// 丟一個背景 task 進 run queue。
    pub fn spawn(&self, fut: impl Future<Output = ()> + Send + 'static) {
        let task = Arc::new(Task {
            future: Mutex::new(Some(Box::pin(fut))),
            inner: Arc::clone(&self.inner),
        });
        self.inner.queue.lock().unwrap().push_back(task);
    }

    /// IO future 掛起前的登記:waker 進 interest table、fd 進 poller。
    /// 回傳(或沿用)這個 fd 的 registry token。
    fn arm_io(
        &self,
        fd: RawFd,
        token: Option<Token>,
        interest: Interest,
        waker: &Waker,
    ) -> io::Result<Token> {
        let mut reactor = self.inner.reactor.lock().unwrap();
        let token = match token {
            Some(t) => {
                if let Some(slot) = reactor.wakers.get_mut(t) {
                    slot.clone_from(waker); // 契約:最後一次 poll 的 waker 有效
                }
                t
            }
            None => reactor.wakers.register(fd as usize, waker.clone()),
        };
        reactor.poller.arm(fd, token.to_raw(), interest)?;
        Ok(token)
    }

    /// IO 物件關閉時的拆除:poller 與 interest table 都要清
    /// (unregister bump generation——此後晚到的 readiness 事件查無此人)。
    fn disarm_io(&self, fd: RawFd, token: Option<Token>) {
        let mut reactor = self.inner.reactor.lock().unwrap();
        let _ = reactor.poller.disarm(fd);
        if let Some(t) = token {
            reactor.wakers.unregister(t);
        }
    }
}

/// root future 的 waker:設旗標,run loop 每圈檢查。
struct RootWake {
    woken: AtomicBool,
}

impl Wake for RootWake {
    fn wake(self: Arc<Self>) {
        self.woken.store(true, Ordering::Release);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.woken.store(true, Ordering::Release);
    }
}

pub struct Runtime {
    inner: Arc<Inner>,
}

impl Runtime {
    /// poller 可抽換:`ScanPoller`(V0)或 `EpollPoller`(V1)——
    /// runtime 其餘部分一行不改。
    pub fn new(poller: Box<dyn Poller + Send>) -> Self {
        Self {
            inner: Arc::new(Inner {
                queue: Mutex::new(VecDeque::new()),
                reactor: Mutex::new(Reactor {
                    poller,
                    wakers: FdRegistry::new(),
                }),
            }),
        }
    }

    pub fn handle(&self) -> Handle {
        Handle {
            inner: Arc::clone(&self.inner),
        }
    }

    /// 把 root future 跑到完成,途中驅動所有 spawn 的 task 與 reactor。
    ///
    /// 一圈 = ①root 醒了就 poll ②清空 run queue ③poll reactor:
    /// 還有活要幹(root 又醒了)→ timeout 0(只收割 readiness、不睡);
    /// 沒事 → 最多睡 `SLEEP_CAP`。readiness token 查 interest table、
    /// wake 對應 task(stale → 丟棄)。**③每圈都走**——若只在 idle 才
    /// poll reactor,一個自旋的 task(不斷 yield)會把 IO 事件活活餓死。
    pub fn block_on<F: Future>(&self, fut: F) -> F::Output {
        let mut fut = Box::pin(fut);
        let root = Arc::new(RootWake {
            woken: AtomicBool::new(true), // 第一圈先 poll 一次
        });
        let root_waker = Waker::from(Arc::clone(&root));
        let mut ready_tokens = Vec::new();

        loop {
            // ① root
            if root.woken.swap(false, Ordering::AcqRel)
                && let Poll::Ready(v) = fut.as_mut().poll(&mut Context::from_waker(&root_waker))
            {
                return v;
            }

            // ② run queue(逐個拿,poll 期間不持 queue 鎖——poll 內可能 spawn)
            loop {
                let Some(task) = self.inner.queue.lock().unwrap().pop_front() else {
                    break;
                };
                let waker = Waker::from(Arc::clone(&task));
                let mut slot = task.future.lock().unwrap();
                if let Some(f) = slot.as_mut()
                    && f.as_mut().poll(&mut Context::from_waker(&waker)).is_ready()
                {
                    *slot = None; // 完成:釋放 future,之後的 wake 變 no-op
                }
            }

            // ③ reactor:忙 → 只收割(timeout 0);閒 → 睡到有事或 SLEEP_CAP
            let timeout = if root.woken.load(Ordering::Acquire) {
                Duration::ZERO
            } else {
                SLEEP_CAP
            };
            {
                let mut reactor = self.inner.reactor.lock().unwrap();
                let Reactor { poller, wakers } = &mut *reactor;
                poller
                    .wait(&mut ready_tokens, timeout)
                    .expect("poller wait");
                for raw in ready_tokens.drain(..) {
                    // stale token(連線已關、fd 已重用)→ None → 事件丟棄。
                    if let Some(w) = wakers.get(Token::from_raw(raw)) {
                        w.wake_by_ref();
                    }
                }
            } // 先放開 reactor 鎖,再回去 poll(poll 內會要 arm)
        }
    }
}

/// nonblocking listener + accept future。
pub struct AsyncTcpListener {
    listener: TcpListener,
    handle: Handle,
    token: Option<Token>,
}

impl AsyncTcpListener {
    pub fn bind(addr: &str, handle: &Handle) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            handle: handle.clone(),
            token: None,
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn accept(&mut self) -> AcceptFuture<'_> {
        AcceptFuture { io: self }
    }
}

impl Drop for AsyncTcpListener {
    fn drop(&mut self) {
        self.handle
            .disarm_io(self.listener.as_raw_fd(), self.token.take());
    }
}

pub struct AcceptFuture<'a> {
    io: &'a mut AsyncTcpListener,
}

impl Future for AcceptFuture<'_> {
    type Output = io::Result<(AsyncTcpStream, SocketAddr)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        loop {
            match this.io.listener.accept() {
                Ok((stream, peer)) => {
                    let conn = match AsyncTcpStream::from_std(stream, &this.io.handle) {
                        Ok(c) => c,
                        Err(e) => return Poll::Ready(Err(e)),
                    };
                    return Poll::Ready(Ok((conn, peer)));
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    let fd = this.io.listener.as_raw_fd();
                    match this
                        .io
                        .handle
                        .arm_io(fd, this.io.token, Interest::READABLE, cx.waker())
                    {
                        Ok(t) => this.io.token = Some(t),
                        Err(e) => return Poll::Ready(Err(e)),
                    }
                    return Poll::Pending;
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Poll::Ready(Err(e)),
            }
        }
    }
}

/// nonblocking stream + read / write_all futures。
///
/// leaf future 的完整形狀都在這裡:try → `WouldBlock` → arm(waker 登記 +
/// poller 登記)→ `Pending`;醒來 re-try。
pub struct AsyncTcpStream {
    stream: TcpStream,
    handle: Handle,
    token: Option<Token>,
}

impl AsyncTcpStream {
    /// 教學簡化:同步 connect 完成後才轉 nonblocking
    /// (真 async connect 是 EINPROGRESS + 等 WRITABLE,聲明後略過)。
    pub fn connect(addr: SocketAddr, handle: &Handle) -> io::Result<Self> {
        Self::from_std(TcpStream::connect(addr)?, handle)
    }

    fn from_std(stream: TcpStream, handle: &Handle) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            handle: handle.clone(),
            token: None,
        })
    }

    pub fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> ReadFuture<'a> {
        ReadFuture { io: self, buf }
    }

    pub fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> WriteAllFuture<'a> {
        WriteAllFuture {
            io: self,
            buf,
            written: 0,
        }
    }

    /// WouldBlock 路徑共用的登記(interest 依 future 需要傳入)。
    fn arm(&mut self, interest: Interest, waker: &Waker) -> io::Result<()> {
        let fd = self.stream.as_raw_fd();
        self.token = Some(self.handle.arm_io(fd, self.token, interest, waker)?);
        Ok(())
    }
}

impl Drop for AsyncTcpStream {
    fn drop(&mut self) {
        self.handle
            .disarm_io(self.stream.as_raw_fd(), self.token.take());
    }
}

pub struct ReadFuture<'a> {
    io: &'a mut AsyncTcpStream,
    buf: &'a mut [u8],
}

impl Future for ReadFuture<'_> {
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        loop {
            match this.io.stream.read(this.buf) {
                Ok(n) => return Poll::Ready(Ok(n)),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if let Err(e) = this.io.arm(Interest::READABLE, cx.waker()) {
                        return Poll::Ready(Err(e));
                    }
                    return Poll::Pending;
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Poll::Ready(Err(e)),
            }
        }
    }
}

pub struct WriteAllFuture<'a> {
    io: &'a mut AsyncTcpStream,
    buf: &'a [u8],
    written: usize,
}

impl Future for WriteAllFuture<'_> {
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        while this.written < this.buf.len() {
            match this.io.stream.write(&this.buf[this.written..]) {
                Ok(0) => return Poll::Ready(Err(io::ErrorKind::WriteZero.into())),
                Ok(n) => this.written += n,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if let Err(e) = this.io.arm(Interest::WRITABLE, cx.waker()) {
                        return Poll::Ready(Err(e));
                    }
                    return Poll::Pending;
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Poll::Ready(Err(e)),
            }
        }
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::thread;

    fn scan() -> Box<dyn Poller + Send> {
        Box::new(ScanPoller::new(Duration::from_millis(1)))
    }

    fn epoll() -> Box<dyn Poller + Send> {
        Box::new(EpollPoller::new().unwrap())
    }

    /// [Dry-Run] readiness 環路走通:accept → read(Pending → 醒 → 讀到)
    /// → write 回。client 在 std thread 上,延遲送資料逼出 Pending 路徑。
    /// trace(以 read 為例):read → WouldBlock → arm(waker 入 registry、
    /// fd 入 poller)→ Pending → run loop 睡 poller → token 回報 →
    /// registry 查 waker → wake → re-poll → 這次 read 拿到 bytes → Ready。
    fn echo_roundtrip(poller: Box<dyn Poller + Send>) {
        let rt = Runtime::new(poller);
        let handle = rt.handle();
        let mut listener = AsyncTcpListener::bind("127.0.0.1:0", &handle).unwrap();
        let addr = listener.local_addr().unwrap();

        let client = thread::spawn(move || {
            let mut c = TcpStream::connect(addr).unwrap();
            thread::sleep(Duration::from_millis(30)); // 逼 server 端先 Pending
            c.write_all(b"hello").unwrap();
            let mut buf = [0u8; 5];
            c.read_exact(&mut buf).unwrap();
            buf
        });

        rt.block_on(async {
            let (mut conn, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 64];
            let n = conn.read(&mut buf).await.unwrap();
            conn.write_all(&buf[..n]).await.unwrap();
        });

        assert_eq!(&client.join().unwrap(), b"hello");
    }

    #[test]
    fn echo_roundtrip_scan_poller() {
        echo_roundtrip(scan());
    }

    #[test]
    fn echo_roundtrip_epoll_poller() {
        echo_roundtrip(epoll());
    }

    /// 單執行緒雙 task 交錯:兩條連線各由一個 spawn 的 task 服務,
    /// client 端刻意讓 A 先等(晚送資料)、B 先送——若 runtime 是
    /// 「一次只能顧一個」(像 inline server),B 會陪 A 等;
    /// 事實上 B 先完成,證明單執行緒在 await 點之間多工。
    fn two_tasks_interleave(poller: Box<dyn Poller + Send>) {
        let rt = Runtime::new(poller);
        let handle = rt.handle();
        let mut listener = AsyncTcpListener::bind("127.0.0.1:0", &handle).unwrap();
        let addr = listener.local_addr().unwrap();
        let done = Arc::new(AtomicUsize::new(0));

        let client = thread::spawn(move || {
            let mut a = TcpStream::connect(addr).unwrap();
            let mut b = TcpStream::connect(addr).unwrap();
            b.write_all(b"B").unwrap(); // B 先送
            let mut bb = [0u8; 1];
            b.read_exact(&mut bb).unwrap(); // B 先收到回音(A 還在等)
            thread::sleep(Duration::from_millis(50));
            a.write_all(b"A").unwrap(); // A 才送
            let mut ab = [0u8; 1];
            a.read_exact(&mut ab).unwrap();
            (ab[0], bb[0])
        });

        rt.block_on(async {
            for _ in 0..2 {
                let (mut conn, _) = listener.accept().await.unwrap();
                let done = Arc::clone(&done);
                handle.spawn(async move {
                    let mut buf = [0u8; 8];
                    let n = conn.read(&mut buf).await.unwrap();
                    conn.write_all(&buf[..n]).await.unwrap();
                    done.fetch_add(1, Ordering::Relaxed);
                });
            }
            // root 等兩個 task 都完成(粗糙的 async 等待:讓位即可)
            while done.load(Ordering::Relaxed) < 2 {
                crate::runtime::executor::YieldNow::new().await;
            }
        });

        let (a, b) = client.join().unwrap();
        assert_eq!((a, b), (b'A', b'B'));
    }

    #[test]
    fn two_tasks_interleave_scan_poller() {
        two_tasks_interleave(scan());
    }

    #[test]
    fn two_tasks_interleave_epoll_poller() {
        two_tasks_interleave(epoll());
    }
}
