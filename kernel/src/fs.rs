pub const STDIN: usize = 0;
pub const STDOUT: usize = 1;
pub const STDERR: usize = 2;
pub const STAT_SIZE: usize = 128;

const FIRST_DYNAMIC_FD: usize = 3;
const MAX_FD: usize = 16;
const MAX_PATH_LEN: usize = 128;
const DEV_NULL_PATH: &[u8] = b"/dev/null";
const STAT_MODE_OFFSET: usize = 16;
const S_IFCHR: u32 = 0o020000;
const STDIO_MODE: u32 = S_IFCHR | 0o666;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FileKind {
    DevNull,
}

static mut FD_TABLE: [Option<FileKind>; MAX_FD] = [None; MAX_FD];

pub fn read(fd: usize, _buf: usize, _len: usize) -> isize {
    match fd {
        STDIN => 0,
        _ if dynamic_file_kind(fd) == Some(FileKind::DevNull) => 0,
        _ => -1,
    }
}

pub fn write(fd: usize, buf: usize, len: usize) -> isize {
    match fd {
        STDOUT | STDERR => {
            let bytes = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };
            for &byte in bytes {
                crate::sbi::console_putchar(byte as usize);
            }
            len as isize
        }
        _ if dynamic_file_kind(fd) == Some(FileKind::DevNull) => len as isize,
        _ => -1,
    }
}

pub fn close(fd: usize) -> isize {
    match fd {
        STDIN | STDOUT | STDERR => 0,
        _ => close_dynamic_fd(fd),
    }
}

pub fn fstat(fd: usize, stat_buf: usize) -> isize {
    if !is_valid_fd(fd) {
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
        alloc_fd(FileKind::DevNull).map_or(-1, |fd| fd as isize)
    } else {
        -1
    }
}

fn is_valid_fd(fd: usize) -> bool {
    match fd {
        STDIN | STDOUT | STDERR => true,
        _ => dynamic_file_kind(fd).is_some(),
    }
}

fn dynamic_file_kind(fd: usize) -> Option<FileKind> {
    if fd < FIRST_DYNAMIC_FD || fd >= MAX_FD {
        return None;
    }

    unsafe { FD_TABLE[fd] }
}

fn alloc_fd(kind: FileKind) -> Option<usize> {
    let mut fd = FIRST_DYNAMIC_FD;

    while fd < MAX_FD {
        unsafe {
            if FD_TABLE[fd].is_none() {
                FD_TABLE[fd] = Some(kind);
                return Some(fd);
            }
        }

        fd += 1;
    }

    None
}

fn close_dynamic_fd(fd: usize) -> isize {
    if fd < FIRST_DYNAMIC_FD || fd >= MAX_FD {
        return -1;
    }

    unsafe {
        if FD_TABLE[fd].is_some() {
            FD_TABLE[fd] = None;
            0
        } else {
            -1
        }
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
