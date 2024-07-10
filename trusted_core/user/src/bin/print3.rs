#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;


#[no_mangle]
fn main() -> i32{
    // user_lib::syscall::syscall(150, [0, 0, 0]);
    for _ in 0..50 {
        println!("wow")
    }
    0
}