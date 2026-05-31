mod frame_allocator;

pub use frame_allocator::{alloc_frame, FrameTracker, PAGE_SIZE};

pub fn init() {
    frame_allocator::init();
}
