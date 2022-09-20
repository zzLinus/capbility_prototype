use crate::kmem::KMEM;
use bitflags::*;
use core::arch::asm;

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
    pub fn new() -> Self {
        let frame = KMEM.lock().palloc(1).unwrap();
        PageTable {
            root_ppn: PhysPageNum(frame / PAGE_SIZE),
        }
    }
    pub fn page_map(&mut self, vpn: usize, ppn: usize, flags: PTEFlags) {
        let pte = self.find_pte_or_create(vpn.into()).unwrap();
        *pte = PTE::new(ppn.into(), flags | PTEFlags::V);
    }
    pub fn page_unmap(&mut self, vpn: usize) {
        let pte = self.find_pte(vpn.into()).unwrap();
        *pte = PTE { bits: 0 };
    }
    pub fn load(&self) {
        let satp = 8usize << 60 | self.root_ppn.0;
        unsafe {
            asm!("csrw satp, {0}", in(reg) satp);
            asm!("sfence.vma");
        }
    }
    fn find_pte_or_create(&mut self, vpn: VirtPageNum) -> Option<&mut PTE> {
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
                let frame = KMEM.lock().palloc(1).unwrap();
                *pte = PTE::new(PhysPageNum(frame / PAGE_SIZE), PTEFlags::V);
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
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PTE, 512) }
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

pub fn vpn_align_down(v: usize) -> VirtPageNum {
    let vaddr: VirtAddr = v.into();
    let vpn: VirtPageNum = vaddr.align_down();
    vpn
}
pub fn vpn_align_up(v: usize) -> VirtPageNum {
    let vaddr: VirtAddr = v.into();
    let vpn: VirtPageNum = vaddr.align_up();
    vpn
}

#[cfg(kernel_test)]
use crate::test_framework::TestResult;

#[cfg(kernel_test)]
pub fn pagetable_test() -> TestResult {
    let mut testresult = TestResult {
        passed: 0,
        failed: 0,
    };

    if pagetable_page_map_test() {
        testresult.passed += 1;
    } else {
        testresult.failed += 1;
    }
    if pagetable_page_unmap_test() {
        testresult.passed += 1;
    } else {
        testresult.failed += 1;
    }

    if pte_new_test() {
        testresult.passed += 1;
    } else {
        testresult.failed += 1;
    }
    if pte_get_ppn_test() {
        testresult.passed += 1;
    } else {
        testresult.failed += 1;
    }
    if pte_get_flags_test() {
        testresult.passed += 1;
    } else {
        testresult.failed += 1;
    }
    if pte_is_valid_test() {
        testresult.passed += 1;
    } else {
        testresult.failed += 1;
    }
    if virt_page_num_levels_test() {
        testresult.passed += 1;
    } else {
        testresult.failed += 1;
    }

    testresult
}

#[cfg(kernel_test)]
pub fn pagetable_page_map_test() -> bool {
    println!("Pagetable::page_map");
    let vpn = 200;
    let mut pagetable = PageTable::new();
    pagetable.page_map(vpn, 200, PTEFlags::V);

    let p = pagetable.find_pte(vpn.into()).unwrap();
    let ppn = p.get_ppn().0;
    if ppn != 200 {
        println!("failed");
        return false;
    }
    println!("pass");
    true
}
#[cfg(kernel_test)]
pub fn pagetable_page_unmap_test() -> bool {
    println!("Pagetable::page_unmap");
    let mut vpn = 1;
    let mut ppn = 0x10000000;
    let mut pagetable = PageTable::new();
    for i in 1..50000 {
        vpn += 1;
        ppn += 2;
        pagetable.page_map(vpn, ppn, PTEFlags::V);
        let p = pagetable.find_pte(vpn.into()).unwrap().get_ppn().0;
        if p != ppn {
            println!("failed");
            return false;
        }
        // println!("{} {:#x} -> {:#x}", i, vpn, ppn);
        // print this in QEMU will not get all data, but if you donot print this, it runs well.

        pagetable.page_unmap(vpn);
        let p = pagetable.find_pte(vpn.into()).unwrap().get_ppn().0;
        if p != 0 {
            println!("failed");
            return false;
        }
    }

    println!("pass");
    true
}

#[cfg(kernel_test)]
pub fn pte_new_test() -> bool {
    println!("PTE::new");
    let p = PTE::new(0x1000.into(), PTEFlags::U);
    if p.bits != 0x1000 << 10 | PTEFlags::U.bits as usize {
        println!("failed");
        return false;
    }
    let p = PTE::new(0x1000.into(), PTEFlags::R);
    if p.bits != 0x1000 << 10 | PTEFlags::R.bits as usize {
        println!("failed");
        return false;
    }

    println!("pass");
    true
}
#[cfg(kernel_test)]
pub fn pte_get_ppn_test() -> bool {
    println!("PTE::get_ppn");
    for i in 1..1024 {
        let pte = PTE { bits: i };
        if pte.get_ppn().0 != 0 {
            println!("failed");
            return false;
        }
    }
    for i in 1..1024 {
        let pte = PTE { bits: i << 10 };
        if pte.get_ppn().0 != i {
            println!("failed");
            return false;
        }
    }
    for i in 1..1024 {
        let pte = PTE { bits: i << 20 };
        if pte.get_ppn().0 != (i << 10) {
            println!("failed");
            return false;
        }
    }
    for i in 1..1024 {
        let pte = PTE { bits: i << 60 };

        if pte.get_ppn().0 != 0 {
            println!("failed");
            return false;
        }
    }
    println!("pass");
    true
}
#[cfg(kernel_test)]
pub fn pte_get_flags_test() -> bool {
    println!("PTE::nget_flags");
    let p = PTE { bits: 1 };
    if p.get_flags() != PTEFlags::V {
        println!("failed");
        return false;
    }
    let p = PTE {
        bits: PTEFlags::R.bits as usize,
    };
    if p.get_flags() != PTEFlags::R {
        println!("failed");
        return false;
    }
    let p = PTE {
        bits: PTEFlags::W.bits as usize,
    };
    if p.get_flags() != PTEFlags::W {
        println!("failed");
        return false;
    }
    let p = PTE {
        bits: PTEFlags::X.bits as usize,
    };
    if p.get_flags() != PTEFlags::X {
        println!("failed");
        return false;
    }
    let p = PTE {
        bits: PTEFlags::U.bits as usize,
    };
    if p.get_flags() != PTEFlags::U {
        println!("failed");
        return false;
    }
    let p = PTE {
        bits: PTEFlags::G.bits as usize,
    };
    if p.get_flags() != PTEFlags::G {
        println!("failed");
        return false;
    }
    let p = PTE {
        bits: PTEFlags::A.bits as usize,
    };
    if p.get_flags() != PTEFlags::A {
        println!("failed");
        return false;
    }
    let p = PTE {
        bits: PTEFlags::D.bits as usize,
    };
    if p.get_flags() != PTEFlags::D {
        println!("failed");
        return false;
    }

    println!("pass");
    true
}
#[cfg(kernel_test)]
pub fn pte_is_valid_test() -> bool {
    println!("PTE::is_valid");
    let p = PTE { bits: 1 };
    if !p.is_valid() {
        println!("failed");
        return false;
    }
    let p = PTE { bits: 0 };
    if p.is_valid() {
        println!("failed");
        return false;
    }
    println!("pass");
    true
}

#[cfg(kernel_test)]
pub fn virt_page_num_levels_test() -> bool {
    println!("VirtPageNum::levels");
    for i in 1..500 {
        let vpn = VirtPageNum(i | i << 9 | i << 18);
        let levels = vpn.levels();

        if levels[0] != i || levels[1] != i || levels[2] != i {
            println!("failed");
            return false;
        }
    }

    println!("pass");
    true
}
