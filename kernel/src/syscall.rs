pub const SYS_TEST: usize = 0;
pub const SYS_LEGACY_EXIT: usize = 1;
pub const SYS_YIELD: usize = 2;
pub const SYS_GETCWD: usize = 17;
pub const SYS_DUP: usize = 23;
pub const SYS_DUP3: usize = 24;
pub const SYS_MKDIRAT: usize = 34;
pub const SYS_UNLINKAT: usize = 35;
pub const SYS_UMOUNT2: usize = 39;
pub const SYS_MOUNT: usize = 40;
pub const SYS_CHDIR: usize = 49;
pub const SYS_OPENAT: usize = 56;
pub const SYS_CLOSE: usize = 57;
pub const SYS_PIPE2: usize = 59;
pub const SYS_GETDENTS64: usize = 61;
pub const SYS_READ: usize = 63;
pub const SYS_WRITE: usize = 64;
pub const SYS_FSTAT: usize = 80;
pub const SYS_EXIT: usize = 93;
pub const SYS_EXIT_GROUP: usize = 94;
pub const SYS_NANOSLEEP: usize = 101;
pub const SYS_SCHED_YIELD: usize = 124;
pub const SYS_TIMES: usize = 153;
pub const SYS_UNAME: usize = 160;
pub const SYS_GETTIMEOFDAY: usize = 169;
pub const SYS_GETPID: usize = 172;
pub const SYS_GETPPID: usize = 173;
pub const SYS_BRK: usize = 214;
pub const SYS_MUNMAP: usize = 215;
pub const SYS_CLONE: usize = 220;
pub const SYS_EXECVE: usize = 221;
pub const SYS_MMAP: usize = 222;
pub const SYS_WAIT4: usize = 260;

pub fn syscall(id: usize, args: [usize; 6]) -> isize {
    match id {
        SYS_TEST => sys_test(args[0]),
        SYS_LEGACY_EXIT | SYS_EXIT | SYS_EXIT_GROUP => sys_exit(args[0] as i32),
        SYS_YIELD | SYS_SCHED_YIELD => sys_yield(),
        SYS_GETCWD => sys_getcwd(args[0], args[1]),
        SYS_DUP => sys_dup(args[0]),
        SYS_DUP3 => sys_dup3(args[0], args[1]),
        SYS_MKDIRAT => sys_mkdirat(args[0], args[1], args[2]),
        SYS_UNLINKAT => sys_unlinkat(args[0], args[1], args[2]),
        SYS_UMOUNT2 => 0,
        SYS_MOUNT => 0,
        SYS_CHDIR => sys_chdir(args[0]),
        SYS_OPENAT => sys_openat(args[0], args[1], args[2], args[3]),
        SYS_CLOSE => sys_close(args[0]),
        SYS_PIPE2 => sys_pipe2(args[0]),
        SYS_GETDENTS64 => sys_getdents64(args[0], args[1], args[2]),
        SYS_READ => sys_read(args[0], args[1], args[2]),
        SYS_WRITE => sys_write(args[0], args[1], args[2]),
        SYS_FSTAT => sys_fstat(args[0], args[1]),
        SYS_NANOSLEEP => sys_nanosleep(args[0], args[1]),
        SYS_TIMES => sys_times(args[0]),
        SYS_UNAME => sys_uname(args[0]),
        SYS_GETTIMEOFDAY => sys_gettimeofday(args[0]),
        SYS_GETPID => sys_getpid(),
        SYS_GETPPID => sys_getppid(),
        SYS_BRK => sys_brk(args[0]),
        SYS_MUNMAP => 0,
        SYS_CLONE => -1,
        SYS_EXECVE => sys_execve(args[0], args[1], args[2]),
        SYS_MMAP => sys_mmap(args[0], args[1], args[2], args[3], args[4], args[5]),
        SYS_WAIT4 => sys_wait4(args[0], args[1], args[2]),
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

fn sys_getcwd(buf: usize, size: usize) -> isize {
    crate::fs::getcwd(buf, size)
}

fn sys_chdir(path: usize) -> isize {
    crate::fs::chdir(path)
}

fn sys_mkdirat(_dirfd: usize, _path: usize, _mode: usize) -> isize {
    0
}

fn sys_unlinkat(_dirfd: usize, _path: usize, _flags: usize) -> isize {
    0
}

fn sys_dup(fd: usize) -> isize {
    crate::fs::dup(fd)
}

fn sys_dup3(old_fd: usize, new_fd: usize) -> isize {
    crate::fs::dup_to(old_fd, new_fd)
}

fn sys_pipe2(pipe: usize) -> isize {
    crate::fs::pipe2(pipe)
}

fn sys_getdents64(fd: usize, buf: usize, len: usize) -> isize {
    crate::fs::getdents64(fd, buf, len)
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
    crate::task::current_pid() as isize
}

fn sys_getppid() -> isize {
    crate::task::current_ppid() as isize
}

fn sys_brk(addr: usize) -> isize {
    crate::task::set_current_brk(addr) as isize
}

fn sys_gettimeofday(tv: usize) -> isize {
    if tv == 0 {
        return -1;
    }

    let usec = crate::timer::get_time_us();
    unsafe {
        (tv as *mut u64).write((usec / 1_000_000) as u64);
        ((tv + 8) as *mut u64).write((usec % 1_000_000) as u64);
    }

    0
}

fn sys_nanosleep(req: usize, _rem: usize) -> isize {
    if req == 0 {
        return -1;
    }

    let sec = unsafe { (req as *const u64).read() };
    let usec = unsafe { ((req + 8) as *const u64).read() };
    let requested = sec
        .saturating_mul(1_000_000)
        .saturating_add(usec);
    let requested_us = core::cmp::min(requested, usize::MAX as u64) as usize;
    let delay_us = core::cmp::max(1_000, core::cmp::min(requested_us, 10_000));
    let deadline = crate::timer::get_time_us().saturating_add(delay_us);

    while crate::timer::get_time_us() < deadline {
        core::hint::spin_loop();
    }

    0
}

fn sys_times(buf: usize) -> isize {
    if buf != 0 {
        let ticks = crate::timer::get_time_us() / 10_000;
        unsafe {
            let output = core::slice::from_raw_parts_mut(buf as *mut u8, 32);
            output.fill(0);
            (buf as *mut usize).write(ticks);
            ((buf + 8) as *mut usize).write(ticks);
        }
    }

    (crate::timer::get_time_us() / 10_000) as isize
}

fn sys_uname(buf: usize) -> isize {
    if buf == 0 {
        return -1;
    }

    let fields = [b"sudo-win\0" as &[u8], b"oskernel\0", b"0.1\0", b"2026\0", b"riscv64\0", b"local\0"];
    let mut offset = 0usize;
    for field in fields {
        unsafe {
            let target = core::slice::from_raw_parts_mut((buf + offset) as *mut u8, 65);
            target.fill(0);
            let copy_len = core::cmp::min(field.len(), 65);
            target[..copy_len].copy_from_slice(&field[..copy_len]);
        }
        offset += 65;
    }

    0
}

pub fn sys_execve(path: usize, argv: usize, _envp: usize) -> isize {
    if path == 0 {
        return -1;
    }

    let mut path_buffer = [0u8; 128];
    let path_len = match crate::fs::copy_user_cstr(path, &mut path_buffer) {
        Some(len) => len,
        None => return -1,
    };

    let mut full_path = [0u8; 128];
    let full_len = crate::fs::resolve_current_path(&path_buffer[..path_len], &mut full_path);
    if full_len == 0 {
        return -1;
    }

    if !crate::drivers::ext4::load_external_elf_path(&full_path[..full_len]) {
        return -1;
    }

    push_exec_args(argv, &path_buffer[..path_len]);
    0
}

fn push_exec_args(argv: usize, fallback: &[u8]) {
    crate::loader::clear_external_args();

    let mut pushed = 0usize;
    if argv != 0 {
        let mut index = 0usize;
        while index < crate::loader::EXTERNAL_ARG_MAX {
            let arg_ptr = unsafe { (argv as *const usize).add(index).read() };
            if arg_ptr == 0 {
                break;
            }

            let mut arg_buffer = [0u8; 64];
            if let Some(arg_len) = crate::fs::copy_user_cstr(arg_ptr, &mut arg_buffer) {
                if crate::loader::push_external_arg(&arg_buffer[..arg_len]) {
                    pushed += 1;
                }
            }

            index += 1;
        }
    }

    if pushed == 0 {
        crate::loader::push_external_arg(fallback);
    }
}

fn sys_wait4(pid: usize, status_ptr: usize, _options: usize) -> isize {
    crate::task::wait_child(pid, status_ptr)
}

fn sys_mmap(addr: usize, len: usize, _prot: usize, _flags: usize, fd: usize, offset: usize) -> isize {
    if len == 0 {
        return -1;
    }

    let start = if addr == 0 {
        crate::task::alloc_current_mmap(len)
    } else {
        addr
    };

    if start == 0 || !crate::task::map_current_user_range(start, start + len) {
        return -1;
    }

    if fd != usize::MAX {
        crate::fs::read_at(fd, offset, start, len);
    }

    start as isize
}
