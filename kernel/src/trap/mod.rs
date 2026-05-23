use core::arch::{asm, global_asm};

global_asm!(include_str!("trap.S"));

const SCAUSE_INTERRUPT_BIT: usize = 1usize << 63;
const SCAUSE_EXCEPTION_CODE_MASK: usize = !SCAUSE_INTERRUPT_BIT;
const SUPERVISOR_TIMER: usize = 5;

extern "C"{
    fn __alltraps();
}

pub fn init(){
    unsafe{
        asm!("csrw stvec, {}",in(reg) __alltraps as usize);
    }
}

pub fn enable_timer_interrupt() {
    unsafe{
        let sie_stimer = 1usize << 5;
        asm!("csrs sie, {}", in(reg) sie_stimer);

        let sstatus_sie = 1usize << 1;
        asm!("csrs sstatus, {}", in(reg) sstatus_sie);
    }
}

#[no_mangle]
pub fn trap_handler() {
    let scause = read_scause();
    let stval = read_stval();

    let is_interrupt = scause & SCAUSE_INTERRUPT_BIT != 0;
    let code = scause & SCAUSE_EXCEPTION_CODE_MASK;

    if is_interrupt && code == SUPERVISOR_TIMER {
        crate::println!("timer tick");
        crate::timer::set_next_trigger();
    } else {
        panic!("unsupported trap: scause={:#x}, stval={:#x}", scause, stval);
    }
}

fn read_scause() -> usize {
    let scause;
    unsafe{
        asm!("csrr {}, scause", out(reg) scause);
    }
    scause
}

fn read_stval() -> usize{
    let stval;
    unsafe{
        asm!("csrr {}, stval", out(reg) stval);
    }
    stval
}