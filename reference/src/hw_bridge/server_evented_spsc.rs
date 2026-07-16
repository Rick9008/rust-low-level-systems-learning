//! SPSC evented server:與 [`super::server_evented`] 同架構,換掉佇列。
//!
//! 觀察:evented server 是 **1 條 IO thread × 1 條 command worker**——
//! 天然恰好一產一消,兩條通道(job 去程、response 回程)各自都是
//! SPSC pair。原版用 `Mutex<VecDeque>`(ThreadPool 內部)+ `Mutex<Vec>`
//! (outbox);本版換成兩條 [`crate::spsc_ring`]:
//!
//! ```text
//! [IO thread] --- SpscRing<(token, RawFrame)> ---> [worker(獨占 handler)]
//!            <--- SpscRing<(token, resp bytes)> ---     │
//!      ▲                                                │
//!      └────────────── eventfd wake ────────────────────┘
//! ```
//!
//! 換佇列買到什麼(對照 docs/cost-model.md):
//! - 熱路徑 push/pop 零鎖零 syscall(~10–50ns vs mutex 有競爭 µs 級);
//!   買的是 **p99.9**——IO thread 不會因 worker 持鎖被 preempt 而卡住。
//! - handler **免 `Arc<Mutex>`**:單 worker 獨占它(`H` 直接 move 進
//!   worker thread)——「並發策略由 server 端決定」的極簡版。
//! - 保序不變:單 worker FIFO,與原版同一條性質。
//!
//! 兩端的睡/醒:
//! - worker 沒事:spin-then-park + 掛牌握手(與
//!   [`crate::signal_pipeline`] 同款,含 SeqCst fence 的 SB litmus 分析);
//! - IO thread 沒事:睡在 `epoll_wait`,worker 用 **eventfd** 叫醒——
//!   「對 epoll loop 而言,unpark 就是 eventfd」。
//!
//! 誠實邊界:job ring 滿時 IO thread 短暫 spin(worker 是唯一消費者,
//! 必然騰出;production 的正解是關掉該連線的 EPOLLIN 做真 backpressure,
//! 見 clarify playbook Q1)。resp ring 滿時 worker 先 wake 再 spin
//! (IO thread 醒來 drain 就有空位——wake 在前,否則兩邊互等)。

use super::framer::FrameReader;
use super::handler::CommandHandler;
use super::protocol::{
    Command, ERR_BAD_PAYLOAD, ERR_UNKNOWN_OPCODE, RawFrame, Response, WireError,
};
use crate::event_loop::{Event, EventLoop, Events, Interest, Token, Trigger, WakeHandle};
use crate::spsc_ring::{Consumer, Producer, channel};
use std::collections::{HashMap, VecDeque};
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering, fence};
use std::thread::{self, JoinHandle, Thread};

const LISTENER_TOKEN: Token = Token(0);
const RING_CAP: usize = 1024;
const SPIN_LIMIT: u32 = 100;

struct Conn {
    stream: TcpStream,
    framer: FrameReader,
    out: VecDeque<u8>,
    registered: Interest,
    read_eof: bool,
    in_flight: usize,
}

#[derive(Clone)]
pub struct SpscShutdown {
    stop: Arc<AtomicBool>,
    wake: WakeHandle,
}

impl SpscShutdown {
    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::Release);
        let _ = self.wake.wake();
    }
}

pub struct SpscEventedServer {
    el: EventLoop,
    events: Events,
    listener: TcpListener,
    conns: HashMap<u64, Conn>,
    next_token: u64,
    job_tx: Producer<(u64, RawFrame)>,
    resp_rx: Consumer<(u64, Vec<u8>)>,
    worker_parked: Arc<AtomicBool>,
    worker: Thread,
    worker_join: Option<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    worker_stop: Arc<AtomicBool>,
}

impl SpscEventedServer {
    pub fn bind<H: CommandHandler + 'static>(addr: &str, handler: H) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let el = EventLoop::new()?;
        el.register(
            listener.as_raw_fd(),
            LISTENER_TOKEN,
            Interest::READABLE,
            Trigger::Level,
        )?;

        let (job_tx, job_rx) = channel(RING_CAP);
        let (resp_tx, resp_rx) = channel(RING_CAP);
        let worker_parked = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::new(AtomicBool::new(false));
        let wake = el.wake_handle();
        let (parked2, stop2) = (Arc::clone(&worker_parked), Arc::clone(&worker_stop));
        let worker_join = thread::Builder::new()
            .name("spsc-command-worker".into())
            .spawn(move || worker_loop(job_rx, resp_tx, handler, &parked2, &stop2, wake))
            .expect("spawn command worker");
        let worker = worker_join.thread().clone();

        Ok(Self {
            el,
            events: Events::with_capacity(64),
            listener,
            conns: HashMap::new(),
            next_token: 1,
            job_tx,
            resp_rx,
            worker_parked,
            worker,
            worker_join: Some(worker_join),
            stop: Arc::new(AtomicBool::new(false)),
            worker_stop,
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn shutdown_handle(&self) -> SpscShutdown {
        SpscShutdown {
            stop: Arc::clone(&self.stop),
            wake: self.el.wake_handle(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        while !self.stop.load(Ordering::Acquire) {
            self.el.poll(&mut self.events, None)?;
            if self.events.woken() {
                self.route_resps();
            }
            let batch: Vec<Event> = self.events.iter().copied().collect();
            for ev in batch {
                if ev.token == LISTENER_TOKEN {
                    self.accept_all()?;
                } else {
                    self.drive_conn(ev);
                }
            }
        }
        Ok(())
    }

    fn accept_all(&mut self) -> io::Result<()> {
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    stream.set_nonblocking(true)?;
                    let token = Token(self.next_token);
                    self.next_token += 1;
                    self.el.register(
                        stream.as_raw_fd(),
                        token,
                        Interest::READABLE,
                        Trigger::Level,
                    )?;
                    self.conns.insert(
                        token.0,
                        Conn {
                            stream,
                            framer: FrameReader::new(),
                            out: VecDeque::new(),
                            registered: Interest::READABLE,
                            read_eof: false,
                            in_flight: 0,
                        },
                    );
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
    }

    /// 回程:drain resp ring(worker 每 push 一筆就 eventfd wake 一次)。
    fn route_resps(&mut self) {
        while let Some((token, bytes)) = self.resp_rx.pop() {
            let Some(conn) = self.conns.get_mut(&token) else {
                continue; // 連線已死:回應無處可去,丟棄
            };
            conn.in_flight -= 1;
            conn.out.extend(bytes);
            let dead = flush(conn);
            self.finish_conn_state(token, dead);
        }
    }

    fn drive_conn(&mut self, ev: Event) {
        let Some(conn) = self.conns.get_mut(&ev.token.0) else {
            return;
        };
        let mut dead = ev.error;

        if !dead && (ev.readable || ev.peer_closed) && !conn.read_eof {
            let mut buf = [0u8; 4096];
            loop {
                match conn.stream.read(&mut buf) {
                    Ok(0) => {
                        conn.read_eof = true;
                        break;
                    }
                    Ok(n) => conn.framer.feed(&buf[..n]),
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(_) => {
                        dead = true;
                        break;
                    }
                }
            }
            while !dead {
                match conn.framer.next_frame() {
                    Ok(Some(frame)) => {
                        conn.in_flight += 1;
                        // 去程:SPSC push,零鎖零 syscall(快路徑)。
                        // 滿:短暫 spin——worker 是唯一消費者必然騰出
                        // (production:關 EPOLLIN 做真 backpressure)。
                        let mut job = (ev.token.0, frame);
                        loop {
                            match self.job_tx.push(job) {
                                Ok(()) => break,
                                Err(j) => {
                                    job = j;
                                    std::hint::spin_loop();
                                }
                            }
                        }
                        // 掛牌握手的 producer 半邊(SB litmus,
                        // 見 signal_pipeline 的手 trace)。
                        fence(Ordering::SeqCst);
                        if self.worker_parked.load(Ordering::Relaxed) {
                            self.worker.unpark();
                        }
                    }
                    Ok(None) => break,
                    Err(_) => {
                        dead = true;
                    }
                }
            }
        }

        if !dead && ev.writable && !conn.out.is_empty() {
            dead = flush(conn);
        }
        self.finish_conn_state(ev.token.0, dead);
    }

    fn finish_conn_state(&mut self, token: u64, dead: bool) {
        let Some(conn) = self.conns.get_mut(&token) else {
            return;
        };
        let finished = conn.read_eof && conn.out.is_empty() && conn.in_flight == 0;
        if dead || finished {
            let conn = self.conns.remove(&token).unwrap();
            let _ = self.el.deregister(conn.stream.as_raw_fd());
            return;
        }
        let want = Interest {
            readable: !conn.read_eof,
            writable: !conn.out.is_empty(),
        };
        if want != conn.registered {
            if self
                .el
                .reregister(conn.stream.as_raw_fd(), Token(token), want, Trigger::Level)
                .is_ok()
            {
                conn.registered = want;
            } else {
                let conn = self.conns.remove(&token).unwrap();
                let _ = self.el.deregister(conn.stream.as_raw_fd());
            }
        }
    }
}

impl Drop for SpscEventedServer {
    /// worker 收工:置 stop → unpark(可能睡著)→ join。
    /// worker 的迴圈保證 stop 前已 push 的 job 都處理完(drain 語意)。
    fn drop(&mut self) {
        self.worker_stop.store(true, Ordering::Release);
        self.worker.unpark();
        if let Some(j) = self.worker_join.take() {
            let _ = j.join();
        }
    }
}

/// worker:pop job → handler → push resp + eventfd wake。
/// 沒事:spin-then-park + 掛牌握手(與 signal_pipeline 同款)。
fn worker_loop<H: CommandHandler>(
    mut jobs: Consumer<(u64, RawFrame)>,
    mut resps: Producer<(u64, Vec<u8>)>,
    mut handler: H,
    parked: &AtomicBool,
    stop: &AtomicBool,
    wake: WakeHandle,
) {
    let mut spins: u32 = 0;
    loop {
        let job = match jobs.pop() {
            Some(j) => {
                spins = 0;
                Some(j)
            }
            None => {
                if stop.load(Ordering::Acquire) {
                    return; // pop None ⇒ 殘料 drain 完畢
                }
                if spins < SPIN_LIMIT {
                    spins += 1;
                    std::hint::spin_loop();
                    continue;
                }
                spins = 0;
                idle_park(&mut jobs, parked, stop)
            }
        };
        let Some((token, frame)) = job else {
            continue;
        };

        let resp = match Command::try_from_frame(&frame) {
            Ok(cmd) => handler.handle(cmd), // 免鎖:worker 獨占 handler
            Err(WireError::UnknownOpcode(_)) => Response::Error {
                code: ERR_UNKNOWN_OPCODE,
            },
            Err(WireError::BadPayloadLen { .. }) => Response::Error {
                code: ERR_BAD_PAYLOAD,
            },
        };
        // 回程:滿的話**先 wake 再 spin**——IO thread 醒來 drain 才有空位;
        // 順序反過來就是兩邊互等。
        let mut item = (token, resp.encode());
        loop {
            match resps.push(item) {
                Ok(()) => break,
                Err(it) => {
                    item = it;
                    let _ = wake.wake();
                    std::hint::spin_loop();
                }
            }
        }
        let _ = wake.wake(); // IO thread 可能睡在 epoll_wait
    }
}

/// 掛牌握手(與 signal_pipeline::idle_park 同款,SB litmus 見該模組 doc)。
fn idle_park(
    jobs: &mut Consumer<(u64, RawFrame)>,
    parked: &AtomicBool,
    stop: &AtomicBool,
) -> Option<(u64, RawFrame)> {
    parked.store(true, Ordering::SeqCst);
    fence(Ordering::SeqCst);
    if let Some(j) = jobs.pop() {
        parked.store(false, Ordering::Release);
        return Some(j);
    }
    if !stop.load(Ordering::Acquire) {
        thread::park();
    }
    parked.store(false, Ordering::Release);
    None
}

/// 與 server_evented::flush 同款。
fn flush(conn: &mut Conn) -> bool {
    while !conn.out.is_empty() {
        let (front, _) = conn.out.as_slices();
        match conn.stream.write(front) {
            Ok(0) => return false,
            Ok(n) => {
                conn.out.drain(..n);
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return false,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return true,
        }
    }
    false
}
