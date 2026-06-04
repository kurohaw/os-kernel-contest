use crate::trap::TrapContext;

pub const APP_NUM: usize = crate::loader::APP_NUM;

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
    *cx = TrapContext::app_init_context(crate::loader::app_entry(app_id), user_stack_top);

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