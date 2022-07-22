use bitflags::*;
use crate::kmem::Kmem;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);
pub const PAGE_SIZE: usize = 0x1000;

const PAGE_SIZE_BITS: usize = 0xc;
const PA_WIDTH_SV39: usize = 56;
const VA_WIDTH_SV39: usize = 39;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

extern "C" {
    fn etext();
    fn sheap();
    fn end();
}

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
    pub fn get_pte_array(&self) -> &'static mut [PTE] {
        let pa: PhysAddr = (*self).into();
        unsafe{ core::slice::from_raw_parts_mut(pa.0 as *mut PTE, 512)}
    }
}

impl VirtPageNum {
    pub fn levels(&self) -> [usize; 3] {
        let mut page = self.0;
        let mut level = [0usize; 3];
        for i in (0..3).rev() {
            level[i] = page & 511;
            page >>= 9;
        }
        level
    }
}

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
    pub fn new(ppn:PhysPageNum, flags: PTEFlags) -> Self {
        PTE{
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    pub fn remove_pte() -> Self {
        PTE{ bits: 0}
    }
    pub fn get_ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    pub fn get_flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        (self.get_flags() & PTEFlags::V) != PTEFlags::empty()
    }
}

pub struct PageTable {
    root_ppn: PhysPageNum,
}

impl PageTable {
    pub fn new(mem:&mut Kmem) -> Self {
        let frame = mem.palloc(PAGE_SIZE).unwrap();
        PageTable {
            root_ppn:(frame / PAGE_SIZE).into(),
        }
    }
    fn find_pte_or_create(&mut self, vpn: VirtPageNum, mem:&mut Kmem) -> Option<&mut PTE>
    {
        let levels = vpn.levels();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PTE> = None;
        for (i, level) in levels.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*level];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid(){
                let frame = mem.palloc(1).unwrap();
                *pte = PTE::new((frame / PAGE_SIZE).into(), PTEFlags::V);
            }
            ppn = pte.get_ppn();
        }
        result
    }
    #[allow(unused)]
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags, mem:&mut Kmem) {
        let pte = self.find_pte_or_create(vpn, mem).unwrap();
        *pte = PTE::new(ppn, flags | PTEFlags::V);
    }
}

pub struct Range
{
    l: VirtPageNum,
    r: VirtPageNum,
}

impl Range
{
    pub fn new(start: VirtPageNum, end: VirtPageNum) -> Self {
        Self { l: start, r: end }
    }
}

pub struct VirtualMaparea {
    vpn_range: Range,
    map_flag: Mapflags,
}

bitflags! {
    pub struct Mapflags: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

impl VirtualMaparea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_flag: Mapflags,
    ) -> Self {
        let start_vpn: VirtPageNum = start_va.align_down();
        let end_vpn: VirtPageNum = end_va.align_up();
        Self {
            vpn_range: Range::new(start_vpn, end_vpn),
            map_flag,
        }
    }
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum, mem:&mut Kmem)
    {
        let ppn: PhysPageNum = PhysPageNum(vpn.0);
        let pte_flags = PTEFlags::from_bits(self.map_flag.bits).unwrap();
        page_table.map(vpn, ppn, pte_flags, mem);
    }
    pub fn map(&mut self, page_table:&mut PageTable, mem:&mut Kmem){
        let start = self.vpn_range.l.0;
        let end = self.vpn_range.r.0;
        for vpn in start..end{
            self.map_one(page_table, vpn.into(), mem);
        }
    }
}

pub struct MemorySet {
    page_table: PageTable,
}

impl MemorySet {
    pub fn new(mem:&mut Kmem) -> Self {
        Self {
            page_table: PageTable::new(mem),
        }
    }
    pub fn push(&mut self, mut map_area: VirtualMaparea, mem:&mut Kmem) {
        map_area.map(&mut self.page_table, mem);
    }
    pub fn map_kernel(mem:&mut Kmem)-> Self {
        let mut memory_set = Self::new(mem);
        println!(".text [{:#x}, {:#x})", 0x80000000 as usize, etext as usize);
        println!(".data [{:#x}, {:#x})", etext as usize, sheap as usize);
        println!("heap  [{:#x}, {:#x})", sheap as usize, end as usize);
        println!("mapping .text section");
        memory_set.push(
            VirtualMaparea::new(
                (0x80000000 as usize).into(),
                (etext as usize).into(),
                Mapflags::R | Mapflags::X,
            ),
            mem,
        );
        println!("mapping kernel data and the heap");
        memory_set.push(
            VirtualMaparea::new(
                (etext as usize).into(),
                (end as usize).into(),
                Mapflags::R | Mapflags::W,
            ),
            mem,
        );
            memory_set
    }
}
