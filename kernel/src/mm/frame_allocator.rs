pub const PAGE_SIZE: usize = 4096;

const MEMORY_END: usize = 0x8800_0000;

pub struct FrameTracker {
    ppn: usize,
}

impl FrameTracker {
    pub fn ppn(&self) -> usize {
        self.ppn
    }

    pub fn start_pa(&self) -> usize {
        self.ppn * PAGE_SIZE
    }

    pub fn zero(&self) {
        let start = self.start_pa() as *mut u8;
        unsafe {
            core::slice::from_raw_parts_mut(start, PAGE_SIZE).fill(0);
        }
    }
}

struct StackFrameAllocator {
    current: usize,
    end: usize,
}

impl StackFrameAllocator {
    pub const fn new() -> Self {
        Self { current: 0, end: 0}
    }

    pub fn init(&mut self, start_pa:usize, end_pa:usize) {
        self.current = round_up(start_pa, PAGE_SIZE) / PAGE_SIZE;
        self.end = round_down(end_pa, PAGE_SIZE) / PAGE_SIZE;
    }

    pub fn alloc(&mut self) -> Option<FrameTracker> {
        if self.current == self.end {
            None
        } else {
            let frame = FrameTracker { ppn: self.current};
            self.current += 1;
            frame.zero();
            Some(frame)
        }
    }

    pub fn avaliable_frames(&self) -> usize {
        self.end - self.current
    }
}

static mut FRAME_ALLOCATOR: StackFrameAllocator = StackFrameAllocator::new();

pub fn init() {
    extern "C" {
        fn ekernel();
    }

    unsafe {
        let start_pa = ekernel as usize;
        FRAME_ALLOCATOR.init(start_pa, MEMORY_END);

        crate::println!(
            "frame allocator: start={:#x}, end={:#x}, frames={}",
            round_up(start_pa, PAGE_SIZE),
            MEMORY_END,
            FRAME_ALLOCATOR.avaliable_frames(),
        );
    }
}

pub fn alloc_frame() -> Option<FrameTracker> {
    unsafe { FRAME_ALLOCATOR.alloc() }
}

const fn round_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

const fn round_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}
