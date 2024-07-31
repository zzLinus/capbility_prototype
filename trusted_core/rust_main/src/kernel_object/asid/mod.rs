use core::mem::MaybeUninit;

use super::page_util::{
    PhysAddr, PTE
};
use crate::{BIT, MASK};

use super::page_table::PageTable;
use crate::sync::Mutex;
use alloc::boxed::Box;

pub const ASID_LOW_BITS: usize = 9;
pub const ASID_HIGH_BITS: usize = 7;
pub const INIT_ASID: usize = 1;

/// ASID Pool
/// contains 512(1<<9) pools and each pool maintains 128(1<<7) entries
pub static RISCV_KSASID_TABLE: [Mutex<Option<AsidPool>>; BIT!(ASID_HIGH_BITS)] =
    [const { Mutex::new(None) }; BIT!(ASID_HIGH_BITS)];

#[derive(Copy, Clone)]
struct AsidEntry(*const PTE);

/// # Safety
/// later impl should guarantee that access to the PTE is atmoic
unsafe impl Send for AsidEntry {}

#[derive(Clone)]
pub struct AsidPool {
    pool_array: Box<[MaybeUninit<AsidEntry>; BIT!(ASID_LOW_BITS)]>,
}

impl AsidPool {
    fn new() -> Self {
        Self {
            pool_array: Box::new([MaybeUninit::zeroed(); BIT!(ASID_LOW_BITS)]),
        }
    }

    /// higher `ASID_HIGH_BITS` represents index in RISV_KSASID_TABLE
    fn get_asid_entry_by_index(&mut self, mut asid_base: usize) -> usize {
        let mut i = 0;
        // the first entry in the first pool in reserved, which means that asid value `0` is reserved
        while i < BIT!(ASID_LOW_BITS)
            && (asid_base + i == 0 || self.pool_array[i].as_ptr().is_null())
        {
            i += 1;
        }
        asid_base += i;
        asid_base
    }

    /// lower `ASID_LOWER_BITS` represents offset within a pool found with higher bits of asid_base
    fn set_asid_entry_by_index(&mut self, asid_base: usize, top_page_table: &mut PageTable) {
        let phyaddr: PhysAddr = top_page_table.root_ppn.into();
        let mut asid_entry: AsidEntry = AsidEntry(0 as *mut PTE);
        asid_entry.0 = phyaddr.0 as *mut PTE;
        self.pool_array[asid_base & MASK!(asid_base)] = MaybeUninit::new(asid_entry);
    }
}

/// create an asid pool located at index `asid_pool_index`
fn set_asid_pool_by_index(asid_pool_index: usize) {
    let mut pool_entry = RISCV_KSASID_TABLE[asid_pool_index].lock();
    *pool_entry = Some(AsidPool::new());
}

fn delete_asid(asid_pool_index: usize) {
    RISCV_KSASID_TABLE[asid_pool_index].lock().take();
}
