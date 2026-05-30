#![no_std]
#![no_main]

mod console;
mod lang_items;
mod sbi;
mod syscall;
mod timer;
mod trap;
mod user;

core::arch::global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();

    println!("Hello kernel");
    println!("kernel started");

    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();

    user::run_first_user();
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    unsafe {
        let start = sbss as *const () as usize;
        let end = ebss as *const () as usize;
        core::slice::from_raw_parts_mut(
            start as *mut u8,
            end - start,
        )
        .fill(0);
    }
}

