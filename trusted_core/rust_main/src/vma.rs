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

#[cfg(kernel_test)]
use crate::test_framework::TestResult;

#[cfg(kernel_test)]
pub fn vma_test() -> TestResult {
    let mut testresult = TestResult {
        passed: 0,
        failed: 0,
    };
    if vma_new_test() {
        testresult.passed += 1;
    }
    else {
        testresult.failed += 1;
    }
    if vma_map_test() {
        testresult.passed += 1;
    }
    else {
        testresult.failed += 1;
    }
    if vma_unmap_test() {
        testresult.passed += 1;
    }
    else {
        testresult.failed += 1;
    }
    if vma_copy_from_another_test() {
        testresult.passed += 1;
    }
    else {
        testresult.failed += 1;
    }
    testresult
}

#[cfg(kernel_test)]
pub fn vma_new_test() -> bool {
    println!("VMA::new");
    let vma = VMA::new(VirtAddr(0x1002), VirtAddr(0x1f40), MapType::Identical, MapPerm::R);
    if (vma.range.start != 0x1 as usize) | (vma.range.end != 0x2 as usize) {
        return false;
    }
    if vma.map_type != MapType::Identical {
        return false;
    }
    if vma.perm != MapPerm::R {
        return false;
    }
    println!("pass");
    true
}
#[cfg(kernel_test)]
pub fn vma_map_test() -> bool {
    println!("VMA::map");
    let mut vma = VMA::new(VirtAddr(0x1002), VirtAddr(0x1f40), MapType::Identical, MapPerm::R);
    let mut pagetable = PageTable::new();
    vma.map(&mut pagetable);

    let p = pagetable.translate(1).unwrap();
    let ppn = p.get_ppn().0;
    if ppn != 1 {
        println!("failed");
        return false;
    }

    let mut vma = VMA::new(VirtAddr(0x2002), VirtAddr(0x2f40), MapType::Framed, MapPerm::R);
    let start = vma.range.start;
    let end = vma.range.end;
    let mut ppn: PhysPageNum = PhysPageNum(0);
    for vpn in start..end {
        let layout = Layout::from_size_align(4096usize, 1usize).unwrap();
        let frame = unsafe { PHYS_MEM_ALLOCATOR.alloc(layout) as usize };
        ppn = PhysPageNum(frame / PAGE_SIZE);
        let pte_flags = PTEFlags::from_bits(vma.perm.bits).unwrap();
        pagetable.page_map(vpn, ppn.0, pte_flags);
    }

    let p = pagetable.translate(2).unwrap();
    let pp = p.get_ppn().0;
    if pp != ppn.0 {
        println!("failed");
        return false;
    }

    println!("pass");
    true
}

#[cfg(kernel_test)]
pub fn vma_unmap_test() -> bool {
    println!("VMA::unmap");
    let mut vma_1 = VMA::new(VirtAddr(0x1002), VirtAddr(0x1f40), MapType::Identical, MapPerm::R);
    let mut pagetable = PageTable::new();
    vma_1.map(&mut pagetable);
    let mut vma_2 = VMA::new(VirtAddr(0x2002), VirtAddr(0x2f40), MapType::Framed, MapPerm::R);
    vma_2.map(&mut pagetable);

    vma_1.unmap(&mut pagetable);
    vma_2.unmap(&mut pagetable);

    let p = pagetable.translate(1).unwrap().get_ppn().0;
    let y = pagetable.translate(2).unwrap().get_ppn().0;
    if (p != 0) | (y != 0) {
        println!("failed");
        return false;
    }
    println!("pass");
    true
}

#[cfg(kernel_test)]
pub fn vma_copy_from_another_test() -> bool {
    println!("VMA::copy_from_antoher");
    let vma_1 = VMA::new(VirtAddr(0x1002), VirtAddr(0x1f40), MapType::Identical, MapPerm::R);
    let vma_2 = VMA::copy_from_another(&vma_1);
    if (vma_1.range != vma_2.range) | (vma_1.map_type != vma_2.map_type)
        | (vma_1.perm != vma_2.perm) {
            println!("failed");
            return false;
        }
    println!("pass");
    true
}

