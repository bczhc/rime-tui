use std::io;
use std::io::Read;
use std::os::fd::RawFd;
use std::os::raw::{c_int, c_void};

use libc::size_t;

pub struct FdReader {
    fd: RawFd,
}

impl FdReader {
    pub fn new(fd: RawFd) -> Self {
        Self { fd }
    }
}

impl Read for FdReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let size = libc::read(
                self.fd as c_int,
                buf.as_mut_ptr() as *mut c_void,
                buf.len() as size_t,
            );
            if size == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(size as usize)
        }
    }
}
