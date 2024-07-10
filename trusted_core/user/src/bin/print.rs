#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

// use user_lib::syscall::syscall;

#[no_mangle]
fn main() -> i32 {
    for i in 0..100 {
        println!("hello {}", i);
    }
    // syscall(150, [8, 8, 8]);
    32
}
