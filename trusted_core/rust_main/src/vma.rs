use crate::globalallocator_impl::PHYS_MEM_ALLOCATOR;
use alloc::alloc::Layout;
use core::alloc::GlobalAlloc;
use bitflags::*;
use crate::pagetable::*;
use crate::kmem::PAGE_SIZE;

#[derive(PartialEq)]
pub struct VMA {
    pub range: Range,
    map_type: MapType,
    perm: MapPerm,
}

impl VMA {
    pub fn new(start: VirtAddr, end: VirtAddr, map_type: MapType, perm: MapPerm) -> Self {
        let vpn_start: VirtPageNum = start.align_down();
        let vpn_end: VirtPageNum = end.align_up();
        let map_range = Range {
            start: vpn_start.0,
            end: vpn_end.0,
        };
        Self {
            range: map_range,
            map_type,
            perm,
        }
    }

    pub fn map(&mut self, pagetable: &mut PageTable) {
        let start = self.range.start;
        let end = self.range.end;
        let mut ppn: PhysPageNum;
        for vpn in start..end {
            match self.map_type {
                MapType::Identical => {
                    ppn = PhysPageNum(vpn);
                }
                MapType::Framed => {
                    let layout = Layout::from_size_align(4096usize, 1usize).unwrap();
                    let frame = unsafe { PHYS_MEM_ALLOCATOR.alloc(layout) as usize };
                    ppn = PhysPageNum(frame / PAGE_SIZE);
                }
            }
            let pte_flags = PTEFlags::from_bits(self.perm.bits).unwrap();
            pagetable.page_map(vpn, ppn.0, pte_flags);
        }
    }

    pub fn unmap(&mut self, pagetable: &mut PageTable) {
        let start = self.range.start;
        let end = self.range.end;
        for vpn in start..end {
            match self.map_type {
                MapType::Identical => {
                    pagetable.page_unmap(vpn);
                }
                MapType::Framed => {
                    let ppn = pagetable.translate(vpn).unwrap().get_ppn();
                    let paddr: PhysAddr = ppn.into();
                    let layout = Layout::from_size_align(4096usize, 1usize).unwrap();
                    unsafe { PHYS_MEM_ALLOCATOR.dealloc(paddr.0 as *mut u8, layout) };
                    pagetable.page_unmap(vpn);
                }
            }
        }
    }

    pub fn copy_from_another(vma: &VMA) -> Self {
        let vma_start = vma.range.start;
        let vma_end = vma.range.end;
        let map_range = Range {
            start: vma_start,
            end: vma_end,
        };
        Self {
            range: map_range,
            map_type: vma.map_type,
            perm: vma.perm,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,
    Framed,
}

#[derive(PartialEq)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

bitflags! {
    pub struct MapPerm: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}
