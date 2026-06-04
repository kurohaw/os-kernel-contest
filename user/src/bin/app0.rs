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

    user::sys_yield();
    user::sys_exit(0);
}