#![no_std]
#![no_main]

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    user::write(1, "app0: hello from write\n");
    user::sys_test(100);
    if user::getpid() == 1 {
        user::write(1, "app0: getpid ok\n");
    } else {
        user::write(1, "app0: getpid wrong\n");
    }

    let mut buf = [0u8; 8];
    if user::read(0, &mut buf) == 0 {
        user::write(1, "app0: read eof ok\n");
    } else {
        user::write(1, "app0: read wrong\n");
    }

    let brk0 = user::brk(0);
    if brk0 > 0 {
        user::write(1, "app0: brk query ok\n");
    } else {
        user::write(1, "app0: brk query wrong\n");
    }

    let brk1 = brk0 as usize + 4096;
    if user::brk(brk1) == brk1 as isize {
        user::write(1, "app0: brk set ok\n");
    } else {
        user::write(1, "app0: brk set wrong\n");
    }

    if user::close(0) == 0 {
        user::write(1, "app0: close stdin ok\n");
    } else {
        user::write(1, "app0: close stdin wrong\n");
    }

    if user::close(99) == -1 {
        user::write(1, "app0: close invalid ok\n");
    } else {
        user::write(1, "app0: close invalid wrong\n");
    }

    let mut stat = [0u8; user::STAT_SIZE];
    if user::fstat(1, &mut stat) == 0 {
        user::write(1, "app0: fstat stdout ok\n");
    } else {
        user::write(1, "app0: fstat stdout wrong\n");
    }

    if user::fstat(99, &mut stat) == -1 {
        user::write(1, "app0: fstat invalid ok\n");
    } else {
        user::write(1, "app0: fstat invalid wrong\n");
    }

    let dev_null = user::open(b"/dev/null\0", 0);
    if dev_null >= 3 {
        user::write(1, "app0: open dev/null ok\n");
    } else {
        user::write(1, "app0: open dev/null wrong\n");
    }

    let dev_null_2 = user::open(b"/dev/null\0", 0);
    if dev_null_2 > dev_null {
        user::write(1, "app0: fd alloc ok\n");
    } else {
        user::write(1, "app0: fd alloc wrong\n");
    }

    if dev_null >= 0 && user::close(dev_null as usize) == 0 {
        user::write(1, "app0: close dev/null ok\n");
    } else {
        user::write(1, "app0: close dev/null wrong\n");
    }

    if dev_null_2 >= 0 && user::close(dev_null_2 as usize) == 0 {
        user::write(1, "app0: close dev/null 2 ok\n");
    } else {
        user::write(1, "app0: close dev/null 2 wrong\n");
    }

    if dev_null >= 0 && user::close(dev_null as usize) == -1 {
        user::write(1, "app0: close released fd ok\n");
    } else {
        user::write(1, "app0: close released fd wrong\n");
    }

    if user::open(b"/missing\0", 0) == -1 {
        user::write(1, "app0: open missing ok\n");
    } else {
        user::write(1, "app0: open missing wrong\n");
    }

    let hello_fd = user::open(b"/hello.txt\0", 0);
    if hello_fd >= 3 {
        user::write(1, "app0: open hello ok\n");
    } else {
        user::write(1, "app0: open hello wrong\n");
    }

    let mut file_buf = [0u8; 32];
    let read_len = if hello_fd >= 0 {
        user::read(hello_fd as usize, &mut file_buf)
    } else {
        -1
    };

    if read_len == 18 && file_buf[0] == b'h' && file_buf[17] == b'\n' {
        user::write(1, "app0: read hello ok\n");
    } else {
        user::write(1, "app0: read hello wrong\n");
    }

    if hello_fd >= 0 && user::read(hello_fd as usize, &mut file_buf) == 0 {
        user::write(1, "app0: read hello eof ok\n");
    } else {
        user::write(1, "app0: read hello eof wrong\n");
    }

    if hello_fd >= 0 && user::close(hello_fd as usize) == 0 {
        user::write(1, "app0: close hello ok\n");
    } else {
        user::write(1, "app0: close hello wrong\n");
    }

    user::sys_yield();
    user::sys_exit(0);
}
