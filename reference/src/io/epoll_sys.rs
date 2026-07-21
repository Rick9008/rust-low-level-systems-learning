//! # epoll_sys —— 最小 epoll syscall 綁定 + 安全 wrapper
//!
//! ## [Clarify]
//! 解決:std 沒有暴露 epoll;面試禁外部 crate ⇒ 自己用 `unsafe extern "C"`
//! 綁最小介面(epoll_create1 / epoll_ctl / epoll_wait / close / eventfd /
//! read / write),再包一層 RAII 安全 API。僅支援 Linux(kernel ≥ 2.6.27)。
//! epoll 是 **readiness model**:告訴你「fd 可讀/可寫了」,IO 還是你自己做;
//! 對照 completion model(io_uring:告訴你「IO 做完了」)。
//!
//! ## [Abstract]
//! errno 的取得不自己綁 `__errno_location`,直接用
//! `std::io::Error::last_os_error()`(std 內建讀 errno)——少一個 unsafe 面。
//!
//! ## [Trade-offs]
//! - `EpollEvent` 在 x86_64 上必須 `repr(C, packed)`:kernel ABI 如此
//!   (歷史包袱:x86_64 對齊 u64 會讓 struct 12→16 bytes,與 32-bit ABI 不相容)。
//!   packed 的代價:不能取欄位參照,只能整值讀寫(Copy)。
//! - `epoll_wait` 的 EINTR(被 signal 打斷)在 wrapper 內重試:
//!   caller 不該為「無事發生」寫錯誤處理。其他 errno 如實上拋。
//! - fd 的生命週期用 RAII(Drop close):忘 close 是 fd 洩漏,
//!   double-close 更糟(fd 號碼可能已被重用,關到別人的)。
//!
//! ## [Dry-Run]
//! 測試:packed layout 尺寸、timeout=0 立即返回、eventfd 喚醒、
//! **LT vs ET 的行為差**(同一 fd 不 drain:LT 每次都報、ET 只報一次)。
//!
//! Production 對照:libc crate(綁定)、mio(跨平台 readiness 抽象)、
//! io_uring(completion model,另一條路,本 repo 不實作)。

use std::io;
use std::os::fd::RawFd;

/// kernel ABI 的 epoll_event。
/// x86_64:packed(12 bytes);其他架構自然對齊(16 bytes)。
#[repr(C)]
#[cfg_attr(target_arch = "x86_64", repr(packed))]
#[derive(Clone, Copy)]
pub struct EpollEvent {
    /// 事件遮罩(EPOLLIN | EPOLLOUT | ...)
    pub events: u32,
    /// 使用者自訂資料,kernel 原樣帶回——放 token/指標都行,我們放 u64 token。
    pub data: u64,
}

// ---- epoll 常數(值抄自 <sys/epoll.h> / <sys/eventfd.h>)----
pub const EPOLL_CLOEXEC: i32 = 0x80000;

pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLL_CTL_DEL: i32 = 2;
pub const EPOLL_CTL_MOD: i32 = 3;

pub const EPOLLIN: u32 = 0x001;
pub const EPOLLPRI: u32 = 0x002;
pub const EPOLLOUT: u32 = 0x004;
pub const EPOLLERR: u32 = 0x008;
pub const EPOLLHUP: u32 = 0x010;
/// 對端關閉寫端(半關閉)。比起「read 回 0」早一步在事件層看到。
pub const EPOLLRDHUP: u32 = 0x2000;
/// edge-triggered:狀態「變化」才通知(vs 預設 LT:狀態「持續」就通知)。
pub const EPOLLET: u32 = 1 << 31;
pub const EPOLLONESHOT: u32 = 1 << 30;

pub const EFD_CLOEXEC: i32 = 0x80000;
pub const EFD_NONBLOCK: i32 = 0x800;

// 最小 syscall 介面。每個宣告上方註明語意與 errno 處理。
// SAFETY(整個 extern 區塊):簽名照抄 man page 的 C 原型;
// 呼叫端負責傳入有效的 fd / 指標 / 長度,錯誤一律由回傳值 -1 + errno 表達。
unsafe extern "C" {
    /// epoll_create1(2):建 epoll 實例,回 fd;-1 + errno(EMFILE/ENOMEM…)。
    /// flags 只有 EPOLL_CLOEXEC(exec 時自動關,防洩漏給子行程)。
    unsafe fn epoll_create1(flags: i32) -> i32;

    /// epoll_ctl(2):對 interest list 增/刪/改。0 或 -1+errno
    /// (EEXIST:重複 ADD;ENOENT:MOD/DEL 不存在;EPERM:fd 不支援 epoll,
    /// 例如 regular file——見 file_io_offload 的 doc)。
    /// event 在 DEL 時可為 null(kernel ≥ 2.6.9 忽略之)。
    unsafe fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: *mut EpollEvent) -> i32;

    /// epoll_wait(2):阻塞至有事件/timeout(毫秒;-1=無限;0=立即)。
    /// 回就緒數;-1+errno(EINTR:被 signal 打斷——wrapper 重試)。
    /// events 必須指向至少 maxevents 個元素的緩衝。
    unsafe fn epoll_wait(epfd: i32, events: *mut EpollEvent, maxevents: i32, timeout: i32) -> i32;

    /// close(2):關 fd。-1+errno(EBADF)。錯誤通常只能記錄,無從復原;
    /// 絕不重試(EINTR 時 fd 狀態未定義,重試可能關到重用的 fd)。
    unsafe fn close(fd: i32) -> i32;

    /// eventfd(2):8-byte 計數器 fd。write 加值、read 取值並歸零
    /// (計數 0 時 read 阻塞/EAGAIN)。用作 event loop 的 self-wake。
    unsafe fn eventfd(initval: u32, flags: i32) -> i32;

    /// read(2)/write(2):這裡只用在 eventfd 的 8-byte 計數器上。
    /// -1+errno(EAGAIN:非阻塞下無資料/緩衝滿)。
    unsafe fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
    unsafe fn write(fd: i32, buf: *const u8, count: usize) -> isize;
}

/// -1 → Err(errno),其餘原樣。syscall 回傳值的統一出口。
fn cvt(ret: i32) -> io::Result<i32> {
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret)
    }
}

/// RAII 的 epoll 實例。Drop 時 close(fd 洩漏在長行程是資源枯竭 bug)。
pub struct Epoll {
    epfd: RawFd,
}

impl Epoll {
    pub fn new() -> io::Result<Self> {
        // SAFETY:無指標參數;回傳值交給 cvt 判錯。
        let epfd = cvt(unsafe { epoll_create1(EPOLL_CLOEXEC) })?;
        Ok(Self { epfd })
    }

    pub fn as_raw_fd(&self) -> RawFd {
        self.epfd
    }

    /// 註冊 fd。`events` 是 EPOLLIN|EPOLLOUT|EPOLLET… 的遮罩,
    /// `token` 會在事件裡原樣帶回(用來找回是哪個連線)。
    pub fn add(&self, fd: RawFd, events: u32, token: u64) -> io::Result<()> {
        let mut ev = EpollEvent {
            events,
            data: token,
        };
        // SAFETY:ev 是活的本地變數,指標在呼叫期間有效;fd 由 caller 保證有效。
        cvt(unsafe { epoll_ctl(self.epfd, EPOLL_CTL_ADD, fd, &mut ev) }).map(|_| ())
    }

    /// 修改既有註冊(換 interest / token)。
    pub fn modify(&self, fd: RawFd, events: u32, token: u64) -> io::Result<()> {
        let mut ev = EpollEvent {
            events,
            data: token,
        };
        // SAFETY:同 add。
        cvt(unsafe { epoll_ctl(self.epfd, EPOLL_CTL_MOD, fd, &mut ev) }).map(|_| ())
    }

    /// 取消註冊。fd close 時 kernel 也會自動移除,但顯式 DEL 讓
    /// 生命週期在 code 裡可見(且 fd 被 dup 過時自動移除並不發生)。
    pub fn delete(&self, fd: RawFd) -> io::Result<()> {
        // SAFETY:DEL 忽略 event 參數(kernel ≥ 2.6.9 允許 null)。
        cvt(unsafe { epoll_ctl(self.epfd, EPOLL_CTL_DEL, fd, std::ptr::null_mut()) }).map(|_| ())
    }

    /// 等事件。回傳寫入 `buf` 的事件數。
    /// timeout:None = 無限等;Some(0) = 立即返回(輪詢)。
    /// EINTR 在此重試——被 signal 打斷不是 caller 的錯誤。
    pub fn wait(&self, buf: &mut [EpollEvent], timeout_ms: Option<i32>) -> io::Result<usize> {
        assert!(!buf.is_empty(), "epoll_wait requires maxevents >= 1");
        let timeout = timeout_ms.unwrap_or(-1);
        loop {
            // SAFETY:buf.as_mut_ptr() 指向 len 個合法元素;epfd 由 RAII 保活。
            let n = unsafe { epoll_wait(self.epfd, buf.as_mut_ptr(), buf.len() as i32, timeout) };
            match cvt(n) {
                Ok(n) => return Ok(n as usize),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue, // EINTR
                Err(e) => return Err(e),
            }
        }
    }
}

impl Drop for Epoll {
    fn drop(&mut self) {
        // SAFETY:epfd 由本結構獨佔,只 close 這一次(Drop 僅一次)。
        // close 失敗無從復原,忽略(不在 Drop 裡 panic)。
        unsafe { close(self.epfd) };
    }
}

/// eventfd 的 RAII wrapper:event loop 的跨執行緒喚醒器。
///
/// 為什麼不用 pipe:eventfd 一個 fd(pipe 要兩個)、8 bytes 固定大小、
/// 計數器語意(多次 notify 合併成一次讀)——正是 wake 訊號要的形狀。
pub struct EventFd {
    fd: RawFd,
}

impl EventFd {
    pub fn new() -> io::Result<Self> {
        // 非阻塞:drain 時計數為 0 直接 EAGAIN,不卡 event loop。
        // SAFETY:無指標參數。
        let fd = cvt(unsafe { eventfd(0, EFD_CLOEXEC | EFD_NONBLOCK) })?;
        Ok(Self { fd })
    }

    pub fn as_raw_fd(&self) -> RawFd {
        self.fd
    }

    /// 喚醒:計數器 +1。任何執行緒可呼叫(write(2) 對 eventfd 是原子的)。
    pub fn notify(&self) -> io::Result<()> {
        let one: u64 = 1;
        // SAFETY:傳入 8 bytes 的本地 u64(eventfd 要求恰好 8 bytes,host 序)。
        let n = unsafe { write(self.fd, (&raw const one).cast::<u8>(), 8) };
        if n == 8 {
            Ok(())
        } else {
            // 計數器到 u64::MAX-1 才會滿——實務不可能;如實上拋。
            Err(io::Error::last_os_error())
        }
    }

    /// 清空計數器(收到 wake 事件後呼叫,否則 LT 模式會一直報可讀)。
    /// 回傳累積的 notify 次數;0 = 沒人叫(EAGAIN)。
    pub fn drain(&self) -> io::Result<u64> {
        let mut count: u64 = 0;
        // SAFETY:8-byte 緩衝指向本地 u64。
        let n = unsafe { read(self.fd, (&raw mut count).cast::<u8>(), 8) };
        if n == 8 {
            Ok(count)
        } else {
            let e = io::Error::last_os_error();
            if e.kind() == io::ErrorKind::WouldBlock {
                Ok(0) // 計數為 0:沒有 pending 的 wake
            } else {
                Err(e)
            }
        }
    }
}

impl Drop for EventFd {
    fn drop(&mut self) {
        // SAFETY:fd 獨佔,close 僅一次。
        unsafe { close(self.fd) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [Dry-Run] ABI 檢查:x86_64 上 packed ⇒ 4 + 8 = 12 bytes。
    /// 若忘了 packed,自然對齊是 16 bytes,epoll_wait 會把事件陣列寫歪
    /// ——這種 bug 的表象是「token 值錯亂」,離根因很遠。
    #[test]
    fn boundary_epoll_event_abi_layout() {
        #[cfg(target_arch = "x86_64")]
        assert_eq!(std::mem::size_of::<EpollEvent>(), 12);
        #[cfg(not(target_arch = "x86_64"))]
        assert_eq!(std::mem::size_of::<EpollEvent>(), 16);
    }

    /// boundary:空 epoll、timeout=0 → 立即回 0 事件(不阻塞)。
    #[test]
    fn boundary_wait_timeout_zero_returns_immediately() {
        let ep = Epoll::new().unwrap();
        let mut buf = [EpollEvent { events: 0, data: 0 }; 8];
        assert_eq!(ep.wait(&mut buf, Some(0)).unwrap(), 0);
    }

    /// eventfd 全流程 trace:
    ///   add(efd, EPOLLIN, token=7) → notify ×2(計數=2)→
    ///   wait → 1 個事件、readable、token=7 → drain → 2 →
    ///   wait(0) → 0 事件(計數歸零,LT 不再報)
    #[test]
    fn eventfd_notify_wait_drain_roundtrip() {
        let ep = Epoll::new().unwrap();
        let efd = EventFd::new().unwrap();
        ep.add(efd.as_raw_fd(), EPOLLIN, 7).unwrap();
        efd.notify().unwrap();
        efd.notify().unwrap();
        let mut buf = [EpollEvent { events: 0, data: 0 }; 8];
        let n = ep.wait(&mut buf, Some(1000)).unwrap();
        assert_eq!(n, 1);
        // packed:欄位先按值複製到區域變數(assert_eq! 取參照會踩 E0793)
        let (events, data) = (buf[0].events, buf[0].data);
        assert_eq!(data, 7);
        assert!(events & EPOLLIN != 0);
        assert_eq!(efd.drain().unwrap(), 2); // 兩次 notify 合併
        assert_eq!(ep.wait(&mut buf, Some(0)).unwrap(), 0);
    }

    /// **LT vs ET 的核心行為差**(不 drain,連續 wait 兩次):
    ///   LT:資料還在 → 兩次都報(狀態持續)
    ///   ET:只有第一次報(只在 0→非 0 的「變化」時觸發)
    /// 這就是 ET 必須「讀到 EAGAIN 為止」的原因:漏讀就永遠等不到下一次通知。
    #[test]
    fn boundary_level_reports_again_edge_reports_once() {
        // LT
        let ep = Epoll::new().unwrap();
        let efd = EventFd::new().unwrap();
        ep.add(efd.as_raw_fd(), EPOLLIN, 1).unwrap();
        efd.notify().unwrap();
        let mut buf = [EpollEvent { events: 0, data: 0 }; 8];
        assert_eq!(ep.wait(&mut buf, Some(100)).unwrap(), 1);
        assert_eq!(ep.wait(&mut buf, Some(100)).unwrap(), 1); // 沒 drain:LT 再報

        // ET
        let ep2 = Epoll::new().unwrap();
        let efd2 = EventFd::new().unwrap();
        ep2.add(efd2.as_raw_fd(), EPOLLIN | EPOLLET, 2).unwrap();
        efd2.notify().unwrap();
        assert_eq!(ep2.wait(&mut buf, Some(100)).unwrap(), 1);
        assert_eq!(ep2.wait(&mut buf, Some(50)).unwrap(), 0); // 沒 drain:ET 不再報
        efd2.notify().unwrap(); // 新的「變化」(計數 1→2)
        assert_eq!(ep2.wait(&mut buf, Some(100)).unwrap(), 1); // ET 再觸發
    }

    /// boundary:timeout 真的等了 ~50ms(空 epoll 上不會提前返回)。
    #[test]
    fn boundary_timeout_actually_waits() {
        let ep = Epoll::new().unwrap();
        let mut buf = [EpollEvent { events: 0, data: 0 }; 1];
        let start = std::time::Instant::now();
        assert_eq!(ep.wait(&mut buf, Some(50)).unwrap(), 0);
        assert!(start.elapsed() >= std::time::Duration::from_millis(45));
    }

    /// modify / delete 的基本流:MOD 換 token 後事件帶新 token;DEL 後無事件。
    #[test]
    fn modify_and_delete_registration() {
        let ep = Epoll::new().unwrap();
        let efd = EventFd::new().unwrap();
        ep.add(efd.as_raw_fd(), EPOLLIN, 1).unwrap();
        ep.modify(efd.as_raw_fd(), EPOLLIN, 99).unwrap();
        efd.notify().unwrap();
        let mut buf = [EpollEvent { events: 0, data: 0 }; 8];
        assert_eq!(ep.wait(&mut buf, Some(100)).unwrap(), 1);
        let data = buf[0].data; // packed:按值複製
        assert_eq!(data, 99);
        ep.delete(efd.as_raw_fd()).unwrap();
        efd.notify().unwrap();
        assert_eq!(ep.wait(&mut buf, Some(50)).unwrap(), 0);
    }
}
