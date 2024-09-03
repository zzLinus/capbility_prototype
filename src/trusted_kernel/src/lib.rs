#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(internal_features)]
// styling
#![allow(
    clippy::cognitive_complexity,
    clippy::large_enum_variant,
    clippy::empty_loop,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms
)]
// explicitness
#![allow(clippy::missing_transmute_annotations, clippy::useless_conversion)]
#![deny(unused_must_use)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(core_intrinsics)]
#![feature(noop_waker)]
#![feature(lang_items)]
#![allow(unexpected_cfgs)]
#![feature(naked_functions)]

use crate::pagetable::*;
use core::arch::asm;
use core::intrinsics::size_of;
use core::ops::{Deref, DerefMut, Mul};
mod physmemallocator_buddy;
mod physmemallocator_slab;

mod capability;
mod config;
mod crate_mgmt;
mod elf_parser;
mod kernel_object;
mod sync;
mod unwinding;
use alloc::sync::Arc;
use alloc::vec::Vec;
pub use lazy_static::*;

#[macro_use]
mod console;
extern crate alloc;
// allow invoking proc macro within crate
extern crate self as trusted_kernel;

const UART_BASE: usize = 0x1000_0000;
const UART_END: usize = 0x1000_1000;

use alloc::boxed::Box;
use capability::cap::Cap;
use capability::rights::Rights;
pub use kernel_macros::{trusted_kernel_export, trusted_kernel_invoke};
use kernel_object::TCB;
pub use log::{error, info, warn};
use spin::Mutex;
use unwinding::panic::catch_unwind;
// re-export symbols from kernel_object::unwind_point for upper level service cross crate commu
pub use kernel_object::unwind_point::{
    invoke_proxy, ExportedAPIIdentifier, GlobalInterface, API_REGISTRY,
};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    warn!("Aborting: ");
    if let Some(p) = info.location() {
        warn!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        warn!("no information available.");
    }
    let reason = unwinding::panic::begin_panic(Box::new("unwind"));
    error!("unwind returned with code {:?}", reason);
    unreachable!();
}
#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

pub mod cpu;
pub mod ecall;
pub mod globalallocator_impl;
pub mod kmem;
pub mod pagetable;
#[cfg(kernel_test)]
pub mod test_framework;
pub mod timer;
pub mod trap;
pub mod uart;
pub mod vma;


#[allow(unused_imports)]
use kernel_object::{Frame, PageTable, RetypeInit, Untyped};

/// rust language entry point, C start() jumps here
/// currently pagetable is turned off and it should be activated
/// after PageTable Object is integrated into the kernel
#[no_mangle]
pub extern "C" fn rust_main() {
    let mut my_uart = uart::Uart::new(UART_BASE);
    my_uart.init();
    globalallocator_impl::init_mm();
    cpu::w_sstatus(cpu::r_sstatus() | cpu::SSTATUS_SIE);
    timer::clint_init();

    console::logger_init();
    info!("trusted kernel is booting ...");
    #[cfg(feature = "test")]
    {
        trusted_kernel_invoke!(tests::entry()).unwrap()
    }
}
