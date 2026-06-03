use core::arch::asm;

use super::{PageTable, PageTableEntry, PTEFlags, PhysAddr, VirtAddr};

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
            fn sdata();
            fn edata();
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
                sdata as *const () as usize,
                edata as *const () as usize,
                PTEFlags::R | PTEFlags::W,
                ".data",
            );

            memory_set.map_identical_range(
                edata as *const () as usize,
                ekernel as *const () as usize,
                PTEFlags::R | PTEFlags::W,
                ".bss",
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
            fn sdata();
            fn edata();
        }

        unsafe {
            self.check_kernel_mapping(stext as *const () as usize, true, false, true);
            self.check_kernel_mapping(srodata as *const () as usize, true, false, false);
            self.check_kernel_mapping(sdata as *const () as usize, true, true, false);
            self.check_kernel_mapping(edata as *const() as usize, true, true, false);
        }

        crate::println!("kernel memory set satp token: {:#x}", self.satp_token());
        crate::println!("kernel memory set test passed");
    }
}

pub fn self_check() {
    let kernel_space = MemorySet::new_kernel();
    kernel_space.self_check();
}