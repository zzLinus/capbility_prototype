use core::slice;
use core::str;

pub(super) fn sys_write(_: usize, ptr: usize, len: usize) {
    // SAFETY: points to chunk of consecutive bytes
    let str_slice =
        unsafe { str::from_utf8(slice::from_raw_parts(ptr as *const u8, len)).unwrap() };
    crate::print!("{}", str_slice);
}
