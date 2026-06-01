use super::PAGE_SIZE;

const PAGE_OFFSET_BITS: usize = 12;
const VPN_INDEX_BITS: usize = 9;
const VPN_INDEX_MASK: usize = (1 << VPN_INDEX_BITS) - 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysPageNum(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPageNum(pub usize);

impl PhysAddr {
    pub fn floor(self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    pub fn ceil(self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }

    pub fn page_offset(self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    pub fn is_aligned(self) -> bool {
        self.page_offset() == 0
    }
}

impl VirtAddr {
    pub fn floor(self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }

    pub fn ceil(self) -> VirtPageNum {
        VirtPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }

    pub fn page_offset(self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    pub fn is_aligned(self) -> bool {
        self.page_offset() == 0
    }
}

impl PhysPageNum {
    pub fn start_pa(self) -> PhysAddr {
        PhysAddr(self.0 << PAGE_OFFSET_BITS)
    }
}

impl VirtPageNum {
    pub fn start_va(self) -> VirtAddr {
        VirtAddr(self.0 << PAGE_OFFSET_BITS)
    }

    pub fn indexes(self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut indexes = [0usize; 3];

        for i in (0..3).rev() {
            indexes[i] = vpn & VPN_INDEX_MASK;
            vpn >>= VPN_INDEX_BITS;
        }

        indexes
    }
}