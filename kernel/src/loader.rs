pub const APP_NUM: usize = 2;
pub const USER_APP_BASE: usize = 0x10000;
pub const USER_HEAP_BASE: usize = 0x20000;
pub const USER_HEAP_SIZE: usize = 16 * 1024 * 1024;
pub const EXTERNAL_APP_MAX_SIZE: usize = 4 * 1024 * 1024;
pub const EXTERNAL_ARG_MAX: usize = 8;
const EXTERNAL_GROUP_MAX_LEN: usize = 32;
const EXTERNAL_ARG_MAX_LEN: usize = 64;
const EXTERNAL_CWD_MAX_LEN: usize = 128;
const ELF_PT_LOAD: u32 = 1;
const ELF_PH_SIZE: usize = 56;
const USER_PAGE_SIZE: usize = 4096;

static mut EXTERNAL_APP: [u8; EXTERNAL_APP_MAX_SIZE] = [0; EXTERNAL_APP_MAX_SIZE];
static mut EXTERNAL_GROUP: [u8; EXTERNAL_GROUP_MAX_LEN] = [0; EXTERNAL_GROUP_MAX_LEN];
static mut EXTERNAL_ARGV: [[u8; EXTERNAL_ARG_MAX_LEN]; EXTERNAL_ARG_MAX] =
    [[0; EXTERNAL_ARG_MAX_LEN]; EXTERNAL_ARG_MAX];
static mut EXTERNAL_ARG_LEN: [usize; EXTERNAL_ARG_MAX] = [0; EXTERNAL_ARG_MAX];
static mut EXTERNAL_CWD: [u8; EXTERNAL_CWD_MAX_LEN] = [0; EXTERNAL_CWD_MAX_LEN];
static mut EXTERNAL_APP_LEN: usize = 0;
static mut EXTERNAL_GROUP_LEN: usize = 0;
static mut EXTERNAL_ARG_COUNT: usize = 0;
static mut EXTERNAL_CWD_LEN: usize = 0;
static mut EXTERNAL_APP_READY: bool = false;

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

pub fn external_app_buffer_mut() -> &'static mut [u8] {
    unsafe {
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(EXTERNAL_APP) as *mut u8,
            EXTERNAL_APP_MAX_SIZE,
        )
    }
}

pub fn set_external_app(len: usize) {
    unsafe {
        EXTERNAL_APP_LEN = len;
        EXTERNAL_APP_READY = len > 0;
    }
}

pub fn clear_external_args() {
    unsafe {
        EXTERNAL_ARG_COUNT = 0;
        let arg_len = core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(EXTERNAL_ARG_LEN) as *mut usize,
            EXTERNAL_ARG_MAX,
        );
        arg_len.fill(0);
    }
}

pub fn push_external_arg(arg: &[u8]) -> bool {
    unsafe {
        if EXTERNAL_ARG_COUNT >= EXTERNAL_ARG_MAX {
            return false;
        }

        let index = EXTERNAL_ARG_COUNT;
        let copy_len = core::cmp::min(arg.len(), EXTERNAL_ARG_MAX_LEN);
        let argv = core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(EXTERNAL_ARGV) as *mut u8,
            EXTERNAL_ARG_MAX * EXTERNAL_ARG_MAX_LEN,
        );
        let start = index * EXTERNAL_ARG_MAX_LEN;
        argv[start..start + EXTERNAL_ARG_MAX_LEN].fill(0);
        argv[start..start + copy_len].copy_from_slice(&arg[..copy_len]);

        EXTERNAL_ARG_LEN[index] = copy_len;
        EXTERNAL_ARG_COUNT += 1;
        true
    }
}

pub fn external_arg_count() -> usize {
    let count = unsafe { EXTERNAL_ARG_COUNT };
    if count == 0 {
        1
    } else {
        count
    }
}

pub fn external_arg(index: usize) -> &'static [u8] {
    let count = unsafe { EXTERNAL_ARG_COUNT };
    if count == 0 {
        if index == 0 {
            b"external"
        } else {
            b""
        }
    } else if index < count {
        let len = unsafe { EXTERNAL_ARG_LEN[index] };
        unsafe {
            core::slice::from_raw_parts(
                core::ptr::addr_of!(EXTERNAL_ARGV[index]) as *const u8,
                len,
            )
        }
    } else {
        b""
    }
}

pub fn set_external_cwd(cwd: &[u8]) {
    let copy_len = core::cmp::min(cwd.len(), EXTERNAL_CWD_MAX_LEN);

    unsafe {
        let cwd_buffer = core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(EXTERNAL_CWD) as *mut u8,
            EXTERNAL_CWD_MAX_LEN,
        );
        cwd_buffer.fill(0);
        cwd_buffer[..copy_len].copy_from_slice(&cwd[..copy_len]);
        EXTERNAL_CWD_LEN = copy_len;
    }
}

pub fn external_cwd() -> &'static [u8] {
    let len = unsafe { EXTERNAL_CWD_LEN };
    unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(EXTERNAL_CWD) as *const u8, len) }
}

pub fn set_external_group(group: &[u8]) {
    let copy_len = core::cmp::min(group.len(), EXTERNAL_GROUP_MAX_LEN);

    unsafe {
        let group_buffer = core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(EXTERNAL_GROUP) as *mut u8,
            EXTERNAL_GROUP_MAX_LEN,
        );
        group_buffer.fill(0);
        group_buffer[..copy_len].copy_from_slice(&group[..copy_len]);
        EXTERNAL_GROUP_LEN = copy_len;
    }
}

pub fn has_external_app() -> bool {
    unsafe { EXTERNAL_APP_READY }
}

pub fn external_app_data() -> &'static [u8] {
    unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(EXTERNAL_APP) as *const u8,
            EXTERNAL_APP_LEN,
        )
    }
}

pub fn external_app_entry() -> usize {
    let data = external_app_data();
    if data.len() < 32 {
        return USER_APP_BASE;
    }

    le_u64(data, 24) as usize
}

pub fn external_app_phoff() -> usize {
    let data = external_app_data();
    if data.len() < 64 {
        return 0;
    }

    le_u64(data, 32) as usize
}

pub fn external_app_phdr_vaddr() -> usize {
    let data = external_app_data();
    let phoff = external_app_phoff();
    let phentsize = external_app_phentsize();
    let phnum = external_app_phnum();

    if phoff == 0 || phentsize < ELF_PH_SIZE {
        return 0;
    }

    let mut index = 0usize;
    while index < phnum {
        let offset = phoff + index * phentsize;
        if offset + ELF_PH_SIZE > data.len() {
            return 0;
        }

        if le_u32(data, offset) == ELF_PT_LOAD {
            let file_offset = le_u64(data, offset + 8) as usize;
            let vaddr = le_u64(data, offset + 16) as usize;
            if vaddr < file_offset {
                return 0;
            }
            return vaddr - file_offset + phoff;
        }

        index += 1;
    }

    0
}

pub fn external_app_phentsize() -> usize {
    let data = external_app_data();
    if data.len() < 64 {
        return 0;
    }

    le_u16(data, 54) as usize
}

pub fn external_app_phnum() -> usize {
    let data = external_app_data();
    if data.len() < 64 {
        return 0;
    }

    le_u16(data, 56) as usize
}

pub fn external_app_heap_base() -> usize {
    let data = external_app_data();
    let phoff = external_app_phoff();
    let phentsize = external_app_phentsize();
    let phnum = external_app_phnum();
    let mut end = USER_HEAP_BASE;

    if phoff == 0 || phentsize < ELF_PH_SIZE {
        return end;
    }

    let mut index = 0usize;
    while index < phnum {
        let offset = phoff + index * phentsize;
        if offset + ELF_PH_SIZE > data.len() {
            break;
        }

        if le_u32(data, offset) == ELF_PT_LOAD {
            let vaddr = le_u64(data, offset + 16) as usize;
            let memsz = le_u64(data, offset + 40) as usize;
            if let Some(segment_end) = vaddr.checked_add(memsz) {
                if segment_end > end {
                    end = segment_end;
                }
            }
        }

        index += 1;
    }

    round_up(end, USER_PAGE_SIZE)
}

pub fn print_external_group_end() {
    let group_len = unsafe { EXTERNAL_GROUP_LEN };
    if group_len == 0 {
        return;
    }

    let group = unsafe {
        core::slice::from_raw_parts(core::ptr::addr_of!(EXTERNAL_GROUP) as *const u8, group_len)
    };

    crate::print!("#### OS COMP TEST GROUP END ");
    for &byte in group {
        crate::sbi::console_putchar(byte as usize);
    }
    crate::println!(" ####");
}

pub fn app_entry(app_id: usize) -> usize {
    if app_id >= APP_NUM {
        panic!("invalid app id {}", app_id);
    }

    if app_id == 0 && has_external_app() {
        return external_app_entry();
    }

    USER_APP_BASE
}

fn le_u64(buffer: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
        buffer[offset + 4],
        buffer[offset + 5],
        buffer[offset + 6],
        buffer[offset + 7],
    ])
}

fn le_u32(buffer: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    ])
}

fn le_u16(buffer: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([buffer[offset], buffer[offset + 1]])
}

fn round_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
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
