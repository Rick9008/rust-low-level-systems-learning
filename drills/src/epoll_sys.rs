//! drill:epoll_sys —— 填 syscall wrapper 的錯誤處理紀律。
//!
//! 已給:extern 宣告、常數、RAII 結構、add/modify/delete。
//! 要填:`Epoll::wait`(EINTR 重試)與 `EventFd::drain`(EAGAIN → 0)。
//! 這兩個 errno 的處理是「syscall 包裝」面試的固定考點。

use std::io;
use std::os::fd::RawFd;

#[repr(C)]
#[cfg_attr(target_arch = "x86_64", repr(packed))]
#[derive(Clone, Copy)]
pub struct EpollEvent {
    pub events: u32,
    pub data: u64,
}

pub const EPOLL_CLOEXEC: i32 = 0x80000;
pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLL_CTL_DEL: i32 = 2;
pub const EPOLL_CTL_MOD: i32 = 3;
pub const EPOLLIN: u32 = 0x001;
pub const EPOLLOUT: u32 = 0x004;
pub const EPOLLET: u32 = 1 << 31;
pub const EFD_CLOEXEC: i32 = 0x80000;
pub const EFD_NONBLOCK: i32 = 0x800;

// SAFETY:簽名照抄 man page;錯誤以 -1 + errno 表達。
unsafe extern "C" {
    unsafe fn epoll_create1(flags: i32) -> i32;
    unsafe fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: *mut EpollEvent) -> i32;
    unsafe fn epoll_wait(epfd: i32, events: *mut EpollEvent, maxevents: i32, timeout: i32) -> i32;
    unsafe fn close(fd: i32) -> i32;
    unsafe fn eventfd(initval: u32, flags: i32) -> i32;
    unsafe fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
    unsafe fn write(fd: i32, buf: *const u8, count: usize) -> isize;
}

fn cvt(ret: i32) -> io::Result<i32> {
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret)
    }
}

pub struct Epoll {
    epfd: RawFd,
}

impl Epoll {
    pub fn new() -> io::Result<Self> {
        // SAFETY:無指標參數。
        let epfd = cvt(unsafe { epoll_create1(EPOLL_CLOEXEC) })?;
        Ok(Self { epfd })
    }

    pub fn add(&self, fd: RawFd, events: u32, token: u64) -> io::Result<()> {
        let mut ev = EpollEvent {
            events,
            data: token,
        };
        // SAFETY:ev 是活的本地變數。
        cvt(unsafe { epoll_ctl(self.epfd, EPOLL_CTL_ADD, fd, &mut ev) }).map(|_| ())
    }

    /// spec:epoll_wait 包裝。
    /// - timeout_ms:None → -1(無限等);Some(ms) 原樣傳
    /// - 迴圈呼叫:回傳 -1 且 errno 是 EINTR(`io::ErrorKind::Interrupted`)
    ///   → **重試**(被 signal 打斷不是 caller 的錯);其他錯誤上拋
    /// - 成功回寫入的事件數(n as usize)
    ///
    /// SAFETY 提示:`buf.as_mut_ptr()` + `buf.len() as i32`。
    pub fn wait(&self, buf: &mut [EpollEvent], timeout_ms: Option<i32>) -> io::Result<usize> {
        assert!(!buf.is_empty());
        todo!("spec: loop {{ unsafe epoll_wait; EINTR continue; 其他 return }}")
    }
}

impl Drop for Epoll {
    fn drop(&mut self) {
        // SAFETY:fd 獨佔,close 僅一次;錯誤無從復原,吞掉。
        unsafe { close(self.epfd) };
    }
}

pub struct EventFd {
    fd: RawFd,
}

impl EventFd {
    pub fn new() -> io::Result<Self> {
        // SAFETY:無指標參數。
        let fd = cvt(unsafe { eventfd(0, EFD_CLOEXEC | EFD_NONBLOCK) })?;
        Ok(Self { fd })
    }

    pub fn as_raw_fd(&self) -> RawFd {
        self.fd
    }

    pub fn notify(&self) -> io::Result<()> {
        let one: u64 = 1;
        // SAFETY:恰好 8 bytes 的本地 u64。
        let n = unsafe { write(self.fd, (&raw const one).cast::<u8>(), 8) };
        if n == 8 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    /// spec:讀 8 bytes 取出計數並歸零。
    /// - 讀到 8 bytes → Ok(計數值)
    /// - 失敗且 errno 是 EAGAIN(`io::ErrorKind::WouldBlock`)→ Ok(0)
    ///   (非阻塞 + 計數為 0 = 沒人叫,不是錯誤)
    /// - 其他錯誤上拋
    ///
    /// SAFETY 提示:`(&raw mut count).cast::<u8>()`。
    pub fn drain(&self) -> io::Result<u64> {
        todo!("spec: read 8 bytes;WouldBlock → Ok(0)")
    }
}

impl Drop for EventFd {
    fn drop(&mut self) {
        // SAFETY:同 Epoll。
        unsafe { close(self.fd) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// boundary:timeout=0 立即返回、eventfd notify→wait→drain 全流程。
    #[test]
    #[ignore = "填完 wait/drain 後移除"]
    fn eventfd_roundtrip() {
        let ep = Epoll::new().unwrap();
        let efd = EventFd::new().unwrap();
        ep.add(efd.as_raw_fd(), EPOLLIN, 7).unwrap();
        let mut buf = [EpollEvent { events: 0, data: 0 }; 8];
        assert_eq!(ep.wait(&mut buf, Some(0)).unwrap(), 0); // 還沒 notify
        efd.notify().unwrap();
        efd.notify().unwrap();
        assert_eq!(ep.wait(&mut buf, Some(1000)).unwrap(), 1);
        let data = buf[0].data; // packed:按值複製再比對
        assert_eq!(data, 7);
        assert_eq!(efd.drain().unwrap(), 2); // 兩次 notify 合併
        assert_eq!(efd.drain().unwrap(), 0); // 空了:EAGAIN → 0
        assert_eq!(ep.wait(&mut buf, Some(0)).unwrap(), 0);
    }

    /// boundary:timeout 真的等(空 epoll 不提前返回)。
    #[test]
    #[ignore = "填完 wait 後移除"]
    fn timeout_actually_waits() {
        let ep = Epoll::new().unwrap();
        let mut buf = [EpollEvent { events: 0, data: 0 }; 1];
        let start = std::time::Instant::now();
        assert_eq!(ep.wait(&mut buf, Some(50)).unwrap(), 0);
        assert!(start.elapsed() >= std::time::Duration::from_millis(45));
    }
}
