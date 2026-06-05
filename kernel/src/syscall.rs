pub const SYS_TEST: usize = 0;
pub const SYS_EXIT: usize = 1;
pub const SYS_YIELD: usize = 2;
pub const SYS_OPENAT: usize = 56;
pub const SYS_CLOSE: usize = 57;
pub const SYS_READ: usize = 63;
pub const SYS_WRITE: usize = 64;
pub const SYS_FSTAT: usize = 80;
pub const SYS_GETPID: usize = 172;
pub const SYS_BRK: usize = 214;

pub fn syscall(id: usize, args: [usize; 4]) -> isize {
    match id {
        SYS_TEST => sys_test(args[0]),
        SYS_EXIT => sys_exit(args[0] as i32),
        SYS_YIELD => sys_yield(),
        SYS_OPENAT => sys_openat(args[0], args[1], args[2], args[3]),
        SYS_CLOSE => sys_close(args[0]),
        SYS_READ => sys_read(args[0], args[1], args[2]),
        SYS_WRITE => sys_write(args[0], args[1], args[2]),
        SYS_FSTAT => sys_fstat(args[0], args[1]),
        SYS_GETPID => sys_getpid(),
        SYS_BRK => sys_brk(args[0]),
        _ => {
            crate::println!("unsupported syscall: id={}", id);
            -1
        }
    }
}

fn sys_test(value: usize) -> isize {
    crate::println!("sys_test called, arg0={}", value);
    (value + 1) as isize
}

fn sys_exit(code: i32) -> isize {
    crate::task::exit_current(code);
}

fn sys_yield() -> isize {
    0
}

fn sys_openat(dirfd: usize, path: usize, flags: usize, mode: usize) -> isize {
    crate::fs::openat(dirfd, path, flags, mode)
}

fn sys_close(fd: usize) -> isize {
    crate::fs::close(fd)
}

fn sys_fstat(fd: usize, stat_buf: usize) -> isize {
    crate::fs::fstat(fd, stat_buf)
}

fn sys_read(fd: usize, buf: usize, len: usize) -> isize {
    crate::fs::read(fd, buf, len)
}

fn sys_write(fd: usize, buf: usize, len: usize) -> isize {
    crate::fs::write(fd, buf, len)
} 

fn sys_getpid() -> isize {
    crate::task::current_task_id() as isize
}

fn sys_brk(addr: usize) -> isize {
    crate::task::set_current_brk(addr) as isize
}
