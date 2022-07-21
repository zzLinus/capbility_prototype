#![no_std]
#![feature(panic_info_message)]

use core::arch::asm;

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
extern "C"
fn abort() -> ! {
        loop {
                unsafe {
                    asm!("wfi");
                }
        }
}

pub mod uart;
pub mod kmem;
pub mod pagetable;

#[no_mangle]
// rust language entry point, C start() jumps here
extern "C"
fn rust_main() {
    let mut my_uart = uart::Uart::new(0x1000_0000);
    my_uart.init();
    let mut mem = kmem::Kmem::new();
    let memory_set = pagetable::MemorySet::map_kernel(&mut mem);

    println!("safeOS is booting ...");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
