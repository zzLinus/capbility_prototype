#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(lang_items)]
pub mod syscall;

#[macro_use]
pub mod console;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    extern "C" {
        fn main() -> i32;
    }
    // SAFETY: main is properly defined in other file
    let exit_code = unsafe { main() };
    syscall::sys_exit(exit_code);
    loop {}
}

#[panic_handler]
fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    let err = panic_info.message();
    if let Some(location) = panic_info.location() {
        println!(
            "Panicked at {}:{}, {}",
            location.file(),
            location.line(),
            err
        );
    } else {
        println!("Panicked: {}", err);
    }
    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn eh_personality() {}
#[inline(never)]
#[no_mangle]
pub unsafe extern "C-unwind" fn _Unwind_Resume() {}
