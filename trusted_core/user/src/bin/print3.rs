#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;


#[no_mangle]
fn main() -> i32{
    for _ in 0..50 {
        println!("wow")
    }
    0
}