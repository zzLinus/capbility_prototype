#![no_std]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#[warn(dead_code)]
use core::arch::asm;
use crate::pagetable::*;
use crate::kmem::Kmem;
mod physmemallocator_buddy;
mod physmemallocator_slab;
mod mutex;
extern crate alloc;
const UART_BASE: usize = 0x1000_0000;
const UART_END: usize = 0x1000_1000;
// Macros for print
#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let _ = write!(crate::uart::Uart::new(0x1000_0000), $($args)+);
    });
}
#[macro_export]
macro_rules! println
{
    () => ({
        print!("\r\n")
    });
    ($fmt:expr) => ({
        print!(concat!($fmt, "\r\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        print!(concat!($fmt, "\r\n"), $($args)+)
    });
}

#[no_mangle]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Aborting: ");
    if let Some(p) = info.location() {
        println!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        println!("no information available.");
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

fn vspace_init(mem:&mut Kmem) -> PageTable {
    println!(".text [{:#x}, {:#x})", kernel_base as usize, text_end as usize);
    println!(".rodata [{:#x}, {:#x})", rodata_start as usize, rodata_end as usize);
    println!(".data [{:#x}, {:#x})", data_start as usize, data_end as usize);
    println!(".bss [{:#x}, {:#x})", bss_start as usize, bss_end as usize);
    println!("heap  [{:#x}, {:#x})", heap_start as usize, end as usize);

    let mut pagetable_kernel = PageTable::new(mem);
    let mut start_temp:VirtPageNum = vpn_align_down(kernel_base as usize);
    let mut end_temp:VirtPageNum = vpn_align_up(text_end as usize);
    for vpn in start_temp.0..end_temp.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::X, mem);
    }

    start_temp = vpn_align_down(rodata_start as usize);
    end_temp = vpn_align_up(rodata_end as usize);
    for vpn in start_temp.0..end_temp.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::W, mem);
    }

    start_temp = vpn_align_down(data_start as usize);
    end_temp = vpn_align_up(data_end as usize);
    for vpn in start_temp.0..end_temp.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::W, mem);
    }

    start_temp = vpn_align_down(bss_start as usize);
    end_temp = vpn_align_up(bss_end as usize);
    for vpn in start_temp.0..end_temp.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::W, mem);
    }

    start_temp = vpn_align_down(heap_start as usize);
    end_temp = vpn_align_up(end as usize);
    for vpn in start_temp.0..end_temp.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::W, mem);
    }

    start_temp = vpn_align_down(end as usize);
    end_temp = vpn_align_up(kernel_end as usize);
    for vpn in start_temp.0..end_temp.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::W, mem);
    }

    start_temp = vpn_align_down(UART_BASE as usize);
    end_temp = vpn_align_up(UART_END as usize);
    for vpn in start_temp.0..end_temp.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::W, mem);
    }
    pagetable_kernel
}

pub mod cpu;
pub mod ecall;
pub mod timer;
pub mod trap;
pub mod uart;
pub mod kmem;
pub mod pagetable;
pub mod globalallocator_impl;

#[cfg(kernel_test)]
pub mod test_framework;

#[no_mangle]
// rust language entry point, C start() jumps here
extern "C" fn rust_main() {
    let mut my_uart = uart::Uart::new(UART_BASE);
    my_uart.init();
    let mut mem = Kmem::new();
    let pagetable_kernel = vspace_init(&mut mem);
    globalallocator_impl::init_mm();
    cpu::w_sstatus(cpu::r_sstatus() | cpu::SSTATUS_SIE);
    timer::clint_init();
    pagetable_kernel.load();
    println!("safeOS is booting ...");

    #[cfg(kernel_test)]
    test_framework::test_main();
    
    loop {}
}
