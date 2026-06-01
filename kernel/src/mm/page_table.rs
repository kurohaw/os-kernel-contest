use super::PhysPageNum;

const PTE_PPN_SHIFT: usize = 10;
const PTE_FLAGS_MASK: usize = 0x3ff;
const PTE_PPN_MASK: usize = (1usize << 44) - 1;

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

    pub fn is_vaild(self) -> bool {
        self.flags().contains(PTEFlags::V)
    }

    pub fn readable(self) -> bool {
        self.flags().contains(PTEFlags::R)
    }

    pub fn writable(self) -> bool {
        self.flags().contains(PTEFlags::W)
    }

    pub fn excutable(self) -> bool {
        self.flags().contains(PTEFlags::X)
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
}