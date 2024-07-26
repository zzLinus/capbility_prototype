use core::slice;
use core::str;

pub(super) fn sys_write(_: usize, ptr: usize, len: usize) {
    // SAFETY: upper stream user app makes sure `ptr` and `len` form a valid slice
    let str_slice =
        unsafe { str::from_utf8(slice::from_raw_parts(ptr as *const u8, len)).unwrap() };
    crate::print!("{}", str_slice);
}
