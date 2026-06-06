use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};

const VIRTIO_MMIO_BASES: [usize; 8] = [
    0x1000_1000,
    0x1000_2000,
    0x1000_3000,
    0x1000_4000,
    0x1000_5000,
    0x1000_6000,
    0x1000_7000,
    0x1000_8000,
];

const VIRTIO_MAGIC: u32 = 0x7472_6976;
const VIRTIO_VERSION_LEGACY: u32 = 1;
const VIRTIO_VERSION_MODERN: u32 = 2;
const VIRTIO_DEVICE_BLOCK: u32 = 2;

const REG_MAGIC: usize = 0x000;
const REG_VERSION: usize = 0x004;
const REG_DEVICE_ID: usize = 0x008;
const REG_DEVICE_FEATURES: usize = 0x010;
const REG_DEVICE_FEATURES_SEL: usize = 0x014;
const REG_DRIVER_FEATURES: usize = 0x020;
const REG_DRIVER_FEATURES_SEL: usize = 0x024;
const REG_LEGACY_GUEST_PAGE_SIZE: usize = 0x028;
const REG_QUEUE_SEL: usize = 0x030;
const REG_QUEUE_NUM_MAX: usize = 0x034;
const REG_QUEUE_NUM: usize = 0x038;
const REG_QUEUE_ALIGN: usize = 0x03c;
const REG_QUEUE_PFN: usize = 0x040;
const REG_QUEUE_READY: usize = 0x044;
const REG_QUEUE_NOTIFY: usize = 0x050;
const REG_INTERRUPT_STATUS: usize = 0x060;
const REG_INTERRUPT_ACK: usize = 0x064;
const REG_STATUS: usize = 0x070;
const REG_QUEUE_DESC_LOW: usize = 0x080;
const REG_QUEUE_DESC_HIGH: usize = 0x084;
const REG_QUEUE_DRIVER_LOW: usize = 0x090;
const REG_QUEUE_DRIVER_HIGH: usize = 0x094;
const REG_QUEUE_DEVICE_LOW: usize = 0x0a0;
const REG_QUEUE_DEVICE_HIGH: usize = 0x0a4;

const STATUS_ACKNOWLEDGE: u32 = 1;
const STATUS_DRIVER: u32 = 2;
const STATUS_DRIVER_OK: u32 = 4;
const STATUS_FEATURES_OK: u32 = 8;
const STATUS_FAILED: u32 = 128;

const VIRTIO_F_VERSION_1: u32 = 1;
const VIRTQ_DESC_F_NEXT: u16 = 1;
const VIRTQ_DESC_F_WRITE: u16 = 2;
const QUEUE_SIZE: usize = 8;
const QUEUE_ALIGN_LEGACY: usize = 4096;
const QUEUE_MEM_SIZE: usize = 8192;
const DESC_TABLE_SIZE: usize = core::mem::size_of::<VirtqDesc>() * QUEUE_SIZE;
const AVAIL_RING_SIZE: usize = 2 + 2 + 2 * QUEUE_SIZE;
pub const BLOCK_SIZE: usize = 512;
const SECTOR_ZERO: u64 = 0;
const VIRTIO_BLK_T_IN: u32 = 0;
const VIRTIO_BLK_S_OK: u8 = 0;
const READ_SPIN_LIMIT: usize = 10_000_000;

#[derive(Clone, Copy)]
struct BlockDevice {
    base: usize,
    version: u32,
    ready: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

#[repr(C, align(2))]
struct AvailRing {
    flags: u16,
    idx: u16,
    ring: [u16; QUEUE_SIZE],
}

#[repr(C, align(4))]
struct UsedRing {
    flags: u16,
    idx: u16,
    ring: [VirtqUsedElem; QUEUE_SIZE],
}

#[repr(C)]
struct BlkReqHeader {
    request_type: u32,
    reserved: u32,
    sector: u64,
}

#[repr(C, align(4096))]
struct QueueMem([u8; QUEUE_MEM_SIZE]);

static mut QUEUE_MEM: QueueMem = QueueMem([0; QUEUE_MEM_SIZE]);
static mut DEVICE: BlockDevice = BlockDevice {
    base: 0,
    version: 0,
    ready: false,
};
static mut REQ_HEADER: BlkReqHeader = BlkReqHeader {
    request_type: VIRTIO_BLK_T_IN,
    reserved: 0,
    sector: SECTOR_ZERO,
};
static mut INIT_BUFFER: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
static mut STATUS: u8 = 0xff;

pub fn init() {
    match find_block_device() {
        Some(base) => {
            let version = mmio_read32(base, REG_VERSION);
            crate::println!("virtio-blk: device found at {:#x}, version={}", base, version);
            if unsafe { init_device(base, version) } {
                crate::println!("virtio-blk: ready");
            } else {
                crate::println!("virtio-blk: init failed");
            }
        }
        None => {
            crate::println!("virtio-blk: device not found");
        }
    }
}

fn find_block_device() -> Option<usize> {
    for base in VIRTIO_MMIO_BASES {
        let magic = mmio_read32(base, REG_MAGIC);
        let version = mmio_read32(base, REG_VERSION);
        let device_id = mmio_read32(base, REG_DEVICE_ID);

        if magic == VIRTIO_MAGIC
            && (version == VIRTIO_VERSION_LEGACY || version == VIRTIO_VERSION_MODERN)
            && device_id == VIRTIO_DEVICE_BLOCK
        {
            return Some(base);
        }
    }

    None
}

pub fn read_sector(sector: u64, buffer: &mut [u8]) -> bool {
    if buffer.len() != BLOCK_SIZE {
        return false;
    }

    unsafe {
        if !DEVICE.ready {
            return false;
        }

        read_sector_raw(DEVICE.base, DEVICE.version, sector, buffer.as_mut_ptr())
    }
}

unsafe fn init_device(base: usize, version: u32) -> bool {
    mmio_write32(base, REG_STATUS, 0);
    set_status(base, STATUS_ACKNOWLEDGE);
    set_status(base, STATUS_DRIVER);

    if version == VIRTIO_VERSION_MODERN {
        mmio_write32(base, REG_DEVICE_FEATURES_SEL, 1);
        let device_features_hi = mmio_read32(base, REG_DEVICE_FEATURES);
        if device_features_hi & VIRTIO_F_VERSION_1 == 0 {
            set_status(base, STATUS_FAILED);
            return false;
        }

        mmio_write32(base, REG_DRIVER_FEATURES_SEL, 0);
        mmio_write32(base, REG_DRIVER_FEATURES, 0);
        mmio_write32(base, REG_DRIVER_FEATURES_SEL, 1);
        mmio_write32(base, REG_DRIVER_FEATURES, VIRTIO_F_VERSION_1);

        set_status(base, STATUS_FEATURES_OK);
        if mmio_read32(base, REG_STATUS) & STATUS_FEATURES_OK == 0 {
            set_status(base, STATUS_FAILED);
            return false;
        }
    } else {
        mmio_write32(base, REG_DRIVER_FEATURES_SEL, 0);
        mmio_write32(base, REG_DRIVER_FEATURES, 0);
        mmio_write32(base, REG_LEGACY_GUEST_PAGE_SIZE, QUEUE_ALIGN_LEGACY as u32);
    }

    if !setup_queue(base, version) {
        set_status(base, STATUS_FAILED);
        return false;
    }

    set_status(base, STATUS_DRIVER_OK);

    DEVICE = BlockDevice {
        base,
        version,
        ready: true,
    };

    let init_buffer =
        core::slice::from_raw_parts_mut(addr_of_mut!(INIT_BUFFER) as *mut u8, BLOCK_SIZE);
    if !read_sector(SECTOR_ZERO, init_buffer) {
        DEVICE.ready = false;
        return false;
    }

    true
}

unsafe fn setup_queue(base: usize, version: u32) -> bool {
    core::ptr::write_bytes(queue_base() as *mut u8, 0, QUEUE_MEM_SIZE);

    mmio_write32(base, REG_QUEUE_SEL, 0);

    let max = mmio_read32(base, REG_QUEUE_NUM_MAX);
    if max < QUEUE_SIZE as u32 {
        return false;
    }

    mmio_write32(base, REG_QUEUE_NUM, QUEUE_SIZE as u32);

    if version == VIRTIO_VERSION_LEGACY {
        mmio_write32(base, REG_QUEUE_ALIGN, QUEUE_ALIGN_LEGACY as u32);
        mmio_write32(base, REG_QUEUE_PFN, (queue_base() >> 12) as u32);
    } else {
        mmio_write32(base, REG_QUEUE_READY, 0);
        write_addr(base, REG_QUEUE_DESC_LOW, REG_QUEUE_DESC_HIGH, desc_ptr() as usize);
        write_addr(
            base,
            REG_QUEUE_DRIVER_LOW,
            REG_QUEUE_DRIVER_HIGH,
            avail_ptr() as usize,
        );
        write_addr(
            base,
            REG_QUEUE_DEVICE_LOW,
            REG_QUEUE_DEVICE_HIGH,
            used_ptr(VIRTIO_VERSION_MODERN) as usize,
        );
        mmio_write32(base, REG_QUEUE_READY, 1);
    }

    true
}

unsafe fn read_sector_raw(base: usize, version: u32, sector: u64, buffer: *mut u8) -> bool {
    REQ_HEADER.request_type = VIRTIO_BLK_T_IN;
    REQ_HEADER.reserved = 0;
    REQ_HEADER.sector = sector;
    core::ptr::write_bytes(buffer, 0, BLOCK_SIZE);
    STATUS = 0xff;

    let desc = desc_ptr();
    let avail = avail_ptr();
    let used = used_ptr(version);

    write_volatile(desc.add(0), VirtqDesc {
        addr: addr_of!(REQ_HEADER) as u64,
        len: core::mem::size_of::<BlkReqHeader>() as u32,
        flags: VIRTQ_DESC_F_NEXT,
        next: 1,
    });
    write_volatile(desc.add(1), VirtqDesc {
        addr: buffer as u64,
        len: BLOCK_SIZE as u32,
        flags: VIRTQ_DESC_F_NEXT | VIRTQ_DESC_F_WRITE,
        next: 2,
    });
    write_volatile(desc.add(2), VirtqDesc {
        addr: addr_of_mut!(STATUS) as u64,
        len: 1,
        flags: VIRTQ_DESC_F_WRITE,
        next: 0,
    });

    let old_used_idx = read_volatile(addr_of!((*used).idx));
    let avail_idx = read_volatile(addr_of!((*avail).idx));
    write_volatile(addr_of_mut!((*avail).ring[(avail_idx as usize) % QUEUE_SIZE]), 0);
    write_volatile(addr_of_mut!((*avail).idx), avail_idx.wrapping_add(1));

    fence_io();
    mmio_write32(base, REG_QUEUE_NOTIFY, 0);

    let mut spins = 0usize;
    while read_volatile(addr_of!((*used).idx)) == old_used_idx {
        spins += 1;
        if spins > READ_SPIN_LIMIT {
            crate::println!(
                "virtio-blk: read timeout, used_idx={}, old_used_idx={}, status={}",
                read_volatile(addr_of!((*used).idx)),
                old_used_idx,
                read_volatile(addr_of!(STATUS)),
            );
            return false;
        }
    }

    let interrupt_status = mmio_read32(base, REG_INTERRUPT_STATUS);
    if interrupt_status != 0 {
        mmio_write32(base, REG_INTERRUPT_ACK, interrupt_status);
    }

    let status = read_volatile(addr_of!(STATUS));
    if status != VIRTIO_BLK_S_OK {
        crate::println!("virtio-blk: bad status={}", status);
    }
    status == VIRTIO_BLK_S_OK
}

fn queue_base() -> usize {
    addr_of_mut!(QUEUE_MEM) as usize
}

fn desc_ptr() -> *mut VirtqDesc {
    queue_base() as *mut VirtqDesc
}

fn avail_ptr() -> *mut AvailRing {
    (queue_base() + DESC_TABLE_SIZE) as *mut AvailRing
}

fn used_ptr(version: u32) -> *mut UsedRing {
    let align = if version == VIRTIO_VERSION_LEGACY {
        QUEUE_ALIGN_LEGACY
    } else {
        core::mem::align_of::<UsedRing>()
    };
    align_up(queue_base() + DESC_TABLE_SIZE + AVAIL_RING_SIZE, align) as *mut UsedRing
}

fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

fn set_status(base: usize, status: u32) {
    let current = mmio_read32(base, REG_STATUS);
    mmio_write32(base, REG_STATUS, current | status);
}

fn write_addr(base: usize, low_reg: usize, high_reg: usize, addr: usize) {
    mmio_write32(base, low_reg, addr as u32);
    mmio_write32(base, high_reg, (addr >> 32) as u32);
}

fn mmio_read32(base: usize, offset: usize) -> u32 {
    unsafe { read_volatile((base + offset) as *const u32) }
}

fn mmio_write32(base: usize, offset: usize, value: u32) {
    unsafe {
        write_volatile((base + offset) as *mut u32, value);
    }
}

fn fence_io() {
    unsafe {
        core::arch::asm!("fence iorw, iorw");
    }
}
