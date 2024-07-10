use crate::pagetable::*;
use crate::vma::*;
use alloc::vec::Vec;

pub struct Vspace {
    pub pagetable: PageTable,
    pub space: Vec<VMA>,
}

impl Vspace {
    pub fn new() -> Self {
        Self {
            pagetable: PageTable::new(),
            space: Vec::new(),
        }
    }

    pub fn map(&mut self, mut vma: VMA) {
        vma.map(&mut self.pagetable);
        let index = self.find_insert_index(&vma);
        self.space.insert(index, vma);
    }

    pub fn unmap(&mut self, vma: VMA) {
        let start_vpn = vma.range.start;
        if let Some((idx, vma_remove)) = self
            .space
            .iter_mut()
            .enumerate()
            .find(|(_, vma_remove)| vma_remove.range.start == start_vpn)
        {
            vma_remove.unmap(&mut self.pagetable);
            self.space.remove(idx);
        }
    }

    pub fn find_unmap_vma(&mut self, size: usize) -> usize {
        let len = self.space.len();
        let mut start = 0;
        if len == 0 {
            start = 0;
        } else if len == 1 {
            if (self.space[0].range.start - 0) >= size {
                start = 0;
            } else {
                start = self.space[0].range.end;
            }
        } else {
            for i in 1..len + 1 {
                if i == len {
                    start = self.space[i - 1].range.end;
                    let vpn: VirtPageNum = VirtPageNum(start);
                    let vaddr: VirtAddr = vpn.into();
                    return vaddr.0;
                }
                if i == 1 {
                    if self.space[i - 1].range.start >= size {
                        start = 0;
                        let vpn: VirtPageNum = VirtPageNum(start);
                        let vaddr: VirtAddr = vpn.into();
                        return vaddr.0;
                    }
                }
                let unmap_size = self.space[i].range.start - self.space[i - 1].range.end;
                if unmap_size >= size {
                    start = self.space[i - 1].range.end;
                    let vpn: VirtPageNum = VirtPageNum(start);
                    let vaddr: VirtAddr = vpn.into();
                    return vaddr.0;
                }
            }
        }
        let vpn: VirtPageNum = VirtPageNum(start);
        let vaddr: VirtAddr = vpn.into();
        vaddr.0
    }

    pub fn copy_from_another(another_vspace: &mut Vspace) -> Vspace {
        let mut vspace = Self::new();
        for vma in another_vspace.space.iter() {
            let new_vma = VMA::copy_from_another(vma);
            vspace.map(new_vma);
            for vpn in vma.range.start..vma.range.end {
                let src_ppn = another_vspace.pagetable.translate(vpn).unwrap().get_ppn();
                let dst_ppn = vspace.pagetable.translate(vpn).unwrap().get_ppn();
                dst_ppn
                    .get_bytes_array()
                    .copy_from_slice(src_ppn.get_bytes_array());
            }
        }
        vspace
    }

    fn find_insert_index(&mut self, vma: &VMA) -> usize {
        let len = self.space.len();
        let mut index = 0;
        if len == 0 {
            return index;
        }
        for i in 0..len + 1 {
            if i == 0 {
                if vma.range.end <= self.space[0].range.start {
                    index = 0;
                    return index;
                }
            } else if i == len {
                if vma.range.start >= self.space[len - 1].range.end {
                    index = len;
                    return index;
                }
            } else {
                if vma.range.start >= self.space[i - 1].range.end
                    && vma.range.end <= self.space[i].range.start
                {
                    index = i;
                    return index;
                }
            }
        }
        return 0;
    }
}

#[cfg(kernel_test)]
use crate::test_framework::TestResult;

#[cfg(kernel_test)]
pub fn vspace_test() -> TestResult {
    let mut testresult = TestResult {
        passed: 0,
        failed: 0,
    };
    if vspace_find_vma_index_test() {
        testresult.passed += 1;
    } else {
        testresult.failed += 1;
    }
    if vspace_map_test() {
        testresult.passed += 1;
    } else {
        testresult.failed += 1;
    }
    if vspace_unmap_test() {
        testresult.passed += 1;
    } else {
        testresult.failed += 1;
    }
    if vspace_find_unmap_vma_test() {
        testresult.passed += 1;
    } else {
        testresult.failed += 1;
    }
    if vspace_copy_from_another_test() {
        testresult.passed += 1;
    } else {
        testresult.failed += 1;
    }
    testresult
}

#[cfg(kernel_test)]
pub fn vspace_find_vma_index_test() -> bool {
    println!("Vspace::find_vma_index");
    let mut vspace = Vspace::new();
    let vma_1 = VMA::new(
        VirtAddr(0x2002),
        VirtAddr(0x2f40),
        MapType::Identical,
        MapPerm::R,
    );
    let mut index = vspace.find_insert_index(&vma_1);
    if index != 0 {
        println!("failed");
        return false;
    }
    vspace.space.push(vma_1);
    let vma_2 = VMA::new(
        VirtAddr(0x1002),
        VirtAddr(0x1f40),
        MapType::Identical,
        MapPerm::R,
    );
    index = vspace.find_insert_index(&vma_2);
    if index != 0 {
        println!("failed");
        return false;
    }
    vspace.space.insert(index, vma_2);
    let vma_3 = VMA::new(
        VirtAddr(0x4002),
        VirtAddr(0x4f40),
        MapType::Identical,
        MapPerm::R,
    );
    index = vspace.find_insert_index(&vma_3);
    if index != 2 {
        println!("failed");
        return false;
    }
    vspace.space.insert(index, vma_3);
    let vma_4 = VMA::new(
        VirtAddr(0x3002),
        VirtAddr(0x3f40),
        MapType::Identical,
        MapPerm::R,
    );
    index = vspace.find_insert_index(&vma_4);
    if index != 2 {
        println!("failed");
        return false;
    }
    println!("pass");
    true
}

#[cfg(kernel_test)]
pub fn vspace_map_test() -> bool {
    println!("Vspace::map");
    let mut vspace = Vspace::new();
    let vma = VMA::new(
        VirtAddr(0x2002),
        VirtAddr(0x2f40),
        MapType::Identical,
        MapPerm::R,
    );
    vspace.map(vma);
    let vma_x = vspace.space.pop().unwrap();
    let range = Range { start: 2, end: 3 };
    if vma_x.range != range {
        println!("failed");
        return false;
    }
    println!("pass");
    true
}

#[cfg(kernel_test)]
pub fn vspace_unmap_test() -> bool {
    println!("Vspace::unmap");
    let mut vspace = Vspace::new();
    let vma = VMA::new(
        VirtAddr(0x2002),
        VirtAddr(0x2f40),
        MapType::Identical,
        MapPerm::R,
    );
    vspace.map(vma);
    let vma_1 = VMA::new(
        VirtAddr(0x2002),
        VirtAddr(0x2f40),
        MapType::Identical,
        MapPerm::R,
    );
    vspace.unmap(vma_1);
    if vspace.space.len() != 0 {
        println!("failed");
        return false;
    }
    println!("pass");
    true
}

#[cfg(kernel_test)]
pub fn vspace_find_unmap_vma_test() -> bool {
    println!("Vspace::find_unmap_vma");
    let mut vspace = Vspace::new();
    let vma = VMA::new(
        VirtAddr(0x2002),
        VirtAddr(0x2f40),
        MapType::Identical,
        MapPerm::R,
    );
    vspace.map(vma);
    let mut vaddr = vspace.find_unmap_vma(1);
    if vaddr != 0x0 {
        println!("failed");
        return false;
    }
    let mut vma_1 = VMA::new(
        VirtAddr(0x0002),
        VirtAddr(0x0f40),
        MapType::Identical,
        MapPerm::R,
    );
    vspace.map(vma_1);
    vaddr = vspace.find_unmap_vma(1);
    if vaddr != 0x1000 {
        println!("failed");
        return false;
    }
    vaddr = vspace.find_unmap_vma(3);
    if vaddr != 0x3000 {
        println!("failed");
        return false;
    }
    vma_1 = VMA::new(
        VirtAddr(0x5002),
        VirtAddr(0x5f40),
        MapType::Identical,
        MapPerm::R,
    );
    vspace.map(vma_1);
    vaddr = vspace.find_unmap_vma(2);
    if vaddr != 0x3000 {
        println!("failed");
        return false;
    }
    println!("pass");
    true
}

#[cfg(kernel_test)]
pub fn vspace_copy_from_another_test() -> bool {
    println!("Vspace::copy_from_another");
    let mut vspace = Vspace::new();
    let vma = VMA::new(
        VirtAddr(0x2002),
        VirtAddr(0x2f40),
        MapType::Framed,
        MapPerm::R,
    );
    vspace.map(vma);
    let vspace_1 = Vspace::copy_from_another(&mut vspace);
    if vspace_1.space != vspace.space {
        println!("failed");
        return false;
    }
    println!("pass");
    true
}
