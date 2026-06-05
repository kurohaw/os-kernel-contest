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

    user::sys_yield();
    user::sys_exit(1);
}

