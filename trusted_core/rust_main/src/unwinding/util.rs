use gimli::{EndianSlice, NativeEndian, Pointer};

pub type StaticSlice = EndianSlice<'static, NativeEndian>;

pub unsafe fn get_unlimited_slice<'a>(start: *const u8) -> &'a [u8] {
    // Create the largest possible slice for this address.
    let start = start as usize;
    let end = start.saturating_add(isize::MAX as _);
    let len = end - start;
    unsafe { core::slice::from_raw_parts(start as *const _, len) }
}

pub unsafe fn deref_pointer(ptr: Pointer) -> usize {
    match ptr {
        Pointer::Direct(x) => x as _,
        Pointer::Indirect(x) => unsafe { *(x as *const _) },
    }
}

#[allow(non_camel_case_types)]
pub type c_int = i32;
