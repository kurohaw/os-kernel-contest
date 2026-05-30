use crate::trap::TrapContext;

const USER_STACK_SIZE: usize = 4096 * 2;

#[repr(align(16))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static mut USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

pub fn init_user_context() -> usize {
    let user_stack_top = user_stack_top();
    let cx_addr = user_stack_top - core::mem::size_of::<TrapContext>();

    let cx = unsafe { &mut *(cx_addr as *mut TrapContext) };
    *cx = TrapContext::app_init_context(user_entry as usize, user_stack_top);

    cx_addr
}

fn user_stack_top() -> usize {
    core::ptr::addr_of!(USER_STACK) as usize + USER_STACK_SIZE 
}

#[no_mangle]
pub extern "C" fn user_entry() -> ! {
    unsafe {
        core::arch::asm!(
            "li a7, 0",
            "li a0, 41",
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
