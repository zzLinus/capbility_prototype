#![forbid(unsafe_code)]
#![no_std]
extern crate alloc;
use trusted_kernel::trusted_kernel_export;

#[trusted_kernel_export]
pub fn mmap(_virt: usize, _size: usize) -> Result<&'static str, &'static str> {
    Ok("[mm] mmap successful")
}
