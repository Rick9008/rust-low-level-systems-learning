# 如果可以用 libc:epoll 長什麼樣

本 repo 的 `epoll_sys` 自己寫 `unsafe extern "C"`,是**面試約束**(禁外部 crate),
不是 production 建議。這份文件回答一個實際問題:**允許用 `libc` crate 的話,差別在哪?**

前置:[epoll_sys](epoll_sys.md)(手寫綁定)、[event_loop](event_loop.md)(readiness model)。

## 一句話:libc 給你宣告,不給你安全

`libc` 是 **FFI 宣告的集合**,不是抽象層。它給你的是:

- 型別定義(`libc::epoll_event`,而且**已經幫你處理好 x86_64 的 `repr(packed)`**)
- `unsafe extern "C"` 函式宣告(`epoll_create1` / `epoll_ctl` / `epoll_wait`)
- 常數(`EPOLLIN` / `EPOLLET` / `EPOLL_CTL_ADD` …)

它**不會**給你的:

- 安全性——每個呼叫還是 `unsafe`,回傳值還是 `-1`,錯誤還是躺在 `errno`
- RAII——epfd 不會自己關,你還是得寫 `Drop`
- 型別安全——token 還是 raw `u64`,fd 還是 raw `i32`
- 抽象——LT/ET、interest 更新、EINTR 重試,一樣是你的事

**換句話說:用了 libc,`epoll_sys.rs` 那 334 行你還是得寫得出來,只是不用抄 `<sys/epoll.h>` 的數字。**
這正是為什麼面試禁用 crate 也考得出東西——libc 省掉的是抄寫,不是理解。

## 省掉的兩件事(都是真的有價值)

### 1. `repr(packed)` 的架構差異

手寫版必須自己知道這件事(`reference/src/epoll_sys.rs:35`):

```rust
#[repr(C)]
#[cfg_attr(target_arch = "x86_64", repr(packed))]   // ← 你得知道 x86_64 要 packed
pub struct EpollEvent { pub events: u32, pub data: u64 }
```

x86_64 上 kernel 期待 `u32` 緊接著 `u64`,**中間沒有 padding**(12 bytes)。
自然對齊會是 16 bytes,多出來的 4 bytes 洞會讓你把垃圾遞給 kernel、
或是把 `data` 從錯誤的偏移量讀出來——事件全部錯位,而且不會 crash,只會行為詭異。

libc 已經標好了。實測(`libc 0.2`,x86_64):

```
size_of::<libc::epoll_event>()  = 12
align_of::<libc::epoll_event>() = 1     ← packed
```

### 2. errno

手寫版要自己宣告 `__errno_location()` 才拿得到 errno。用 libc(其實用 std 就夠):

```rust
if r < 0 { return Err(io::Error::last_os_error()); }   // ← 它就是在讀 errno
```

`io::Error::last_os_error()` 是 std 的東西,**不需要 libc**。這是手寫版唯一真正繞遠路的地方。

## 沒省掉、而且會咬你的三個地雷

### 地雷一:常數是 `i32`,欄位是 `u32`,而 `EPOLLET` 是**負數**

實測 `libc 0.2`:

| 常數 | 型別 | 值 |
|---|---|---|
| `libc::EPOLLIN` | `c_int` = `i32` | `1` |
| `libc::EPOLLOUT` | `i32` | `4` |
| `libc::EPOLLRDHUP` | `i32` | `8192` |
| **`libc::EPOLLET`** | `i32` | **`-2147483648`** |

`0x80000000` 塞進 `i32` 就是負的。而 `epoll_event.events` 的型別是 **`u32`**。所以:

```rust
ep.add(fd, (libc::EPOLLIN | libc::EPOLLET) as u32, token)?;
//                                        ^^^^^^^ 不能省
```

`1 | -2147483648` 在 i32 下是 `-2147483647`,`as u32` 之後是 `0x80000001` —— 剛好正確。
但只要你中途做了任何算術(比較大小、右移),號誌就會咬你。

本 repo 的手寫版直接把常數宣成 `u32`(`epoll_sys.rs:52-61`),沒有這個問題。
**這是手寫版比 libc 好用的地方**,不是巧合——libc 忠實反映 C 的 `int`,而 C 在這裡本來就設計得很爛。

### 地雷二:packed 欄位不能取參考

```rust
for ev in &events[..n] {
    let token = ev.u64;          // ✅ 複製出來
    // let t = &ev.u64;          // ❌ E0793:packed 欄位的參考可能未對齊 = UB
}
```

這條規則 libc 幫不了你——它是 Rust 的語言規則,而 `epoll_event` 恰好是 packed。
(Rust 2024 起這是 hard error,以前只是 warning。)

### 地雷三:`EINTR` 不是錯誤

`epoll_wait` 被信號打斷會回 `-1` / `EINTR`。**你必須重試,不能往上丟。**
這在 libc 和手寫版是一模一樣的,而且是最常被忘記的一行:

```rust
if e.kind() == io::ErrorKind::Interrupted { continue; }
```

## 完整可跑的最小版本

以下這段是**真的編過、真的跑過**的(`libc = "0.2"`,edition 2024,三個 `nc` client 打進去)。
對照 `reference/src/epoll_sys.rs` 看,你會發現結構一模一樣——只是 `unsafe extern "C"` 區塊消失了。

```rust
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::fd::{AsRawFd, RawFd};

/// 安全包裝:擁有 epfd,Drop 時關掉。libc 不會幫你做這件事。
struct Epoll(RawFd);

impl Epoll {
    fn new() -> io::Result<Self> {
        // SAFETY: flags 合法;失敗回 -1,errno 由 last_os_error 讀。
        let fd = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
        if fd < 0 { return Err(io::Error::last_os_error()); }
        Ok(Self(fd))
    }

    fn add(&self, fd: RawFd, events: u32, token: u64) -> io::Result<()> {
        let mut ev = libc::epoll_event { events, u64: token };
        // SAFETY: self.0 有效;&mut ev 在呼叫期間有效。
        let r = unsafe { libc::epoll_ctl(self.0, libc::EPOLL_CTL_ADD, fd, &mut ev) };
        if r < 0 { return Err(io::Error::last_os_error()); }
        Ok(())
    }

    fn wait(&self, buf: &mut [libc::epoll_event], timeout_ms: i32) -> io::Result<usize> {
        loop {
            // SAFETY: buf 是有效的可寫切片,長度如實傳遞。
            let n = unsafe {
                libc::epoll_wait(self.0, buf.as_mut_ptr(), buf.len() as i32, timeout_ms)
            };
            if n >= 0 { return Ok(n as usize); }
            let e = io::Error::last_os_error();
            if e.kind() == io::ErrorKind::Interrupted { continue; } // EINTR:重試,不是錯誤
            return Err(e);
        }
    }
}

impl Drop for Epoll {
    // SAFETY: self.0 是我們自己 create 出來的、還沒關過。
    fn drop(&mut self) { unsafe { libc::close(self.0) }; }
}

const LISTENER: u64 = 0;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;   // 鐵律:event loop 裡的每個 fd 都必須 nonblocking
    println!("listening on {}", listener.local_addr()?);

    let ep = Epoll::new()?;
    ep.add(listener.as_raw_fd(), libc::EPOLLIN as u32, LISTENER)?;  // as u32 不能省

    let mut conns: HashMap<u64, TcpStream> = HashMap::new();
    let mut next: u64 = 1;
    let mut events = vec![libc::epoll_event { events: 0, u64: 0 }; 16];

    loop {
        let n = ep.wait(&mut events, 2000)?;
        if n == 0 { break; }                       // timeout

        for ev in &events[..n] {
            let token = ev.u64;                    // packed 欄位:複製,不能取參考

            if token == LISTENER {
                loop {                             // 照規矩抽乾(ET 下這是義務,LT 下是效率)
                    match listener.accept() {
                        Ok((s, _)) => {
                            s.set_nonblocking(true)?;
                            let t = next; next += 1;
                            ep.add(s.as_raw_fd(), libc::EPOLLIN as u32, t)?;
                            conns.insert(t, s);
                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                        Err(e) => return Err(e),
                    }
                }
            } else if let Some(s) = conns.get_mut(&token) {
                let mut buf = [0u8; 256];
                match s.read(&mut buf) {
                    Ok(0) => { conns.remove(&token); }          // EOF
                    Ok(k) => { s.write_all(&buf[..k])?; }       // 註:真實世界要處理短寫
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(_) => { conns.remove(&token); }
                }
            }
        }
    }
    Ok(())
}
```

> `write_all` 在這裡是**示範用的作弊**:nonblocking socket 上它可能寫不完就回 `WouldBlock`。
> 正確做法是緩存欠帳 + 掛 `EPOLLOUT`,見 [tcp_echo](tcp_echo.md)。
> 這正好說明了 libc 的極限:**它讓你把 syscall 叫出來,但 backpressure 的狀態機還是你自己的事。**

## 往上還有兩層

| 層級 | 你得自己做的事 | 何時用 |
|---|---|---|
| **手寫 `extern "C"`**(本 repo) | 全部,含抄常數、`repr(packed)`、errno | 面試禁 crate;或你想真的懂 |
| **`libc`** | 除了型別宣告以外的全部:RAII、token 管理、LT/ET、EINTR、backpressure | 你要 Linux-only 的極致控制 |
| **`mio`** | backpressure 與協定。它給你 `Poll` / `Token` / `Events` / `Interest`,**跨平台**(Linux epoll、macOS kqueue、Windows IOCP) | 你要自己寫 runtime,但不想寫三套 |
| **`tokio`** | 幾乎不用。`async fn` + `.await`,executor / reactor / waker 全包 | 99% 的 production |

**`mio` 是本 repo 的 `event_loop` 模組的 production 對應物**——你把 `epoll_sys` + `event_loop` 寫完,
你寫的就是一個 Linux-only 的 mini mio。而 tokio 的 reactor 底下就是 mio。

這條線值得記住,因為面試問「你怎麼實作 async runtime」時,答案的骨架就是它:

```
epoll (kernel)  →  mio (readiness 抽象)  →  reactor (event → Waker)  →  executor (poll future)
                                             ↑ 本 repo 的 event_loop      ↑ 本 repo 的 executor
```

## 為什麼本 repo 仍然不用 libc

README 的「誠實聲明」寫得很清楚:std-only 是面試約束。但還有第二個理由——

**手寫那 334 行,你會被迫回答「這個 `u64` 為什麼是 `u64`」、「為什麼要 packed」、「errno 到底住在哪」。**
用 libc,這些問題會被 `use libc::*` 一行吃掉,而它們正是面試官要問的東西。

反過來說:**production 請用 libc(或直接用 mio)。** 自己抄 `<sys/epoll.h>` 的數字,
在別的架構上會靜默錯掉——本 repo 的 `EPOLLET = 1 << 31` 在所有 Linux 架構上都對,
但那是因為 epoll 的常數剛好跨架構一致;`EPOLL_CLOEXEC = 0x80000` 這種就不保證了。
自寫綁定是**教學器材**,不是可攜的工程。
