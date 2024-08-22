#![forbid(unsafe_code)]
#![no_std]
extern crate alloc;
use trusted_kernel::{info, warn};
use trusted_kernel::{trusted_kernel_export, trusted_kernel_invoke};

#[trusted_kernel_export]
pub fn call_mm_from_pm() {
    let virt = 0x0;
    let size = 4096;
    match trusted_kernel_invoke!(mm::mmap(virt: usize, size: usize) -> Result<&'static str, &'static str>)
    {
        Some(feedback) => info!("[pm] Got from mm::mmap {:?}", feedback),
        None => warn!("failed cross crate all"),
    };
}
