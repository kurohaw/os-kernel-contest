use core::arch::asm;

mod address;
mod frame_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};

pub use frame_allocator::{alloc_frame, FrameTracker, PAGE_SIZE};

pub use memory_set::MemorySet;
pub use page_table::{PageTable, PageTableEntry, PTEFlags};

static mut KERNEL_SPACE: Option<MemorySet> = None;
pub fn init() {
    frame_allocator::init();
    page_table::self_check();
    
    let kernel_space = MemorySet::new_kernel();
    kernel_space.self_check();

    let mut app_id = 0;
    while app_id < crate::user::APP_NUM {
        let user_space = MemorySet::new_user(app_id);
        user_space.self_check_user(app_id);
        app_id += 1;
    }

    unsafe {
        KERNEL_SPACE = Some(kernel_space);
        KERNEL_SPACE.as_ref().unwrap().activate();
    }
}

pub fn activate_satp(token: usize) {
    unsafe {
        asm!("csrw satp, {}", in(reg) token);
        asm!("sfence.vma");
    }
}