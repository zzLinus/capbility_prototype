#![no_std]

#[macro_use]
extern crate tester;
extern crate alloc;
use trusted_kernel::{error, info, trusted_kernel_export};

declare_registry!(global);

/// pull in every sub module containing registered kernel test
mod cross_crate;

#[trusted_kernel_export(name = "entry")]
pub fn run_all_tests() {
    let (num_passed, failed_cases) = test_all!(global);
    info!("{} cases passed", num_passed);
    for failure in failed_cases {
        error!("failed: {:?}", failure);
    }
}
