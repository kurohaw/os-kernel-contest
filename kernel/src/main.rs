#![no_std]
#![no_main]

mod console;
mod fs;
mod lang_items;
mod loader;
mod mm;
mod sbi;
mod syscall;
mod task;
mod timer;
mod trap;
mod user;


core::arch::global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();

    println!("Hello kernel");
    println!("kernel started");

    loader::init();

    mm::init();

    trap::init();
    trap::enable_user_memory_access();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();

    task::init();
    task::run_first_task();
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

