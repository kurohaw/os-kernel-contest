pub const APP_NUM: usize = 2;
pub const USER_APP_BASE: usize = 0x10000;

pub fn init() {
    let mut app_id = 0;

    while app_id < APP_NUM {
        let data = app_data(app_id);
        assert!(!data.is_empty(), "user app binary should not be empty");

        crate::println!(
            "loader: app{} binary size={} bytes, entry={:#x}",
            app_id,
            data.len(),
            USER_APP_BASE,
        );

        app_id += 1;
    }
}

pub fn app_data(app_id: usize) -> &'static [u8] {
    match app_id {
        0 => include_bytes!("../../user/build/app0.bin"),
        1 => include_bytes!("../../user/build/app1.bin"),
        _ => panic!("invalid app id {}", app_id),
    }
}
pub fn app_entry(app_id: usize) -> usize {
    if app_id >= APP_NUM {
        panic!("invalid app id {}", app_id);
    }

    USER_APP_BASE
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