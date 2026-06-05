pub const STDIN: usize = 0;
pub const STDOUT: usize = 1;
pub const STDERR: usize = 2;

pub fn read(fd: usize, _buf: usize, _len: usize) -> isize {
    match fd {
        STDIN => 0,
        _ => -1,
    }
}

pub fn write(fd: usize, buf: usize, len: usize) -> isize {
    if fd != STDOUT && fd != STDERR {
        return -1;
    }

    let bytes = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };

    for &byte in bytes {
        crate::sbi::console_putchar(byte as usize);
    }

    len as isize
}