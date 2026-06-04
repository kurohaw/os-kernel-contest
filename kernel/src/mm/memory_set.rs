use core::arch::asm;

use super::{PageTable, PageTableEntry, PTEFlags, PhysAddr, VirtAddr};
use super::frame_allocator::MEMORY_END;

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

        let _ = app_id;

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
        }

        memory_set
    }
 
    fn map_identical_range(&self, start: usize, end: usize, flags: PTEFlags, name: &str) {
        if start == end {
            return;
        }

        crate::println!(
            "kernel map {}: {:#x}..{:#x}, flags={:#x}",
            name,
            start,
            end,
            (flags | PTEFlags::V).bits(),
        );

        self.page_table.map_range(
            VirtAddr(start),
            VirtAddr(end),
            PhysAddr(start),
            flags,
        );
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
            fn suser_text();
            fn sdata();
            fn edata();
            fn suser_stack();
        }

        unsafe {
            self.check_kernel_mapping(stext as *const () as usize, true, false, true);
            self.check_kernel_mapping(srodata as *const () as usize, true, false, false);
            self.check_kernel_mapping(suser_text as *const () as usize, true, false, true);
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
        fn suser_text();
    }

    let (user_stack_bottom, _) = crate::user::user_stack_range(app_id);

    unsafe {
        let kernel_text = self
            .translate(VirtAddr(stext as *const () as usize))
            .expect("user address space should map kernel text");

        assert!(kernel_text.readable());
        assert!(kernel_text.executable());
        assert!(!kernel_text.user());

        let user_text = self
            .translate(VirtAddr(suser_text as *const () as usize))
            .expect("user address space should map user text");

        assert!(user_text.readable());
        assert!(user_text.executable());
        assert!(user_text.user());

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