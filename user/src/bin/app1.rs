#![no_std]
#![no_main]

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    user::write(1, "app1: hello from write\n");
    user::sys_test(200);

    if user::getpid() == 1 {
        user::write(1, "app1: getpid ok\n");
    } else {
        user::write(1, "app1: getpid wrong\n");
    }

    let mut buf = [0u8; 8];
    if user::read(0, &mut buf) == 0 {
        user::write(1, "app1: read eof ok\n");
    } else {
        user::write(1, "app1: read wrong\n");
    }

    let brk0 = user::brk(0);
    if brk0 > 0 {
        user::write(1, "app1: brk query ok\n");
    } else {
        user::write(1, "app1: brk query wrong\n");
    }

    let brk1 = brk0 as usize + 4096;
    if user::brk(brk1) == brk1 as isize {
        user::write(1, "app1: brk set ok\n");
    } else {
        user::write(1, "app1: brk set wrong\n");
    }

    if user::close(0) == 0 {
        user::write(1, "app1: close stdin ok\n");
    } else {
        user::write(1, "app1: close stdin wrong\n");
    }

    if user::close(99) == -1 {
        user::write(1, "app1: close invalid ok\n");
    } else {
        user::write(1, "app1: close invalid wrong\n");
    }

    let mut stat = [0u8; user::STAT_SIZE];
    if user::fstat(1, &mut stat) == 0 {
        user::write(1, "app1: fstat stdout ok\n");
    } else {
        user::write(1, "app1: fstat stdout wrong\n");
    }

    if user::fstat(99, &mut stat) == -1 {
        user::write(1, "app1: fstat invalid ok\n");
    } else {
        user::write(1, "app1: fstat invalid wrong\n");
    }

    user::sys_yield();
    user::sys_exit(1);
}

