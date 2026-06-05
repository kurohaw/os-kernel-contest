#![no_std]
#![no_main]

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    user::write(1, "app0: hello from write\n");
    user::sys_test(100);
    if user::getpid() == 0 {
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

    user::sys_yield();
    user::sys_exit(0);
}