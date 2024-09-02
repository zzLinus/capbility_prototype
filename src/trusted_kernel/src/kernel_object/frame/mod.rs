use super::page_util::clear_memory;
use super::page_util::{PTEFlags, PhysAddr, PhysPageNum, VirtPageNum, PTE};
use crate::cpu::{sfence_vma, w_satp};

use super::untyped::RetypeInit;
use crate::capability::object::{KObj, ObjPtr};

pub struct Frame {
    pub root_ppn: PhysPageNum,
}

impl RetypeInit for Frame {
    type StoredAs = [u8; 4096];
    fn retype_init_in(obj_ptr: ObjPtr<Self::StoredAs>) -> KObj {
        let paddr = obj_ptr.as_ptr() as usize;
        let page_size = obj_ptr.len();
        KObj::Frame(Self::new(paddr, page_size))
    }
}

impl Frame {
    pub fn new(paddr: usize, page_size: usize) -> Self {
        Frame {
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
            // if not found, then create
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
