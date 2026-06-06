pub const STDIN: usize = 0;
pub const STDOUT: usize = 1;
pub const STDERR: usize = 2;
pub const STAT_SIZE: usize = 128;

const FIRST_DYNAMIC_FD: usize = 3;
const MAX_FD: usize = 128;
const MAX_PATH_LEN: usize = 128;
const MEMORY_FILE_CAP: usize = 1024;
const DEV_NULL_PATH: &[u8] = b"/dev/null";
const HELLO_PATH: &[u8] = b"/hello.txt";
const HELLO_CONTENT: &[u8] = b"hello from kernel\n";
const O_CREATE: usize = 0x40;
const O_DIRECTORY: usize = 0x10000;
const STAT_INO_OFFSET: usize = 8;
const STAT_MODE_OFFSET: usize = 16;
const STAT_NLINK_OFFSET: usize = 20;
const STAT_SIZE_OFFSET: usize = 48;
const S_IFCHR: u32 = 0o020000;
const S_IFDIR: u32 = 0o040000;
const S_IFREG: u32 = 0o100000;
const STDIO_MODE: u32 = S_IFCHR | 0o666;
const REGULAR_MODE: u32 = S_IFREG | 0o444;
const DIRECTORY_MODE: u32 = S_IFDIR | 0o555;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FileKind {
    Console,
    DevNull,
    Directory,
    Hello,
    Ext4,
    Memory,
    PipeRead,
    PipeWrite,
}

#[derive(Clone, Copy)]
struct FileDescriptor {
    kind: FileKind,
    offset: usize,
    inode_no: u32,
    size: u64,
}

static mut FD_TABLE: [Option<FileDescriptor>; MAX_FD] = [None; MAX_FD];
static mut MEMORY_FILES: [[u8; MEMORY_FILE_CAP]; MAX_FD] = [[0; MEMORY_FILE_CAP]; MAX_FD];
static mut CURRENT_CWD: [u8; MAX_PATH_LEN] = [0; MAX_PATH_LEN];
static mut CURRENT_CWD_LEN: usize = 0;
static mut PIPE_BUFFER: [u8; MEMORY_FILE_CAP] = [0; MEMORY_FILE_CAP];
static mut PIPE_LEN: usize = 0;
static mut PIPE_READ_OFFSET: usize = 0;

pub fn read(fd: usize, buf: usize, len: usize) -> isize {
    match fd {
        STDIN => 0,
        _ if descriptor_kind(fd) == Some(FileKind::DevNull) => 0,
        _ if descriptor_kind(fd) == Some(FileKind::Hello) => read_hello_file(fd, buf, len),
        _ if descriptor_kind(fd) == Some(FileKind::Ext4) => read_ext4_file(fd, buf, len),
        _ if descriptor_kind(fd) == Some(FileKind::Memory) => read_memory_file(fd, buf, len),
        _ if descriptor_kind(fd) == Some(FileKind::PipeRead) => read_pipe(buf, len),
        _ => -1,
    }
}

pub fn write(fd: usize, buf: usize, len: usize) -> isize {
    match fd {
        STDOUT | STDERR => write_console(buf, len),
        _ if descriptor_kind(fd) == Some(FileKind::Console) => write_console(buf, len),
        _ if descriptor_kind(fd) == Some(FileKind::DevNull) => len as isize,
        _ if descriptor_kind(fd) == Some(FileKind::Memory) => write_memory_file(fd, buf, len),
        _ if descriptor_kind(fd) == Some(FileKind::PipeWrite) => write_pipe(buf, len),
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

pub fn openat(_dirfd: usize, path: usize, flags: usize, _mode: usize) -> isize {
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
    } else if flags & O_DIRECTORY != 0 || is_dot_path(path_bytes) {
        alloc_fd(FileKind::Directory, 0, 0).map_or(-1, |fd| fd as isize)
    } else if path_bytes == HELLO_PATH {
        alloc_fd(FileKind::Hello, 0, HELLO_CONTENT.len() as u64).map_or(-1, |fd| fd as isize)
    } else if let Some(file) = open_ext4_relative(path_bytes) {
        alloc_fd(FileKind::Ext4, file.inode_no, file.size).map_or(-1, |fd| fd as isize)
    } else if flags & O_CREATE != 0 {
        alloc_fd(FileKind::Memory, 0, 0).map_or(-1, |fd| fd as isize)
    } else {
        -1
    }
}

pub fn reset_cwd_from_loader() {
    let cwd = crate::loader::external_cwd();
    unsafe {
        CURRENT_CWD.fill(0);
        let len = core::cmp::min(cwd.len(), MAX_PATH_LEN);
        CURRENT_CWD[..len].copy_from_slice(&cwd[..len]);
        CURRENT_CWD_LEN = len;
    }
}

pub fn getcwd(buf: usize, size: usize) -> isize {
    if buf == 0 || size == 0 {
        return -1;
    }

    let cwd = current_cwd();
    let prefix_len = 1usize;
    let needed = prefix_len + cwd.len() + 1;
    if needed > size {
        return -1;
    }

    unsafe {
        let output = core::slice::from_raw_parts_mut(buf as *mut u8, size);
        output.fill(0);
        output[0] = b'/';
        output[prefix_len..prefix_len + cwd.len()].copy_from_slice(cwd);
    }

    buf as isize
}

pub fn chdir(path: usize) -> isize {
    if path == 0 {
        return -1;
    }

    let mut path_buffer = [0u8; MAX_PATH_LEN];
    let path_len = match copy_user_cstr(path, &mut path_buffer) {
        Some(len) => len,
        None => return -1,
    };
    let mut resolved = [0u8; MAX_PATH_LEN];
    let resolved_len = resolve_relative_path(&path_buffer[..path_len], &mut resolved);

    unsafe {
        CURRENT_CWD.fill(0);
        CURRENT_CWD[..resolved_len].copy_from_slice(&resolved[..resolved_len]);
        CURRENT_CWD_LEN = resolved_len;
    }

    0
}

pub fn dup(fd: usize) -> isize {
    let descriptor = descriptor_for_dup(fd);
    match descriptor {
        Some(descriptor) => alloc_descriptor(descriptor).map_or(-1, |new_fd| new_fd as isize),
        None => -1,
    }
}

pub fn dup_to(old_fd: usize, new_fd: usize) -> isize {
    if new_fd >= MAX_FD {
        return -1;
    }

    let descriptor = match descriptor_for_dup(old_fd) {
        Some(descriptor) => descriptor,
        None => return -1,
    };

    unsafe {
        FD_TABLE[new_fd] = Some(descriptor);
    }
    new_fd as isize
}

pub fn pipe2(pipe: usize) -> isize {
    if pipe == 0 {
        return -1;
    }

    unsafe {
        PIPE_LEN = 0;
        PIPE_READ_OFFSET = 0;
    }

    let read_fd = match alloc_fd(FileKind::PipeRead, 0, 0) {
        Some(fd) => fd,
        None => return -1,
    };
    let write_fd = match alloc_fd(FileKind::PipeWrite, 0, 0) {
        Some(fd) => fd,
        None => return -1,
    };

    unsafe {
        (pipe as *mut i32).write(read_fd as i32);
        ((pipe + core::mem::size_of::<i32>()) as *mut i32).write(write_fd as i32);
    }

    0
}

pub fn getdents64(_fd: usize, buf: usize, len: usize) -> isize {
    if buf == 0 || len < 32 {
        return -1;
    }

    let name = b".\0";
    let reclen = 24 + name.len();
    if len < reclen {
        return -1;
    }

    unsafe {
        let output = core::slice::from_raw_parts_mut(buf as *mut u8, reclen);
        output.fill(0);
        (buf as *mut u64).write(1);
        ((buf + 8) as *mut i64).write(reclen as i64);
        ((buf + 16) as *mut u16).write(reclen as u16);
        ((buf + 18) as *mut u8).write(4);
        core::ptr::copy_nonoverlapping(name.as_ptr(), (buf + 19) as *mut u8, name.len());
    }

    reclen as isize
}

pub fn read_at(fd: usize, offset: usize, buf: usize, len: usize) -> isize {
    match descriptor_kind(fd) {
        Some(FileKind::Memory) => read_memory_at(fd, offset, buf, len),
        Some(FileKind::Hello) => copy_slice_at(HELLO_CONTENT, offset, buf, len),
        Some(FileKind::Ext4) => read_ext4_at(fd, offset, buf, len),
        _ => 0,
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

fn alloc_descriptor(descriptor: FileDescriptor) -> Option<usize> {
    let mut fd = FIRST_DYNAMIC_FD;

    while fd < MAX_FD {
        unsafe {
            if FD_TABLE[fd].is_none() {
                FD_TABLE[fd] = Some(descriptor);
                return Some(fd);
            }
        }

        fd += 1;
    }

    None
}

fn descriptor_for_dup(fd: usize) -> Option<FileDescriptor> {
    match fd {
        STDOUT | STDERR => Some(FileDescriptor {
            kind: FileKind::Console,
            offset: 0,
            inode_no: fd as u32,
            size: 0,
        }),
        STDIN => Some(FileDescriptor {
            kind: FileKind::DevNull,
            offset: 0,
            inode_no: fd as u32,
            size: 0,
        }),
        _ => descriptor(fd),
    }
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
                FileKind::Console => Some((STDIO_MODE, descriptor.inode_no as u64, 0)),
                FileKind::DevNull => Some((STDIO_MODE, descriptor.inode_no as u64, 0)),
                FileKind::Directory => Some((DIRECTORY_MODE, descriptor.inode_no as u64, 0)),
                FileKind::Hello => Some((REGULAR_MODE, descriptor.inode_no as u64, descriptor.size)),
                FileKind::Ext4 => Some((
                    REGULAR_MODE,
                    descriptor.inode_no as u64,
                    descriptor.size,
                )),
                FileKind::Memory => Some((REGULAR_MODE, descriptor.inode_no as u64, descriptor.size)),
                FileKind::PipeRead | FileKind::PipeWrite => {
                    Some((STDIO_MODE, descriptor.inode_no as u64, descriptor.size))
                }
            }
        }
    }
}

fn write_console(buf: usize, len: usize) -> isize {
    let bytes = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };
    for &byte in bytes {
        crate::sbi::console_putchar(byte as usize);
    }
    len as isize
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

fn read_ext4_at(fd: usize, offset: usize, buf: usize, len: usize) -> isize {
    if buf == 0 || len == 0 {
        return 0;
    }

    let descriptor = match descriptor(fd) {
        Some(descriptor) => descriptor,
        None => return -1,
    };
    let output = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len) };
    let file = crate::drivers::ext4::Ext4File {
        inode_no: descriptor.inode_no,
        size: descriptor.size,
    };

    crate::drivers::ext4::read_file_at(file, offset, output).map_or(-1, |read_len| read_len as isize)
}

fn write_memory_file(fd: usize, buf: usize, len: usize) -> isize {
    if buf == 0 {
        return -1;
    }

    let descriptor = match descriptor(fd) {
        Some(descriptor) => descriptor,
        None => return -1,
    };
    let offset = descriptor.offset;
    let copy_len = core::cmp::min(len, MEMORY_FILE_CAP.saturating_sub(offset));
    unsafe {
        core::ptr::copy_nonoverlapping(buf as *const u8, MEMORY_FILES[fd].as_mut_ptr().add(offset), copy_len);
        FD_TABLE[fd] = Some(FileDescriptor {
            kind: descriptor.kind,
            offset: offset + copy_len,
            inode_no: descriptor.inode_no,
            size: core::cmp::max(descriptor.size as usize, offset + copy_len) as u64,
        });
    }
    copy_len as isize
}

fn read_memory_file(fd: usize, buf: usize, len: usize) -> isize {
    let descriptor = match descriptor(fd) {
        Some(descriptor) => descriptor,
        None => return -1,
    };
    let read_len = read_memory_at(fd, descriptor.offset, buf, len);
    if read_len > 0 {
        unsafe {
            FD_TABLE[fd] = Some(FileDescriptor {
                kind: descriptor.kind,
                offset: descriptor.offset + read_len as usize,
                inode_no: descriptor.inode_no,
                size: descriptor.size,
            });
        }
    }
    read_len
}

fn read_memory_at(fd: usize, offset: usize, buf: usize, len: usize) -> isize {
    if buf == 0 || len == 0 {
        return 0;
    }

    let descriptor = match descriptor(fd) {
        Some(descriptor) => descriptor,
        None => return -1,
    };
    let size = descriptor.size as usize;
    if offset >= size {
        return 0;
    }

    let copy_len = core::cmp::min(len, size - offset);
    unsafe {
        core::ptr::copy_nonoverlapping(MEMORY_FILES[fd].as_ptr().add(offset), buf as *mut u8, copy_len);
    }
    copy_len as isize
}

fn write_pipe(buf: usize, len: usize) -> isize {
    if buf == 0 {
        return -1;
    }

    let copy_len = core::cmp::min(len, MEMORY_FILE_CAP);
    unsafe {
        core::ptr::copy_nonoverlapping(buf as *const u8, PIPE_BUFFER.as_mut_ptr(), copy_len);
        PIPE_LEN = copy_len;
        PIPE_READ_OFFSET = 0;
    }
    copy_len as isize
}

fn read_pipe(buf: usize, len: usize) -> isize {
    if buf == 0 || len == 0 {
        return 0;
    }

    unsafe {
        if PIPE_READ_OFFSET >= PIPE_LEN {
            return 0;
        }
        let copy_len = core::cmp::min(len, PIPE_LEN - PIPE_READ_OFFSET);
        core::ptr::copy_nonoverlapping(PIPE_BUFFER.as_ptr().add(PIPE_READ_OFFSET), buf as *mut u8, copy_len);
        PIPE_READ_OFFSET += copy_len;
        copy_len as isize
    }
}

fn copy_slice_at(data: &[u8], offset: usize, buf: usize, len: usize) -> isize {
    if buf == 0 || len == 0 || offset >= data.len() {
        return 0;
    }

    let copy_len = core::cmp::min(len, data.len() - offset);
    unsafe {
        core::ptr::copy_nonoverlapping(data.as_ptr().add(offset), buf as *mut u8, copy_len);
    }
    copy_len as isize
}

fn open_ext4_relative(path: &[u8]) -> Option<crate::drivers::ext4::Ext4File> {
    if let Some(file) = crate::drivers::ext4::open_path(path) {
        return Some(file);
    }

    if path.starts_with(b"/") {
        return None;
    }

    let cwd = current_cwd();
    if cwd.is_empty() {
        return None;
    }

    let mut full = [0u8; MAX_PATH_LEN];
    let mut len = core::cmp::min(cwd.len(), MAX_PATH_LEN);
    full[..len].copy_from_slice(&cwd[..len]);
    if len < MAX_PATH_LEN {
        full[len] = b'/';
        len += 1;
    }
    let copy_len = core::cmp::min(path.len(), MAX_PATH_LEN - len);
    full[len..len + copy_len].copy_from_slice(&path[..copy_len]);
    crate::drivers::ext4::open_path(&full[..len + copy_len])
}

fn current_cwd() -> &'static [u8] {
    let len = unsafe { CURRENT_CWD_LEN };
    unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(CURRENT_CWD) as *const u8, len) }
}

fn resolve_relative_path(path: &[u8], output: &mut [u8; MAX_PATH_LEN]) -> usize {
    let mut source = path;
    while source.starts_with(b"./") {
        source = &source[2..];
    }

    if source.starts_with(b"/") {
        source = &source[1..];
    }

    let copy_len = core::cmp::min(source.len(), MAX_PATH_LEN);
    output[..copy_len].copy_from_slice(&source[..copy_len]);
    copy_len
}

fn is_dot_path(path: &[u8]) -> bool {
    path == b"." || path == b"./" || path == b"./mnt" || path == b"mnt" || path.ends_with(b"/mnt")
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
