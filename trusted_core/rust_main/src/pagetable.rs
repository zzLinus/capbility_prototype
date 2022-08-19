use bitflags::*;
use crate::kmem::Kmem;

const PAGE_SIZE: usize = 0x1000;
const PAGE_SIZE_BITS: usize = 0xc;
const PA_WIDTH_SV39: usize = 56;
const VA_WIDTH_SV39: usize = 39;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

pub struct PageTable {
    root_ppn: PhysPageNum,
}

impl PageTable {
    pub fn new(mem:&mut Kmem) -> Self {
        let frame = mem.palloc(1).unwrap();
        PageTable {
            root_ppn:(frame / PAGE_SIZE).into(),
        }
    }
    pub fn page_map(&mut self, vpn: usize, ppn: usize, flags: PTEFlags, mem: &mut Kmem) {
        let pte = self.find_pte_or_create(vpn.into(), mem).unwrap();
        *pte = PTE::new(ppn.into(), flags | PTEFlags::V);
    }
    pub fn page_unmap(&mut self, vpn: usize) {
        let pte = self.find_pte(vpn.into()).unwrap();
        *pte = PTE { bits: 0 };
    }
    fn find_pte_or_create(&mut self, vpn: VirtPageNum, mem: &mut Kmem) -> Option<&mut PTE> {
        let levels = vpn.levels();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PTE> = None;
        for (i, level) in levels.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*level];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = mem.palloc(1).unwrap();
                *pte = PTE::new((frame / PAGE_SIZE).into(), PTEFlags::V);
            }
            ppn = pte.get_ppn();
        }
        result
    }
    fn find_pte(&mut self, vpn: VirtPageNum) -> Option<&mut PTE> {
        let levels = vpn.levels();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PTE> = None;
        for (i, level) in levels.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*level];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.get_ppn();
        }
        result
    }
}

struct PTE {
    bits: usize,
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
    fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PTE {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    fn get_ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    fn get_flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    fn is_valid(&self) -> bool {
        (self.get_flags() & PTEFlags::V) != PTEFlags::empty()
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
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
    fn get_pte_array(&self) -> &'static mut [PTE] {
        let pa: PhysAddr = (*self).into();
        unsafe{ core::slice::from_raw_parts_mut(pa.0 as *mut PTE, 512) }
    }
}

impl VirtPageNum {
    fn levels(&self) -> [usize; 3] {
        let mut page = self.0;
        let mut level = [0usize; 3];
        for i in (0..3).rev() {
            level[i] = page & 511;
            page >>= 9;
        }
        level
    }
}


