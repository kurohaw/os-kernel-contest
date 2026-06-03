use core::arch::{asm, global_asm};

global_asm!(include_str!("trap.S"));

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize,
}

impl TrapContext {
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus = read_sstatus();

        const SSTATUS_SPP: usize = 1 << 8;
        const SSTATUS_SPIE: usize = 1 << 5;

        sstatus &= !SSTATUS_SPP;
        sstatus |= SSTATUS_SPIE;

        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
        };

        cx.x[2] = sp;
        cx
    }
}

const SCAUSE_INTERRUPT_BIT: usize = 1usize << 63;
const SCAUSE_EXCEPTION_CODE_MASK: usize = !SCAUSE_INTERRUPT_BIT;
const SUPERVISOR_TIMER: usize = 5;
const USER_ENV_CALL: usize = 8;

extern "C"{
    fn __alltraps();
    fn __restore();
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

pub fn enable_user_memory_access() {
    const SSTATUS_SUM: usize = 1 << 18;

    unsafe {
        asm!("csrs sstatus, {}", in(reg) SSTATUS_SUM);
    }
}

pub unsafe fn restore(cx_addr: usize) -> ! {
    asm!(
        "mv sp, {cx}",
        "j {restore}",
        cx = in(reg) cx_addr,
        restore = sym __restore,
        options(noreturn),
    )
}

#[no_mangle]
pub extern "C" fn trap_handler(cx: &mut TrapContext) {
    let scause = read_scause();
    let stval = read_stval();

    match decode_trap(scause) {
        Trap::SupervisorTimer => handle_timer_interrupt(),
        Trap::EnvironmentCall => handle_environment_call(cx),
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

fn handle_environment_call(cx: &mut TrapContext) {
    //ecall 指令的长度是4字节
    cx.sepc += 4;

    let id = cx.x[17];
    let args = [cx.x[10], cx.x[11], cx.x[12]];

    let ret = crate::syscall::syscall(id, args);
    cx.x[10] = ret as usize;

    if id == crate::syscall::SYS_YIELD {
        crate::task::suspend_current_and_run_next();
    }
}
fn decode_trap(scause: usize) -> Trap {
    let is_interrupt = scause & SCAUSE_INTERRUPT_BIT != 0;
    let code = scause & SCAUSE_EXCEPTION_CODE_MASK;

    if is_interrupt && code == SUPERVISOR_TIMER {
        Trap::SupervisorTimer
    }else if !is_interrupt && code == USER_ENV_CALL  {
        Trap::EnvironmentCall
    }else {
        Trap::Unknown { is_interrupt, code}
    }
}

enum Trap{
    SupervisorTimer,
    EnvironmentCall,
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

fn read_sstatus() -> usize {
    let sstatus;
    unsafe {
        asm!("csrr {}, sstatus", out(reg) sstatus);
    }
    sstatus
}
