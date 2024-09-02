use trusted_kernel::{info, trusted_kernel_invoke, warn};

#[kernel_test(global)]
pub fn test_invoke_mm_from_pm() -> bool {
    match trusted_kernel_invoke!(pm::call_mm_from_pm()) {
        Some(_) => {
            info!("call stack [kernel -> pm -> kernel -> mm] success");
            true
        }
        None => {
            warn!("fail to invoke mm::mmap");
            false
        }
    }
}

// deliberatly fail one test here
#[kernel_test(global)]
pub fn test_invoke_non_exist_api() -> bool {
    trusted_kernel_invoke!(pm::nonexist_api()).is_some()
}
