use super::page_config::{CONFIG_PT_LEVELS, PT_INDEX_BITS, SAFE_OS_PAGE_BITS};
use crate::BIT;

pub(super) fn clear_memory(ptr: *mut u8, bits: usize) {
    unsafe {
        core::slice::from_raw_parts_mut(ptr, BIT!(bits)).fill(0);
    }
}

#[inline]
pub(super) fn convert_to_mut_type_ref<T>(addr: usize) -> &'static mut T {
    assert_ne!(addr, 0);
    unsafe { &mut *(addr as *mut T) }
}

#[inline]
pub(super) fn convert_to_option_mut_type_ref<T>(addr: usize) -> Option<&'static mut T> {
    if addr == 0 {
        return None;
    }
    Some(convert_to_mut_type_ref::<T>(addr))
}

#[inline]
pub(super) fn riscv_get_lvl_pgsize_bits(n: usize) -> usize {
    ((PT_INDEX_BITS) * (((CONFIG_PT_LEVELS) - 1) - (n))) + SAFE_OS_PAGE_BITS
}

#[inline]
pub(super) fn riscv_get_lvl_pgsize(n: usize) -> usize {
    BIT!(riscv_get_lvl_pgsize_bits(n))
}
