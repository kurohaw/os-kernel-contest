use core::arch::{asm, global_asm};

global_asm!(include_str!("trap.S"));

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize,
}

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
pub extern "C" fn trap_handler(cx: &mut TrapContext) {
    let scause = read_scause();
    let stval = read_stval();

    match decode_trap(scause) {
        Trap::SupervisorTimer => handle_timer_interrupt(),
        Trap::Unknown {is_interrupt, code} => {
            panic!(
                "unsupported trap: interrupt={}, code={}, scause={:#x}, stval={:#x}, sepc={:#x}",
                is_interrupt,
                code,
                scause,
                stval,
                cx.sepc,
            );
        }
    }


}

fn handle_timer_interrupt() {
    crate::println!("timer tick");
    crate::timer::set_next_trigger();
}

fn decode_trap(scause: usize) -> Trap {
    let is_interrupt = scause & SCAUSE_INTERRUPT_BIT != 0;
    let code = scause & SCAUSE_EXCEPTION_CODE_MASK;

    if is_interrupt && code == SUPERVISOR_TIMER {
        Trap::SupervisorTimer
    }else {
        Trap::Unknown { is_interrupt, code}
    }
}

enum Trap{
    SupervisorTimer,
    Unknown{
        is_interrupt: bool,
        code: usize,
    },
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
