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
            .find(|(_, vma_remove)| vma_remove.range.start == start_vpn) {
                vma_remove.unmap(&mut self.pagetable);
                self.space.remove(idx);
            }
    }

    pub fn find_unmap_vma(&mut self, size: usize) -> usize {
        let len = self.space.len();
        let mut start = 0;
        if len == 0 {
            start = 0;
        }
        else if len == 1 {
            if (self.space[0].range.start - 0) >= size {
                start = 0;
            }
            else {
                start = self.space[0].range.end;
            }
        }
        else {
            for i in 1..len+1 {
                if i == len {
                    start = self.space[i-1].range.end;
                    let vpn: VirtPageNum = VirtPageNum(start);
                    let vaddr: VirtAddr = vpn.into();
                    return vaddr.0;
                }
                if i == 1 {
                    if self.space[i-1].range.start >= size {
                        start = 0;
                        let vpn: VirtPageNum = VirtPageNum(start);
                        let vaddr: VirtAddr = vpn.into();
                        return vaddr.0;
                    }
                }
                let unmap_size = self.space[i].range.start - self.space[i-1].range.end;
                if unmap_size >= size {
                    start = self.space[i-1].range.end;
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
        for i in 0..len+1 {
            if i == 0 {
                if vma.range.end <= self.space[0].range.start {
                    index = 0;
                    return index;
                }
            }
            else if i == len {
                if vma.range.start >= self.space[len-1].range.end {
                    index = len;
                    return index;
                }
            }
            else {
                if vma.range.start >= self.space[i-1].range.end && vma.range.end <= self.space[i].range.start {
                    index = i;
                    return index;
                }
            }
        }
        return 0;
    }
}
