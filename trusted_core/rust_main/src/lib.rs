#![no_std]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#[warn(dead_code)]
use core::arch::asm;
use crate::pagetable::*;
use crate::kmem::Kmem;
mod heapallocator_buddy;
mod mutex;
extern crate alloc;
const UART_BASE: usize = 0x1000_0000;
const KERNEL_BASE: usize = 0x80000000;
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
    fn etext();
    fn sheap();
    fn end();
}

fn vspace_init(mem:&mut Kmem) -> PageTable {
    println!(".text [{:#x}, {:#x})", KERNEL_BASE, etext as usize);
    println!(".data [{:#x}, {:#x})", etext as usize, sheap as usize);
    println!("heap  [{:#x}, {:#x})", sheap as usize, end as usize);

    let mut pagetable_kernel = PageTable::new(mem);
    let text_start:VirtAddr = KERNEL_BASE.into();
    let text_start_align:VirtPageNum = text_start.align_down();

    let text_end:VirtAddr = (etext as usize).into();
    let text_end_align:VirtPageNum = text_end.align_up();

    let data_start:VirtPageNum = text_end_align;
    let heap_end:VirtAddr = (end as usize).into();
    let heap_end_align:VirtPageNum = heap_end.align_up();

    println!("mapping .text section");
    for vpn in text_start_align.0..text_end_align.0 {
        pagetable_kernel.page_map(vpn, vpn, PTEFlags::R | PTEFlags::X, mem);
    }
    println!("mapping kernel data and the heap");
    for vpn in data_start.0..heap_end_align.0 {
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

#[no_mangle]
// rust language entry point, C start() jumps here
extern "C" fn rust_main() {
    let mut my_uart = uart::Uart::new(UART_BASE);
    my_uart.init();
    let mut mem = Kmem::new();
    let _pagetable_kernel = vspace_init(&mut mem);
    globalallocator_impl::init_heap();
    cpu::w_sstatus(cpu::r_sstatus() | cpu::SSTATUS_SIE);
    timer::clint_init();
    println!("safeOS is booting ...");
    loop {}
}
