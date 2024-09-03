//! RFC for #![feature(custom_test_frame_work)] was closed (used in https://os.phil-opp.com)
//! comments on RFC close: https://github.com/rust-osdev/bootloader/issues/366
//! this crate mimic its behavior to allow running tests without std libtest support
//! user may declare test registry at crate level
//! and submit test fn in different modules in a distributed manner
//! unlike std libtest, prototype of test fn is not contrainted to be fn()
//!
//! by introducing this level of flexibility, user is required to provide a runner for each declared registry,
//! which contains test fn with the same prototype
//! one particular scenario where this design is useful is
//! test fn may panic (assert!, panic!, ...), runner can adopt crate specific panic handling logic to catch the panic
//! to resume testing without crashing the entire program
//!
//! # Example
//!
//! ```
//! tester::declare_registry(global: [fn() -> Result<(), &'static str>], runner)
//! pub fn runner(f: fn() -> Result<(), &'static str>) -> bool {
//!     (f)().is_ok()
//! }
//! let (num_passed, failed_cases) = tester::run_all!(global);
//!
//! // in submodule
//! #[kernel_test(global)]
//! fn test() -> Result<(), &'static str>()
//! ```

#![no_std]
use tester_macros;
extern crate alloc;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};
pub use tester_macros::{declare_registry, kernel_test, test_all};

/// fn registering process is completed at compile time
/// test fn unit should be self contained, i.e not allowed to capture variable with non-static lifetime
pub struct TestRegistry<F, R = fn(F) -> bool>
where
    F: Copy + 'static,
    R: Fn(F) -> bool,
{
    pub test_units: &'static [TestUnit<F>],
    pub runner: R,
}

impl<F, R> TestRegistry<F, R>
where
    F: Copy + 'static,
    R: Fn(F) -> bool,
{
    pub fn new(registry: &'static [TestUnit<F>], runner: R) -> Self {
        Self {
            test_units: registry,
            runner,
        }
    }
    /// return:
    ///     number of passed tests
    ///     collections of detailed info of failed cases
    pub fn run_all(&self) -> (usize, Vec<TestFnInfo>) {
        let mut num_passed: usize = 0;
        let failed_cases = self
            .test_units
            .into_iter()
            .filter_map(|unit| {
                if (self.runner)(unit.test_fn) {
                    num_passed += 1;
                    None
                } else {
                    Some(unit.info.clone())
                }
            })
            .collect::<Vec<_>>();
        (num_passed, failed_cases)
    }
}

#[derive(Clone)]
pub struct TestFnInfo {
    pub file: &'static str,
    pub fn_name: &'static str,
    pub line: u32,
    pub column: u32,
}

impl Debug for TestFnInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.write_str(&alloc::format!(
            "{}, in {}:{}",
            self.fn_name,
            self.file,
            self.line
        ))
    }
}

pub struct TestUnit<F> {
    pub test_fn: F,
    pub info: TestFnInfo,
}
