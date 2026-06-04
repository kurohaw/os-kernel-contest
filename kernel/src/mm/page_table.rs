use super::{alloc_frame, FrameTracker, PhysAddr, PhysPageNum, VirtAddr, VirtPageNum, PAGE_SIZE};

const PTE_PPN_SHIFT: usize = 10;
const PTE_FLAGS_MASK: usize = 0x3ff;
const PTE_PPN_MASK: usize = (1usize << 44) - 1;
const SATP_MODE_SV39: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PTEFlags {
    bits: usize,
}

impl PTEFlags{
    pub const V: Self = Self { bits: 1 << 0};
    pub const R: Self = Self { bits: 1 << 1};
    pub const W: Self = Self { bits: 1 << 2};
    pub const X: Self = Self { bits: 1 << 3};
    pub const U: Self = Self { bits: 1 << 4};
    pub const G: Self = Self { bits: 1 << 5};
    pub const A: Self = Self { bits: 1 << 6};
    pub const D: Self = Self { bits: 1 << 7};

    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn from_bits(bits: usize) -> Self {
        Self { bits }
    }

    pub const fn bits(self) -> usize {
        self.bits
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }
}

impl core::ops::BitOr for PTEFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct PageTableEntry {
    bits: usize,
}

impl PageTableEntry {
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self {
            bits: (ppn.0 << PTE_PPN_SHIFT) | flags.bits(),
        }
    }

    pub fn bits(self) -> usize {
        self.bits
    }

    pub fn ppn(self) -> PhysPageNum {
        PhysPageNum((self.bits >> PTE_PPN_SHIFT) & PTE_PPN_MASK)
    }

    pub fn flags(self) -> PTEFlags {
        PTEFlags::from_bits(self.bits & PTE_FLAGS_MASK)
    }

    pub fn is_valid(self) -> bool {
        self.flags().contains(PTEFlags::V)
    }

    pub fn readable(self) -> bool {
        self.flags().contains(PTEFlags::R)
    }

    pub fn writable(self) -> bool {
        self.flags().contains(PTEFlags::W)
    }

    pub fn executable(self) -> bool {
        self.flags().contains(PTEFlags::X)
    }

    pub fn user(self) -> bool {
        self.flags().contains(PTEFlags::U)
    }
}

const PTE_COUNT: usize = PAGE_SIZE / core::mem::size_of::<PageTableEntry>();

pub struct PageTable {
    root_ppn: PhysPageNum,
    _root_frame: FrameTracker,
}

impl PageTable {
    pub fn new() -> Self {
        let root_frame = alloc_frame().expect("failed to allocate root page table");
        let root_ppn = PhysPageNum(root_frame.ppn());

        Self {
            root_ppn,
            _root_frame: root_frame,
        }
    }

    pub fn root_ppn(&self) -> PhysPageNum {
        self.root_ppn
    }

    pub fn satp_token(&self) -> usize {
        (SATP_MODE_SV39 << 60) | self.root_ppn.0
    }

    pub fn map(&self, vpn:VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn);
        assert!(!pte.is_valid(), "vpn {:#x} is already mapped", vpn.0);

        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    pub fn map_range(
        &self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        start_pa: PhysAddr,
        flags: PTEFlags,
    ) {
        assert!(start_va.is_aligned(), "start_va must be page aligned");
        assert!(end_va.is_aligned(), "edn_va must be page aligned");
        assert!(start_pa.is_aligned(), "start_pa must be page aligned ");
        assert!(start_va.0 < end_va.0, "invalid map range");

        let mut vpn = start_va.floor().0;
        let end_vpn = end_va.floor().0;
        let mut ppn = start_pa.floor().0;

        while vpn < end_vpn {
            self.map(VirtPageNum(vpn), PhysPageNum(ppn), flags);
            vpn += 1;
            ppn += 1;
        }
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        let indexes = vpn.indexes();
        let mut ppn = self.root_ppn;

        for i in 0..3 {
            let pte = ppn_to_pte_array(ppn)[indexes[i]];

            if !pte.is_valid() {
                return None;
            }

            if i == 2 {
                return Some(pte);
            }

            ppn = pte.ppn();
        }

        None
    }

    fn find_pte_create(&self, vpn: VirtPageNum) -> &'static mut PageTableEntry {
        let indexes = vpn.indexes();
        let mut ppn = self.root_ppn;

        for i in 0..2 {
            let pte = &mut ppn_to_pte_array(ppn)[indexes[i]];

            if !pte.is_valid() {
                let frame = alloc_frame().expect("failed to allocate page table frame");
                *pte = PageTableEntry::new(PhysPageNum(frame.ppn()),PTEFlags::V);
            }

            ppn = pte.ppn();
        }

        &mut ppn_to_pte_array(ppn)[indexes[2]]
    }
}

fn ppn_to_pte_array(ppn: PhysPageNum) -> &'static mut [PageTableEntry] {
    let start_pa = ppn.0 * PAGE_SIZE;

    unsafe {
        core::slice::from_raw_parts_mut(
            start_pa as *mut PageTableEntry,
            PTE_COUNT,
        )
    }
}

pub fn self_check() {
    let pte = PageTableEntry::new(
        PhysPageNum(0x80200),
        PTEFlags::V | PTEFlags::R | PTEFlags::W,
    );

    crate::println!(
        "page table entry test: ppn={:#x}, flags={:#x}",
        pte.ppn().0,
        pte.flags().bits(),
    );

    let page_table = PageTable::new();
    let vpn = VirtPageNum(0x100);
    let ppn = PhysPageNum(0x80200);
    let flags = PTEFlags::R | PTEFlags::W;

    page_table.map(vpn, ppn, flags);

    let translate = page_table
        .translate(vpn)
        .expect("mapped vpn should be translated");
    
        crate::println!(
            "page table map test: vpn={:#x}, ppn={:#x}, flags={:#x}",
            vpn.0,
            translate.ppn().0,
            translate.flags().bits(),
        );

    let range_start_va = VirtAddr(0x20_0000);
    let range_end_va = VirtAddr(0x20_3000);
    let range_start_pa = PhysAddr(0x8040_0000);

    page_table.map_range(
        range_start_va,
        range_end_va,
        range_start_pa,
        PTEFlags::R | PTEFlags::X,
    );

    let start_vpn = range_start_va.floor().0;
    let start_ppn = range_start_pa.floor().0;

    for i in 0..3 {
        let pte = page_table
            .translate(VirtPageNum(start_vpn + i))
            .expect("mapped range vpn should be translate");

        assert_eq!(pte.ppn().0, start_ppn + i);
        assert!(pte.readable());
        assert!(pte.executable());
    }

    crate::println!(
        "page table range map test: start_vpn={:#x}, start_ppn={:#x}, pages=3",
        start_vpn,
        start_ppn,
    );

}
