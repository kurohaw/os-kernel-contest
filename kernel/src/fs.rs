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
const STAT_INO_OFFSET: usize = 8;
const STAT_MODE_OFFSET: usize = 16;
const STAT_NLINK_OFFSET: usize = 20;
const STAT_SIZE_OFFSET: usize = 48;
const S_IFCHR: u32 = 0o020000;
const S_IFREG: u32 = 0o100000;
const STDIO_MODE: u32 = S_IFCHR | 0o666;
const REGULAR_MODE: u32 = S_IFREG | 0o444;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FileKind {
    DevNull,
    Hello,
    Ext4,
}

#[derive(Clone, Copy)]
struct FileDescriptor {
    kind: FileKind,
    offset: usize,
    inode_no: u32,
    size: u64,
}

static mut FD_TABLE: [Option<FileDescriptor>; MAX_FD] = [None; MAX_FD];

pub fn read(fd: usize, buf: usize, len: usize) -> isize {
    match fd {
        STDIN => 0,
        _ if descriptor_kind(fd) == Some(FileKind::DevNull) => 0,
        _ if descriptor_kind(fd) == Some(FileKind::Hello) => read_hello_file(fd, buf, len),
        _ if descriptor_kind(fd) == Some(FileKind::Ext4) => read_ext4_file(fd, buf, len),
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

    let (mode, inode_no, size) = match fd_stat_info(fd) {
        Some(info) => info,
        None => return -1,
    };

    let stat = unsafe { core::slice::from_raw_parts_mut(stat_buf as *mut u8, STAT_SIZE) };
    stat.fill(0);

    let inode_no = inode_no.to_le_bytes();
    stat[STAT_INO_OFFSET..STAT_INO_OFFSET + 8].copy_from_slice(&inode_no);

    let mode = mode.to_le_bytes();
    stat[STAT_MODE_OFFSET..STAT_MODE_OFFSET + 4].copy_from_slice(&mode);

    let nlink = 1u32.to_le_bytes();
    stat[STAT_NLINK_OFFSET..STAT_NLINK_OFFSET + 4].copy_from_slice(&nlink);

    let size = size.to_le_bytes();
    stat[STAT_SIZE_OFFSET..STAT_SIZE_OFFSET + 8].copy_from_slice(&size);

    0
}

pub fn openat(_dirfd: usize, path: usize, _flags: usize, _mode: usize) -> isize {
    if path == 0 {
        return -1;
    }

    let mut path_buffer = [0u8; MAX_PATH_LEN];
    let path_len = match copy_user_cstr(path, &mut path_buffer) {
        Some(len) => len,
        None => return -1,
    };
    let path_bytes = &path_buffer[..path_len];

    if path_bytes == DEV_NULL_PATH {
        alloc_fd(FileKind::DevNull, 0, 0).map_or(-1, |fd| fd as isize)
    } else if path_bytes == HELLO_PATH {
        alloc_fd(FileKind::Hello, 0, HELLO_CONTENT.len() as u64).map_or(-1, |fd| fd as isize)
    } else if let Some(file) = crate::drivers::ext4::open_path(path_bytes) {
        alloc_fd(FileKind::Ext4, file.inode_no, file.size).map_or(-1, |fd| fd as isize)
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
    descriptor(fd).map(|descriptor| descriptor.kind)
}

fn descriptor(fd: usize) -> Option<FileDescriptor> {
    if fd < FIRST_DYNAMIC_FD || fd >= MAX_FD {
        return None;
    }

    unsafe { FD_TABLE[fd] }
}

fn alloc_fd(kind: FileKind, inode_no: u32, size: u64) -> Option<usize> {
    let mut fd = FIRST_DYNAMIC_FD;

    while fd < MAX_FD {
        unsafe {
            if FD_TABLE[fd].is_none() {
                FD_TABLE[fd] = Some(FileDescriptor {
                    kind,
                    offset: 0,
                    inode_no,
                    size,
                });
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

fn fd_stat_info(fd: usize) -> Option<(u32, u64, u64)> {
    match fd {
        STDIN | STDOUT | STDERR => Some((STDIO_MODE, fd as u64, 0)),
        _ => {
            let descriptor = descriptor(fd)?;
            match descriptor.kind {
                FileKind::DevNull => Some((STDIO_MODE, descriptor.inode_no as u64, 0)),
                FileKind::Hello => Some((REGULAR_MODE, descriptor.inode_no as u64, descriptor.size)),
                FileKind::Ext4 => Some((
                    REGULAR_MODE,
                    descriptor.inode_no as u64,
                    descriptor.size,
                )),
            }
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
            inode_no: descriptor.inode_no,
            size: descriptor.size,
        });

        copy_len as isize
    }
}

fn read_ext4_file(fd: usize, buf: usize, len: usize) -> isize {
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

        let output = core::slice::from_raw_parts_mut(buf as *mut u8, len);
        let file = crate::drivers::ext4::Ext4File {
            inode_no: descriptor.inode_no,
            size: descriptor.size,
        };

        let read_len = match crate::drivers::ext4::read_file_at(file, descriptor.offset, output) {
            Ok(read_len) => read_len,
            Err(_) => return -1,
        };

        FD_TABLE[fd] = Some(FileDescriptor {
            kind: descriptor.kind,
            offset: descriptor.offset + read_len,
            inode_no: descriptor.inode_no,
            size: descriptor.size,
        });

        read_len as isize
    }
}

fn copy_user_cstr(ptr: usize, output: &mut [u8]) -> Option<usize> {
    let mut i = 0;

    while i < output.len() {
        let byte = unsafe { (ptr as *const u8).add(i).read() };

        if byte == 0 {
            return Some(i);
        }

        output[i] = byte;
        i += 1;
    }

    None
}
