pub const APP_NUM: usize = 2;

pub fn app_entry(app_id: usize) -> usize {
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