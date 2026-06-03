mod address;
mod frame_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};

pub use frame_allocator::{alloc_frame, FrameTracker, PAGE_SIZE};

pub use memory_set::MemorySet;
pub use page_table::{PageTable, PageTableEntry, PTEFlags};

pub fn init() {
    frame_allocator::init();
    page_table::self_check();
    memory_set::self_check();
}