#![no_std]
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

// #[warn(dead_code)]
use crate::{
    pagetable::*,
    timer::{CLINT_CMP, CLINT_MTIME},
};
use core::arch::{asm, global_asm};
mod physmemallocator_buddy;
mod physmemallocator_slab;

mod config;
mod kernel_object;
mod scheduler;
mod sync;

#[macro_use]
mod console;
mod syscall;
extern crate alloc;
const UART_BASE: usize = 0x1000_0000;
const UART_END: usize = 0x1000_1000;

use log::{info, warn};

global_asm!(include_str!("link_app.S"));

#[no_mangle]
extern "C" fn eh_personality() {}

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
    abort();
}
#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

extern "C" {
    fn kernel_base();
    fn text_end();
    fn rodata_start();
    fn rodata_end();
    fn data_start();
    fn data_end();
    fn bss_start();
    fn bss_end();
    fn heap_start();
    fn end();
    fn kernel_end();
}

fn vspace_init() -> PageTable {
    info!(
        ".text [{:#x}, {:#x})",
        kernel_base as usize, text_end as usize
    );
    info!(
        ".rodata [{:#x}, {:#x})",
        rodata_start as usize, rodata_end as usize
    );
    info!(
        ".data [{:#x}, {:#x})",
        data_start as usize, data_end as usize
    );
    info!(".bss [{:#x}, {:#x})", bss_start as usize, bss_end as usize);
    info!("heap  [{:#x}, {:#x})", heap_start as usize, end as usize);

    let mut pagetable_kernel = PageTable::new();
    let mut start_temp: VirtPageNum = vpn_align_down(kernel_base as usize);
    let mut end_temp: VirtPageNum = vpn_align_up(text_end as usize);
    for vpn in start_temp.0..end_temp.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::X);
    }

    start_temp = vpn_align_down(rodata_start as usize);
    end_temp = vpn_align_up(rodata_end as usize);
    for vpn in start_temp.0..end_temp.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::W);
    }

    start_temp = vpn_align_down(data_start as usize);
    end_temp = vpn_align_up(data_end as usize);
    for vpn in start_temp.0..end_temp.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::W);
    }

    start_temp = vpn_align_down(bss_start as usize);
    end_temp = vpn_align_up(bss_end as usize);
    for vpn in start_temp.0..end_temp.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::W);
    }

    start_temp = vpn_align_down(heap_start as usize);
    end_temp = vpn_align_up(end as usize);
    for vpn in start_temp.0..end_temp.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::W);
    }

    start_temp = vpn_align_down(end as usize);
    end_temp = vpn_align_up(kernel_end as usize);
    for vpn in start_temp.0..end_temp.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::W);
    }

    start_temp = vpn_align_down(UART_BASE);
    end_temp = vpn_align_up(UART_END);
    for vpn in start_temp.0..end_temp.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::W);
    }

    start_temp = vpn_align_down(CLINT_MTIME);
    pagetable_kernel.page_map(start_temp.0, start_temp.0, PTEFlags::R | PTEFlags::W);
    start_temp = vpn_align_down(CLINT_CMP);
    pagetable_kernel.page_map(start_temp.0, start_temp.0, PTEFlags::R | PTEFlags::W);

    pagetable_kernel
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

#[no_mangle]
/// rust language entry point, C start() jumps here
/// currently pagetable is turned off and it should be activated
/// after PageTable Object is integrated into the kernel
extern "C" fn rust_main() {
    let mut my_uart = uart::Uart::new(UART_BASE);
    my_uart.init();
    globalallocator_impl::init_mm();
    cpu::w_sstatus(cpu::r_sstatus() | cpu::SSTATUS_SIE);
    timer::clint_init();
    console::logger_init();
    info!("safeOS is booting ...");

    scheduler::batch::init_task();
    #[cfg(kernel_test)]
    test_framework::test_main();

    loop {}
}
