//! we need feature ["unwinder","personality","panic"]to be on by default so i removed
//! these features leave the corresponding code active.
//! feature ["panic-handler","system-alloc","print"] are useless and are deleted.
//! features about the arch and fde finding strategy remained the same

// third party mod, allow clippy
#![allow(clippy::all)]
pub mod abi;
mod arch;
pub mod panic;
pub mod panicking;
mod personality;
mod unwinder;
mod util;

#[cfg(kernel_test)]
pub mod kernel_tests {
    use crate::test_framework::TestResult;
    use crate::{print, println};
    use alloc::boxed::Box;

    struct DroppableValue {
        value: usize,
    }

    impl DroppableValue {
        fn new(value: usize) -> Self {
            DroppableValue { value }
        }
    }

    impl Drop for DroppableValue {
        fn drop(&mut self) {
            println!("value dropped: {}", self.value);
        }
    }
    #[inline(never)]
    pub fn recursive(aim: usize) {
        let _boxed = Box::new(DroppableValue::new(1));
        recursive_layer(1, aim);
    }

    fn recursive_layer(depth: usize, aim: usize) {
        let _boxed = Box::new(DroppableValue::new(depth));
        if depth >= aim {
            panic!("Intentional panic at depth {} to test stack unwinding", aim);
        } else {
            recursive_layer(depth + 1, aim);
        }
    }

    pub fn unwind_test() -> TestResult {
        let _res = panic::catch_unwind(|| recursive(10));
        TestResult {
            passed: 1,
            failed: 0,
        }
    }
}
