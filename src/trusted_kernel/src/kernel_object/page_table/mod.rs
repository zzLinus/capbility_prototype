use super::page_util::{clear_memory, SAFE_OS_PAGE_TABLE_BITS};
use super::page_util::{PTEFlags, PhysAddr, PhysPageNum, VirtPageNum, ASID_INVALID, PTE};
use super::untyped::RetypeInit;
use crate::capability::object::{KObj, ObjPtr};
use crate::cpu::{sfence_vma, w_satp};
use core::arch::asm;

#[repr(transparent)]
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
#[derive(Default, Clone)]
pub struct PageTable {
    pub base_paddr: usize,
    pub mapped_vaddr: usize,
    pub mapped_asid: usize,
    pub mapped_flag: bool,
}

impl RetypeInit for PageTable {
    type StoredAs = [u8; 4096];
    fn retype_init_in(obj_ptr: ObjPtr<Self::StoredAs>) -> KObj {
        let base_paddr = obj_ptr.as_ptr() as usize;
        KObj::PageTable(Self {
            base_paddr,
            mapped_vaddr: 0,
            mapped_asid: ASID_INVALID,
            mapped_flag: false,
        })
    }
}

impl PageTable {
    pub fn page_table_map(
        &mut self,
        to_map_vaddr: usize,
        top_table_paddr: usize,
        asid: usize,
    ) -> i32 {
        let vpn_page_num: VirtPageNum = to_map_vaddr.into();
        let mut target_pt: PhysPageNum = top_table_paddr.into();
        let map_pt_num: PhysPageNum = self.base_paddr.into();
        let levels = vpn_page_num.levels();
        for (i, level) in levels.iter().enumerate() {
            let pte = &mut target_pt.get_pte_array()[*level];
            if i == levels.len() - 1 {
                return -1;
            }
            if !pte.is_valid() {
                *pte = PTE::new(map_pt_num, PTEFlags::V);
                self.mapped_flag = true;
                self.mapped_asid = asid;
                self.mapped_vaddr = to_map_vaddr;
                break;
            }
            target_pt = pte.get_ppn();
        }
        sfence_vma();
        0
    }
    pub fn page_table_unmap(&mut self, to_unmap_vaddr: usize, top_table_paddr: usize) -> i32 {
        let vpn_page_num: VirtPageNum = to_unmap_vaddr.into();
        let unmap_pt_num: PhysPageNum = self.base_paddr.into();
        let unmap_pt_addr: PhysAddr = self.base_paddr.into();
        let mut tmp_pnum: PhysPageNum = top_table_paddr.into();
        let levels = vpn_page_num.levels();
        for (i, level) in levels.iter().enumerate() {
            let pte = &mut tmp_pnum.get_pte_array()[*level];
            if i == levels.len() - 1 {
                return -1;
            }
            if pte.get_ppn() == unmap_pt_num {
                *pte = PTE { bits: 0 };
                clear_memory(unmap_pt_addr.0 as *mut u8, SAFE_OS_PAGE_TABLE_BITS);
                self.mapped_vaddr = 0;
                self.mapped_asid = ASID_INVALID;
                self.mapped_flag = false;
                sfence_vma();
                break;
            }
            tmp_pnum = pte.get_ppn();
        }
        0
    }
    /// configure satp register to bind to current virtual addr space
    pub fn set_vm_root(&self, asid: usize) {
        let satp = SatpT::new(8usize, asid, self.base_paddr >> SAFE_OS_PAGE_TABLE_BITS);
        w_satp(satp.words);
        sfence_vma();
        let satp = 8usize << 60 | self.base_paddr;
        unsafe {
            asm!("csrw satp, {0}", in(reg) satp);
            asm!("sfence.vma");
        }
    }
}
