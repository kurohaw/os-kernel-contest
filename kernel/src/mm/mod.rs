mod address;
mod frame_allocator;
mod page_table;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};

pub use frame_allocator::{alloc_frame, FrameTracker, PAGE_SIZE};

pub use page_table::{PageTableEntry, PTEFlags};

pub fn init() {
    frame_allocator::init();
    page_table::self_check();
}