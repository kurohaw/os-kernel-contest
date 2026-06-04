use crate::trap::TrapContext;

pub const APP_NUM: usize = 2;

const USER_STACK_SIZE: usize = 4096 * 2;

#[repr(align(16))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

#[link_section = ".user.stack"]
static mut USER_STACK_0: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

#[link_section = ".user.stack"]
static mut USER_STACK_1: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

pub fn init_user_context(app_id: usize) -> usize {
    let user_stack_top = user_stack_top(app_id);
    let cx_addr = user_stack_top - core::mem::size_of::<TrapContext>();

    let cx = unsafe { &mut *(cx_addr as *mut TrapContext) };
    *cx = TrapContext::app_init_context(user_entry(app_id), user_stack_top);

    cx_addr
}

pub fn user_stack_range(app_id: usize) -> (usize, usize) {
    let top = user_stack_top(app_id);
    let bottom = top - USER_STACK_SIZE;
    (bottom, top)
}

fn user_stack_top(app_id: usize) -> usize {
    match app_id {
        0 => core::ptr::addr_of!(USER_STACK_0) as usize + USER_STACK_SIZE,
        1 => core::ptr::addr_of!(USER_STACK_1) as usize + USER_STACK_SIZE,
        _ => panic!("invaild app id {}", app_id),
    }
}

fn user_entry(app_id: usize) -> usize {
    match app_id {
        0 => user_entry_0 as usize,
        1 => user_entry_1 as usize,
        _ => panic!("invalid app id {}", app_id),
    }
}

#[no_mangle]
#[link_section = ".user.text"]
pub extern "C" fn user_entry_0() -> ! {
    unsafe {
        core::arch::asm!(
            "li a7, 0",
            "li a0, 100",
            "ecall",
            "li a7, 2",
            "ecall",
            "li a7, 1",
            "li a0, 0",
            "ecall",
            "j .",
            options(noreturn),
        );
    }
}

#[no_mangle]
#[link_section = ".user.text"]
pub extern "C" fn user_entry_1() -> ! {
    unsafe {
        core::arch::asm!(
            "li a7, 0",
            "li a0, 200",
            "ecall",
            "li a7, 2",
            "ecall",
            "li a7, 1",
            "li a0, 1",
            "ecall",
            "j .",
            options(noreturn),
        );
    }
}
