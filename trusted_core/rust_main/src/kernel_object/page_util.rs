use bitflags::*;

#[macro_export]
macro_rules! BIT {
    ($e: expr) => {
        {
            1usize << $e
        }
    }
}

#[macro_export]
macro_rules! MASK {
    ($e:expr) => {
        {
             (1usize << $e) - 1usize
        }
    }
}

pub const PT_SIZE_BITS: usize = 12;
pub const RISCV_NORMAL_PAGE: usize = 0;
pub const RISCV_MEGA_PAGE: usize = 1;
pub const RISCV_GIGA_PAGE: usize = 2;
pub const RISCV_TERA_PAGE: usize = 3;

pub const RISCV_PAGE_BITS: usize = 12;
pub const RISCV_MEGA_PAGE_BITS: usize = 21;
pub const RISCV_GIGA_PAGE_BITS: usize = 30;
pub const PAGE_SIZE_BITS: usize = 0xc;
pub const PA_WIDTH_SV39: usize = 56;
pub const VA_WIDTH_SV39: usize = 39;
pub const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
pub const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

pub const PT_INDEX_BITS: usize = 9;
pub const CONFIG_PT_LEVELS: usize = 3;
pub const SAFE_OS_PAGE_BITS: usize = 12;
pub const SAFE_OS_PAGE_TABLE_BITS: usize = 12;
pub const SAFE_OS_HUGE_PAGE_BITS: usize = 30;
pub const SAFE_OS_LARGE_PAGE_BITS: usize = 21;

pub const PAGE_SIZE: usize = 1 << SAFE_OS_PAGE_BITS;

pub(super) fn clear_memory(ptr: *mut u8, bits: usize) {
    unsafe {
        core::slice::from_raw_parts_mut(ptr, BIT!(bits)).fill(0);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VmAttributes {
    pub words: [usize; 1],
}

impl VmAttributes {
    pub fn new(value: usize) -> Self {
        Self {
            words: [value & 0x1usize],
        }
    }

    pub fn from_word(w: usize) -> Self {
        Self { words: [w] }
    }

    pub fn get_execute_never(&self) -> usize {
        self.words[0] & 0x1usize
    }

    pub fn set_execute_never(&mut self, v64: usize) {
        self.words[0] &= !0x1usize;
        self.words[0] |= v64 & 0x1usize;
    }
}

pub const VM_KERNEL_ONLY: usize = 1;
pub const VM_READ_ONLY: usize = 2;
pub const VM_READ_WRITE: usize = 3;

pub fn riscvget_write_from_vmrights(vm_rights: usize) -> bool {
    vm_rights == VM_READ_WRITE
}

pub fn riscvget_read_from_vmrights(vm_rights: usize) -> bool {
    vm_rights != VM_READ_ONLY
}

#[derive(Copy, Clone)]
pub struct PTE {
    pub bits: usize,
}

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

impl PTE {
    pub fn get_ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << PPN_WIDTH_SV39) - 1)).into()
    }

    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PTE {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }

    pub fn get_flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        (self.get_flags() & PTEFlags::V) != PTEFlags::empty()
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << 56) - 1))
    }
}

impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PPN_WIDTH_SV39) - 1))
    }
}

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1 << 39) - 1))
    }
}

impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VPN_WIDTH_SV39) - 1))
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl VirtAddr {
    pub fn align_down(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }

    pub fn align_up(&self) -> VirtPageNum {
        VirtPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
    }
}

impl PhysPageNum {
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
    }

    pub fn get_pte_array(&self) -> &'static mut [PTE] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PTE, 512) }
    }

    pub fn get_pte_first(&self) -> &'static mut PTE {
        let pa: PhysAddr = (*self).into();
        unsafe { &mut *(pa.0 as *mut PTE) }
    }
}

impl VirtPageNum {
    pub fn levels(&self) -> [usize; 3] {
        let mut page = self.0;
        let mut level = [0usize; 3];
        for i in (0..3).rev() {
            level[i] = page & 511;
            page >>= 9;
        }
        level
    }
}