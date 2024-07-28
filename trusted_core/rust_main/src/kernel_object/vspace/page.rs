use crate::cpu::{sfence_vma, w_satp};
use super::page_config::{
    PAGE_SIZE_BITS, PPN_WIDTH_SV39, SAFE_OS_PAGE_BITS, VPN_WIDTH_SV39,
};
use super::utils::clear_memory;
use bitflags::*;
use core::arch::asm;

const PAGE_SIZE: usize = 1 << SAFE_OS_PAGE_BITS;

/// satp register
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SatpT {
    pub words: usize,
}

impl SatpT {
    #[allow(clippy::identity_op)]
    pub fn new(mode: usize, asid: usize, ppn: usize) -> Self {
        SatpT {
            words: 0
                | (mode & 0xfusize) << 60
                | (asid & 0xffffusize) << 44
                | (ppn & 0xfffffffffffusize) << 0,
        }
    }
}

pub struct PageFrame {
    root_ppn: PhysPageNum,
}

impl PageFrame {
    pub fn new(paddr: usize, page_size: usize) -> Self {
        PageFrame {
            root_ppn: PhysPageNum(paddr / page_size),
        }
    }

    pub fn page_map(
        &mut self,
        vpn: usize,
        top_pt_pnum: PhysPageNum,
        pte_exr_flag: PTEFlags,
    ) -> i32 {
        let vpn_page_num: VirtPageNum = vpn.into();
        let map_pt_num: PhysPageNum = self.root_ppn;
        let levels = vpn_page_num.levels();
        let mut target_pt: PhysPageNum = top_pt_pnum;

        for (i, level) in levels.iter().enumerate() {
            let pte = &mut target_pt.get_pte_array()[*level];
            // if not found, than create
            if !pte.is_valid() {
                *pte = PTE::new(
                    map_pt_num,
                    PTEFlags::V
                        | pte_exr_flag
                        | PTEFlags::U
                        | PTEFlags::G
                        | PTEFlags::A
                        | PTEFlags::D,
                );
                break;
            } else if i == 2 {
                return -1;
            }
            target_pt = pte.get_ppn();
        }
        sfence_vma();
        0
    }

    /// unmap a physical page from the page table
    /// vpn: virutal address to unmap
    /// top_pt_pnum: physcial page number associated with top most page table
    pub fn page_table_unmap(
        &mut self,
        vpn: usize,
        top_pt_pnum: PhysPageNum,
        page_size_bits: usize,
    ) -> i32 {
        let vpn_page_num: VirtPageNum = vpn.into();
        let levels = vpn_page_num.levels();
        let unmap_pt_num: PhysPageNum = self.root_ppn;
        let unmap_pt_addr: PhysAddr = self.root_ppn.into();
        let mut tmp_pnum: PhysPageNum = top_pt_pnum;

        for (i, level) in levels.iter().enumerate() {
            let pte = &mut tmp_pnum.get_pte_array()[*level];
            if pte.get_ppn() == unmap_pt_num {
                // clear entry and its associated physical page
                *pte = PTE { bits: 0 };
                clear_memory(unmap_pt_addr.0 as *mut u8, page_size_bits);
                break;
            } else if i == 2 {
                return -1;
            }
            tmp_pnum = pte.get_ppn();
        }
        sfence_vma();
        0
    }
}

pub struct PageTable {
    pub root_ppn: PhysPageNum,
}

impl PageTable {
    pub fn new(paddr: usize) -> Self {
        PageTable {
            root_ppn: PhysPageNum(paddr / PAGE_SIZE),
        }
    }

    pub fn page_table_map(&mut self, vpn: usize, top_pt_pnum: PhysPageNum) -> i32 {
        let vpn_page_num: VirtPageNum = vpn.into();
        let map_pt_num: PhysPageNum = self.root_ppn;
        let levels = vpn_page_num.levels();
        let mut target_pt: PhysPageNum = top_pt_pnum;

        for (i, level) in levels.iter().enumerate() {
            let pte = &mut target_pt.get_pte_array()[*level];
            if i == 2 {
                return -1;
            }
            if !pte.is_valid() {
                *pte = PTE::new(map_pt_num, PTEFlags::V);
                break;
            }
            target_pt = pte.get_ppn();
        }
        sfence_vma();
        0
    }

    pub fn page_table_unmap(&mut self, vpn: usize, top_pt_pnum: PhysPageNum) -> i32 {
        let vpn_page_num: VirtPageNum = vpn.into();
        let levels = vpn_page_num.levels();
        let unmap_pt_num: PhysPageNum = self.root_ppn;
        let unmap_pt_addr: PhysAddr = self.root_ppn.into();
        let mut tmp_pnum: PhysPageNum = top_pt_pnum;

        for (i, level) in levels.iter().enumerate() {
            let pte = &mut tmp_pnum.get_pte_array()[*level];
            if i == 2 {
                return -1;
            }
            if pte.get_ppn() == unmap_pt_num {
                *pte = PTE { bits: 0 };
                clear_memory(unmap_pt_addr.0 as *mut u8, SAFE_OS_PAGE_BITS);
                break;
            }
            tmp_pnum = pte.get_ppn();
        }
        sfence_vma();
        0
    }

    /// configure satp register to bind to current virtual addr space
    pub fn set_vm_root(&self, asid: usize) {
        let satp = SatpT::new(8usize, asid, self.root_ppn.0 >> 12);
        w_satp(satp.words);
        sfence_vma();
        let satp = 8usize << 60 | self.root_ppn.0;
        unsafe {
            asm!("csrw satp, {0}", in(reg) satp);
            asm!("sfence.vma");
        }
    }
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

    fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PTE {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
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

    fn get_pte_array(&self) -> &'static mut [PTE] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PTE, 512) }
    }

    pub fn get_pte_first(&self) -> &'static mut PTE {
        let pa: PhysAddr = (*self).into();
        unsafe { &mut *(pa.0 as *mut PTE) }
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
