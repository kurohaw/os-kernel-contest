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

    user::sys_yield();
    user::sys_exit(1);
}

