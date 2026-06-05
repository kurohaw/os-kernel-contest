pub const STDIN: usize = 0;
pub const STDOUT: usize = 1;
pub const STDERR: usize = 2;
pub const STAT_SIZE: usize = 128;

const FIRST_DYNAMIC_FD: usize = 3;
const MAX_FD: usize = 16;
const MAX_PATH_LEN: usize = 128;
const DEV_NULL_PATH: &[u8] = b"/dev/null";
const HELLO_PATH: &[u8] = b"/hello.txt";
const HELLO_CONTENT: &[u8] = b"hello from kernel\n";
const STAT_MODE_OFFSET: usize = 16;
const S_IFCHR: u32 = 0o020000;
const STDIO_MODE: u32 = S_IFCHR | 0o666;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FileKind {
    DevNull,
    Hello,
}

#[derive(Clone, Copy)]
struct FileDescriptor {
    kind: FileKind,
    offset: usize,
}

static mut FD_TABLE: [Option<FileDescriptor>; MAX_FD] = [None; MAX_FD];

pub fn read(fd: usize, buf: usize, len: usize) -> isize {
    match fd {
        STDIN => 0,
        _ if descriptor_kind(fd) == Some(FileKind::DevNull) => 0,
        _ if descriptor_kind(fd) == Some(FileKind::Hello) => read_hello_file(fd, buf, len),
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
        _ if descriptor_kind(fd) == Some(FileKind::DevNull) => len as isize,
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
    } else if user_cstr_eq(path, HELLO_PATH) {
        alloc_fd(FileKind::Hello).map_or(-1, |fd| fd as isize)
    } else {
        -1
    }
}

fn is_valid_fd(fd: usize) -> bool {
    match fd {
        STDIN | STDOUT | STDERR => true,
        _ => descriptor_kind(fd).is_some(),
    }
}

fn descriptor_kind(fd: usize) -> Option<FileKind> {
    if fd < FIRST_DYNAMIC_FD || fd >= MAX_FD {
        return None;
    }

    unsafe { FD_TABLE[fd].map(|descriptor| descriptor.kind) }
}

fn alloc_fd(kind: FileKind) -> Option<usize> {
    let mut fd = FIRST_DYNAMIC_FD;

    while fd < MAX_FD {
        unsafe {
            if FD_TABLE[fd].is_none() {
                FD_TABLE[fd] = Some(FileDescriptor { kind, offset: 0 });
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

fn read_hello_file(fd: usize, buf: usize, len: usize) -> isize {
    if buf == 0 {
        return -1;
    }

    if len == 0 {
        return 0;
    }

    unsafe {
        let descriptor = match FD_TABLE[fd] {
            Some(descriptor) => descriptor,
            None => return -1,
        };

        let offset = descriptor.offset;
        if offset >= HELLO_CONTENT.len() {
            return 0;
        }

        let remaining = HELLO_CONTENT.len() - offset;
        let copy_len = if len < remaining { len } else { remaining };

        core::ptr::copy_nonoverlapping(
            HELLO_CONTENT.as_ptr().add(offset),
            buf as *mut u8,
            copy_len,
        );

        FD_TABLE[fd] = Some(FileDescriptor {
            kind: descriptor.kind,
            offset: offset + copy_len,
        });

        copy_len as isize
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
