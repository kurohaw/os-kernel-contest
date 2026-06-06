use core::arch::asm;

use super::{alloc_frame, PageTable, PageTableEntry, PTEFlags, PhysAddr, PhysPageNum, VirtAddr, PAGE_SIZE};
use super::frame_allocator::MEMORY_END;

const MMIO_RANGES: &[(usize, usize)] = &[(0x1000_0000, 0x9000)];
const ELF_MAGIC: &[u8; 4] = b"\x7fELF";
const ELF_PT_LOAD: u32 = 1;
const ELF_PH_SIZE: usize = 56;
const ELF_PF_X: u32 = 1;
const ELF_PF_W: u32 = 2;
const ELF_PF_R: u32 = 4;

pub struct MemorySet {
    page_table: PageTable,
}

impl MemorySet {
    pub fn new_kernel() -> Self {
        extern "C" {
            fn stext();
            fn etext();
            fn srodata();
            fn erodata();
            fn suser_text();
            fn euser_text();
            fn sdata();
            fn edata();
            fn ebss();
            fn suser_stack();
            fn euser_stack();
            fn ekernel();
        }

        let memory_set = Self {
            page_table: PageTable::new(),
        };

        unsafe {
            memory_set.map_identical_range(
                stext as *const () as usize,
                etext as *const () as usize,
                PTEFlags::R | PTEFlags::X,
                ".text",
            );

            memory_set.map_identical_range(
                srodata as *const () as usize,
                erodata as *const () as usize,
                PTEFlags::R,
                ".rodata",
            );

            memory_set.map_identical_range(
                suser_text as *const () as usize,
                euser_text as *const () as usize,
                PTEFlags::R | PTEFlags::X | PTEFlags::U,
                ".user.text",
            );

            memory_set.map_identical_range(
                sdata as *const () as usize,
                edata as *const () as usize,
                PTEFlags::R | PTEFlags::W,
                ".data",
            );

            memory_set.map_identical_range(
                edata as *const () as usize,
                ebss as *const () as usize,
                PTEFlags::R | PTEFlags::W,
                ".bss",
            );

            memory_set.map_identical_range(
                suser_stack as *const () as usize,
                euser_stack as *const () as usize,
                PTEFlags::R | PTEFlags::W | PTEFlags::U,
                ".user.stack",
            );

            memory_set.map_identical_range(
                ekernel as *const () as usize,
                MEMORY_END,
                PTEFlags::R | PTEFlags::W,
                ".memory",
            );

            memory_set.map_mmio_ranges();
        }

        memory_set
    }

    pub fn new_user(app_id: usize) -> Self {
        extern "C" {
            fn stext();
            fn etext();
            fn srodata();
            fn erodata();
            fn suser_text();
            fn euser_text();
            fn sdata();
            fn edata();
            fn ebss();
            fn suser_stack();
            fn euser_stack();
            fn ekernel();
        }

        let memory_set = Self {
            page_table: PageTable::new(),
        };

        unsafe {
            memory_set.map_identical_range(
                stext as *const () as usize,
                etext as *const () as usize,
                PTEFlags::R | PTEFlags::X,
                ".text",
            );

            memory_set.map_identical_range(
                sdata as *const () as usize,
                edata as *const () as usize,
                PTEFlags::R | PTEFlags::W,
                ".data",
            );

            memory_set.map_identical_range(
                srodata as *const ()  as usize,
                erodata as *const ()  as usize,
                PTEFlags::R,
                ".rodata",
            );

            memory_set.map_identical_range(
                suser_text as *const () as usize,
                euser_text as *const () as usize,
                PTEFlags::R | PTEFlags::X | PTEFlags::U,
                ".user.text",
            );

            memory_set.map_identical_range(
                edata as *const () as usize,
                ebss as *const () as usize,
                PTEFlags::R | PTEFlags::W,
                ".bss",
            );

            memory_set.map_identical_range(
                suser_stack as *const () as usize,
                euser_stack as *const () as usize,
                PTEFlags::R | PTEFlags::W | PTEFlags::U,
                ".user.stack",
            );

            memory_set.map_identical_range(
                ekernel as *const () as usize,
                MEMORY_END,
                PTEFlags::R | PTEFlags::W,
                ".memory",
            );

            memory_set.map_mmio_ranges();

            if app_id == 0 && crate::loader::has_external_app() {
                memory_set.load_external_elf();
            } else {
                memory_set.load_user_app(app_id);
            }
        }

        memory_set
    }
    
    fn load_user_app(&self, app_id:usize) {
        let data = crate::loader::app_data(app_id);
        let start_va = crate::loader::USER_APP_BASE;

        let mut offset = 0;
        while offset < data.len() {
            let frame = alloc_frame().expect("failed to allocate user app frame");
            let page_va = start_va + offset;
            let copy_len = {
                let remaining = data.len() - offset;
                if remaining < PAGE_SIZE {
                    remaining
                } else {
                    PAGE_SIZE
                }
            };

            self.page_table.map(
                VirtAddr(page_va).floor(),
                PhysPageNum(frame.ppn()),
                PTEFlags::R | PTEFlags::W | PTEFlags::X | PTEFlags::U,
            );

            unsafe {
                core::ptr::copy_nonoverlapping(
                    data.as_ptr().add(offset),
                    frame.start_pa() as *mut u8,
                    copy_len,
                );
            }

            offset += PAGE_SIZE;
        }

        crate::println!(
            "user app loaded: app_id={}, va={:#x}, bytes={}",
            app_id,
            start_va,
            data.len(),
        );
    }

    fn load_external_elf(&self) {
        let data = crate::loader::external_app_data();
        assert!(is_elf(data), "external app should be a valid ELF");

        let entry = le_u64(data, 24) as usize;
        let phoff = le_u64(data, 32) as usize;
        let phentsize = le_u16(data, 54) as usize;
        let phnum = le_u16(data, 56) as usize;

        assert!(phentsize >= ELF_PH_SIZE, "unsupported ELF program header size");

        let mut index = 0usize;
        while index < phnum {
            let offset = phoff + index * phentsize;
            assert!(offset + ELF_PH_SIZE <= data.len(), "invalid ELF program header");

            if le_u32(data, offset) == ELF_PT_LOAD {
                self.load_elf_segment(data, offset);
            }

            index += 1;
        }

        if !crate::loader::has_external_app() {
            crate::println!(
                "external ELF loaded: entry={:#x}, phnum={}, bytes={}",
                entry,
                phnum,
                data.len(),
            );
        }
    }

    pub fn map_user_zero_range(&self, start: usize, end: usize) -> bool {
        if start >= end {
            return true;
        }

        let mut page_va = round_down(start, PAGE_SIZE);
        let end_va = round_up(end, PAGE_SIZE);

        while page_va < end_va {
            if self.page_table.translate(VirtAddr(page_va).floor()).is_none() {
                let frame = match alloc_frame() {
                    Some(frame) => frame,
                    None => return false,
                };

                self.page_table.map(
                    VirtAddr(page_va).floor(),
                    PhysPageNum(frame.ppn()),
                    PTEFlags::R | PTEFlags::W | PTEFlags::U,
                );
            }

            page_va += PAGE_SIZE;
        }

        true
    }

    fn load_elf_segment(&self, elf: &[u8], ph_offset: usize) {
        let flags = le_u32(elf, ph_offset + 4);
        let file_offset = le_u64(elf, ph_offset + 8) as usize;
        let vaddr = le_u64(elf, ph_offset + 16) as usize;
        let filesz = le_u64(elf, ph_offset + 32) as usize;
        let memsz = le_u64(elf, ph_offset + 40) as usize;

        if memsz == 0 {
            return;
        }

        assert!(file_offset + filesz <= elf.len(), "ELF segment exceeds file");

        let segment_start = round_down(vaddr, PAGE_SIZE);
        let segment_end = round_up(vaddr + memsz, PAGE_SIZE);
        let mut page_va = segment_start;

        while page_va < segment_end {
            let frame = alloc_frame().expect("failed to allocate ELF segment frame");
            self.page_table.map(
                VirtAddr(page_va).floor(),
                PhysPageNum(frame.ppn()),
                elf_pte_flags(flags),
            );

            let page_file_start = if page_va < vaddr { vaddr } else { page_va };
            let page_file_end = core::cmp::min(page_va + PAGE_SIZE, vaddr + filesz);

            if page_file_start < page_file_end {
                let src_offset = file_offset + page_file_start - vaddr;
                let dst_offset = page_file_start - page_va;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        elf.as_ptr().add(src_offset),
                        (frame.start_pa() + dst_offset) as *mut u8,
                        page_file_end - page_file_start,
                    );
                }
            }

            page_va += PAGE_SIZE;
        }
    }

    fn map_identical_range(&self, start: usize, end: usize, flags: PTEFlags, name: &str) {
        if start == end {
            return;
        }

        if !crate::loader::has_external_app() {
            crate::println!(
                "kernel map {}: {:#x}..{:#x}, flags={:#x}",
                name,
                start,
                end,
                (flags | PTEFlags::V).bits(),
            );
        }

        self.page_table.map_range(
            VirtAddr(start),
            VirtAddr(end),
            PhysAddr(start),
            flags,
        );
    }

    fn map_mmio_ranges(&self) {
        for &(start, size) in MMIO_RANGES {
            self.map_identical_range(start, start + size, PTEFlags::R | PTEFlags::W, ".mmio");
        }
    }

    pub fn translate(&self, va: VirtAddr) -> Option<PageTableEntry> {
        self.page_table.translate(va.floor())
    }

    pub fn satp_token(&self) -> usize {
        self.page_table.satp_token()
    }

    pub fn activate(&self) {
        let token = self.satp_token();

        unsafe {
            asm!("csrw satp, {}", in(reg) token);
            asm!("sfence.vma");
        }

        crate::println!("kernel memory set activated: satp={:#x}", token);
    }

    fn check_kernel_mapping(&self, va: usize, readable: bool, writable: bool, executable: bool) {
        let pte = self
            .translate(VirtAddr(va))
            .expect("kernel address should be mapped ");
        
        assert_eq!(pte.ppn().0, PhysAddr(va).floor().0);
        assert_eq!(pte.readable(), readable);
        assert_eq!(pte.writable(), writable);
        assert_eq!(pte.executable(), executable);
    }

    pub fn self_check(&self) {
        extern "C" {
            fn stext();
            fn srodata();
            fn sdata();
            fn edata();
            fn suser_stack();
        }

        unsafe {
            self.check_kernel_mapping(stext as *const () as usize, true, false, true);
            self.check_kernel_mapping(srodata as *const () as usize, true, false, false);
            self.check_kernel_mapping(sdata as *const () as usize, true, true, false);
            self.check_kernel_mapping(edata as *const () as usize, true, true, false);
            self.check_kernel_mapping(suser_stack as *const () as usize,true, true, false);
        }

        crate::println!("kernel memory set satp token: {:#x}", self.satp_token());
        crate::println!("kernel memory set test passed");
    }

    pub fn self_check_user(&self, app_id: usize) {
    extern "C" {
        fn stext();
    }

    let (user_stack_bottom, _) = crate::user::user_stack_range(app_id);

    unsafe {
        let kernel_text = self
            .translate(VirtAddr(stext as *const () as usize))
            .expect("user address space should map kernel text");

        assert!(kernel_text.readable());
        assert!(kernel_text.executable());
        assert!(!kernel_text.user());

        let app_entry = self
            .translate(VirtAddr(crate::loader::USER_APP_BASE))
            .expect("user address space should map user app entry");
        
        assert!(app_entry.readable());
        assert!(app_entry.executable());
        assert!(app_entry.user());

        let user_stack = self
            .translate(VirtAddr(user_stack_bottom))
            .expect("user address space should map user stack");

        assert!(user_stack.readable());
        assert!(user_stack.writable());
        assert!(user_stack.user());
    }

    crate::println!(
        "user memory set test passed: app_id={}, satp={:#x}",
        app_id,
        self.satp_token(),
    );
}
}

pub fn self_check() {
    let kernel_space = MemorySet::new_kernel();
    kernel_space.self_check();

    let mut app_id = 0;
    while app_id < crate::user::APP_NUM {
        let user_space = MemorySet::new_user(app_id);
        user_space.self_check_user(app_id);
        app_id += 1;
    }
}

fn is_elf(data: &[u8]) -> bool {
    data.len() >= 64 && &data[..4] == ELF_MAGIC
}

fn elf_pte_flags(flags: u32) -> PTEFlags {
    let mut pte_flags = PTEFlags::U;

    if flags & ELF_PF_R != 0 {
        pte_flags = pte_flags | PTEFlags::R;
    }

    if flags & ELF_PF_W != 0 {
        pte_flags = pte_flags | PTEFlags::R | PTEFlags::W;
    }

    if flags & ELF_PF_X != 0 {
        pte_flags = pte_flags | PTEFlags::X;
    }

    pte_flags
}

fn round_down(value: usize, align: usize) -> usize {
    value & !(align - 1)
}

fn round_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

fn le_u16(buffer: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([buffer[offset], buffer[offset + 1]])
}

fn le_u32(buffer: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    ])
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
