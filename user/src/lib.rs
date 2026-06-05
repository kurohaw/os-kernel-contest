#![no_std]

use core::panic::PanicInfo;

const SYS_TEST: usize = 0;
const SYS_EXIT: usize = 1;
const SYS_YIELD: usize = 2;
const SYS_OPENAT: usize = 56;
const SYS_CLOSE: usize = 57;
const SYS_READ: usize = 63;
const SYS_WRITE: usize = 64;
const SYS_FSTAT: usize = 80;
const SYS_GETPID: usize = 172;
const SYS_BRK: usize = 214;

pub const STAT_SIZE: usize = 128;
pub const AT_FDCWD: isize = -100;

fn syscall(id: usize, args: [usize; 4]) -> isize {
    let ret: isize;

    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("a0") args[0] as isize => ret,
            in("a1") args[1],
            in("a2") args[2],
            in("a3") args[3],
            in("a7") id,
        );
    }

    ret
}

pub fn sys_test(value: usize) -> isize {
    syscall(SYS_TEST, [value, 0, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYS_YIELD, [0, 0, 0, 0])
}

pub fn open(path: &[u8], flags: usize) -> isize {
    openat(AT_FDCWD, path, flags, 0)
}

pub fn openat(dirfd: isize, path: &[u8], flags: usize, mode: usize) -> isize {
    syscall(SYS_OPENAT, [dirfd as usize, path.as_ptr() as usize, flags, mode])
}

pub fn close(fd: usize) -> isize {
    syscall(SYS_CLOSE, [fd, 0, 0, 0])
}

pub fn fstat(fd: usize, buf: &mut [u8; STAT_SIZE]) -> isize {
    syscall(SYS_FSTAT, [fd, buf.as_mut_ptr() as usize, 0, 0])
}

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    syscall(SYS_READ, [fd, buf.as_mut_ptr() as usize, buf.len(), 0])
}

pub fn write(fd: usize, s: &str) -> isize {
    syscall(SYS_WRITE, [fd, s.as_ptr() as usize, s.len(), 0])
}

pub fn brk(addr: usize) -> isize {
    syscall(SYS_BRK, [addr, 0, 0, 0])
}

pub fn getpid() -> isize {
    syscall(SYS_GETPID, [0, 0, 0, 0])
}
pub fn sys_exit(code: i32) -> ! {
    syscall(SYS_EXIT, [code as usize, 0, 0, 0]);
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
