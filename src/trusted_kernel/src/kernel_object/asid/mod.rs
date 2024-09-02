use super::page_table::PageTable;
use super::page_util::{PhysAddr, PTE};
use crate::capability::object::ObjPtr;
use crate::sync::Mutex;
use crate::{BIT, MASK};
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::mem::MaybeUninit;
use core::ptr;

pub const ASID_LOW_BITS: usize = 9;
pub const ASID_HIGH_BITS: usize = 7;
pub const INIT_ASID: usize = 1;
pub const ASID_POOL_NUMS: usize = BIT!(ASID_HIGH_BITS);
pub const ASID_ENTRY_NUMS: usize = BIT!(ASID_LOW_BITS);

/// ASID Pool
/// contains 512(1<<9) pools and each pool maintains 128(1<<7) entries
pub static RISCV_KSASID_TABLE: Mutex<[Option<AsidPoolPtr>; BIT!(ASID_HIGH_BITS)]> =
    Mutex::new([None; BIT!(ASID_HIGH_BITS)]);

#[derive(Copy, Clone)]
struct AsidEntry(*const PTE);
/// # Safety
/// later impl should guarantee that access to the PTE is atmoic
unsafe impl Send for AsidEntry {}
#[derive(Copy, Clone)]
pub struct AsidPoolPtr(*mut AsidPool);
unsafe impl Send for AsidPoolPtr {}
#[derive(Clone)]
pub struct AsidPool {
    pool_array: [AsidEntry; BIT!(ASID_LOW_BITS)],
}
impl Default for AsidPool {
    fn default() -> Self {
        AsidPool {
            pool_array: [AsidEntry(core::ptr::null::<PTE>()); BIT!(ASID_LOW_BITS)],
        }
    }
}
/// # Safety
/// later impl should guarantee that access to the pool_ptr is atmoic
unsafe impl Send for AsidPool {}
impl AsidPool {
    /// higher `ASID_HIGH_BITS` represents index in RISV_KSASID_TABLE
    pub fn get_asid_pool_entry(&mut self) -> usize {
        let mut asid_index = 0;
        // the first entry in the first pool in reserved, which means that asid value `0` is reserved
        while asid_index < BIT!(ASID_LOW_BITS)
            && (asid_index == 0 || self.pool_array[asid_index].0 as usize != 0)
        {
            asid_index += 1;
        }
        asid_index
    }
    /// lower `ASID_LOWER_BITS` represents offset within a pool found with higher bits of asid
    pub fn set_asid_pool_entry(&mut self, asid_index: usize, top_page_table: &mut PageTable) {
        let phyaddr: PhysAddr = top_page_table.base_paddr.into();
        self.pool_array[asid_index >> ASID_LOW_BITS] = AsidEntry(phyaddr.0 as *mut PTE);
        top_page_table.mapped_flag = true;
        top_page_table.mapped_vaddr = 0;
        top_page_table.mapped_asid = asid_index;
    }
}

/// get the index of free asid pool from RISCV_KSASID_TABLE
pub fn get_frist_free_pool_index() -> usize {
    let mut i = 0;
    while RISCV_KSASID_TABLE.lock()[i].is_some() {
        i += 1;
    }
    i
}

/// create an asid pool located at index `asid_pool_index`
pub fn set_asid_pool_by_index(asid_pool_index: usize, asid_pool_addr: usize) {
    RISCV_KSASID_TABLE.lock()[asid_pool_index] = Some(AsidPoolPtr(asid_pool_addr as *mut AsidPool));
}

pub fn delete_asid_pool_by_index(asid_pool_index: usize) {
    RISCV_KSASID_TABLE.lock()[asid_pool_index].take();
}

/// find vspace of a thread by asid
pub fn find_vspace_root_by_asid(asid: usize) -> Option<usize> {
    let asid_pool_ptr = &(RISCV_KSASID_TABLE.lock()[asid >> ASID_LOW_BITS]);
    if let Some(pool_ptr) = asid_pool_ptr {
        unsafe {
            let pool_entry = (*pool_ptr.0).pool_array[asid & MASK!(ASID_LOW_BITS)];
            if pool_entry.0 as usize != 0 {
                Some(pool_entry.0 as usize)
            } else {
                None
            }
        }
    } else {
        None
    }
}
