pub const STDIN: usize = 0;
pub const STDOUT: usize = 1;
pub const STDERR: usize = 2;
pub const DEV_NULL_FD: usize = 3;
pub const STAT_SIZE: usize = 128;

const MAX_PATH_LEN: usize = 128;
const DEV_NULL_PATH: &[u8] = b"/dev/null";
const STAT_MODE_OFFSET: usize = 16;
const S_IFCHR: u32 = 0o020000;
const STDIO_MODE: u32 = S_IFCHR | 0o666;

pub fn read(fd: usize, _buf: usize, _len: usize) -> isize {
    match fd {
        STDIN | DEV_NULL_FD => 0,
        _ => -1,
    }
}

pub fn write(fd: usize, buf: usize, len: usize) -> isize {
    if fd == DEV_NULL_FD {
        return len as isize;
    }

    if fd != STDOUT && fd != STDERR {
        return -1;
    }

    let bytes = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };

    for &byte in bytes {
        crate::sbi::console_putchar(byte as usize);
    }

    len as isize
}

pub fn close(fd: usize) -> isize {
    match fd {
        STDIN | STDOUT | STDERR | DEV_NULL_FD => 0,
        _ => -1,
    }
}

pub fn fstat(fd: usize, stat_buf: usize) -> isize {
    if fd != STDIN && fd != STDOUT && fd != STDERR && fd != DEV_NULL_FD {
        return -1;
    }

    if stat_buf == 0 {
        return -1;
    }

    let stat = unsafe { core::slice::from_raw_parts_mut(stat_buf as *mut u8, STAT_SIZE) };
    stat.fill(0);

    let mode = STDIO_MODE.to_le_bytes();
    stat[STAT_MODE_OFFSET..STAT_MODE_OFFSET + 4].copy_from_slice(&mode);

    0
}

pub fn openat(_dirfd: usize, path: usize, _flags: usize, _mode: usize) -> isize {
    if path == 0 {
        return -1;
    }

    if user_cstr_eq(path, DEV_NULL_PATH) {
        DEV_NULL_FD as isize
    } else {
        -1
    }
}

fn user_cstr_eq(ptr: usize, expected: &[u8]) -> bool {
    let mut i = 0;

    while i < MAX_PATH_LEN {
        let byte = unsafe { (ptr as *const u8).add(i).read() };

        if byte == 0 {
            return i == expected.len();
        }

        if i >= expected.len() || byte != expected[i] {
            return false;
        }

        i += 1;
    }

    false
}
