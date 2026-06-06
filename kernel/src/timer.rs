use core::arch::asm;

const CLOCK_FREQ: usize = 10_000_000;
const TICKS_PER_SEC: usize = 2;

pub fn set_next_trigger() {
    crate::sbi::set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}

pub fn get_time() -> usize{
    let time;
    unsafe{
        asm!("csrr {}, time", out(reg) time);
    }
    time
}

pub fn get_time_us() -> usize {
    get_time() / (CLOCK_FREQ / 1_000_000)
}
