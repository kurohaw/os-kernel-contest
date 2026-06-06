use crate::trap::TrapContext;

pub const APP_NUM: usize = crate::loader::APP_NUM;

const USER_STACK_SIZE: usize = 4096 * 2;
const AT_NULL: usize = 0;
const AT_PHDR: usize = 3;
const AT_PHENT: usize = 4;
const AT_PHNUM: usize = 5;
const AT_PAGESZ: usize = 6;
const AT_BASE: usize = 7;
const AT_FLAGS: usize = 8;
const AT_ENTRY: usize = 9;
const AT_UID: usize = 11;
const AT_EUID: usize = 12;
const AT_GID: usize = 13;
const AT_EGID: usize = 14;
const AT_SECURE: usize = 23;
const AT_RANDOM: usize = 25;
const EXTERNAL_ARGV0: &[u8] = b"external\0";

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
    let mut user_sp = user_stack_top;

    if app_id == 0 && crate::loader::has_external_app() {
        user_sp = init_external_user_stack(cx_addr);
    }

    let cx = unsafe { &mut *(cx_addr as *mut TrapContext) };
    *cx = TrapContext::app_init_context(crate::loader::app_entry(app_id), user_sp);

    if app_id == 0 && crate::loader::has_external_app() {
        cx.x[10] = 1;
        cx.x[11] = user_sp + core::mem::size_of::<usize>();
    }

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

fn init_external_user_stack(stack_top: usize) -> usize {
    let mut sp = stack_top;

    sp = push_bytes(sp, &[0u8; 16]);
    let random_ptr = sp;

    sp = push_bytes(sp, EXTERNAL_ARGV0);
    let argv0_ptr = sp;

    sp &= !0xf;

    sp = push_aux(sp, AT_NULL, 0);
    sp = push_aux(sp, AT_RANDOM, random_ptr);
    sp = push_aux(sp, AT_SECURE, 0);
    sp = push_aux(sp, AT_EGID, 0);
    sp = push_aux(sp, AT_GID, 0);
    sp = push_aux(sp, AT_EUID, 0);
    sp = push_aux(sp, AT_UID, 0);
    sp = push_aux(sp, AT_ENTRY, crate::loader::external_app_entry());
    sp = push_aux(sp, AT_FLAGS, 0);
    sp = push_aux(sp, AT_BASE, 0);
    sp = push_aux(sp, AT_PAGESZ, crate::mm::PAGE_SIZE);
    sp = push_aux(sp, AT_PHNUM, crate::loader::external_app_phnum());
    sp = push_aux(sp, AT_PHENT, crate::loader::external_app_phentsize());
    sp = push_aux(sp, AT_PHDR, crate::loader::external_app_phdr_vaddr());

    sp = push_usize(sp, 0);
    sp = push_usize(sp, 0);
    sp = push_usize(sp, argv0_ptr);
    sp = push_usize(sp, 1);

    sp
}

fn push_aux(sp: usize, key: usize, value: usize) -> usize {
    let sp = push_usize(sp, value);
    push_usize(sp, key)
}

fn push_usize(sp: usize, value: usize) -> usize {
    let next_sp = sp - core::mem::size_of::<usize>();
    unsafe {
        (next_sp as *mut usize).write(value);
    }
    next_sp
}

fn push_bytes(sp: usize, bytes: &[u8]) -> usize {
    let next_sp = sp - bytes.len();
    unsafe {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), next_sp as *mut u8, bytes.len());
    }
    next_sp
}
