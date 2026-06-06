const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_SHUTDOWN: usize = 8;
const SBI_EXT_TIME: usize = 0x54494D45;
const SBI_SET_TIMER: usize = 0;
const SBI_EXT_SRST: usize = 0x53525354;
const SBI_SYSTEM_RESET: usize = 0;
const RESET_TYPE_SHUTDOWN: usize = 0;
const RESET_REASON_NO_REASON: usize = 0;

#[inline(always)]
fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x16") fid,
            in("x17") eid,
        );
    }
    ret
}

pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, 0, c, 0, 0);
}

pub fn set_timer(timer: usize) {
    sbi_call(SBI_EXT_TIME, SBI_SET_TIMER, timer, 0, 0);
}

pub fn shutdown() -> ! {
    let ret = sbi_call(
        SBI_EXT_SRST,
        SBI_SYSTEM_RESET,
        RESET_TYPE_SHUTDOWN,
        RESET_REASON_NO_REASON,
        0,
    );

    if ret != 0 {
        sbi_call(SBI_SHUTDOWN, 0, 0, 0, 0);
    }

    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
