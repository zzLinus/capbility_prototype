use super::page_util::{
    PhysPageNum, PTEFlags, VirtPageNum, PTE, PhysAddr,
};
use crate::cpu::{sfence_vma, w_satp};
use super::page_util::{
    clear_memory, PAGE_SIZE, SAFE_OS_PAGE_BITS,
};
use core::arch::asm;

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

#[derive(Default)]
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