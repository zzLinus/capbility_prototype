#![no_std]

extern crate alloc;

use alloc::{boxed::Box, sync::Arc};
use core::mem::transmute;

pub trait GlobalInterface {
    fn name(&self) -> &str;
    fn func(&self) -> usize;
    fn crate_name(&self) -> &str;
}
#[linkme::distributed_slice]
pub static GLOBAL_INTERFACES: [fn() -> Box<dyn GlobalInterface>];

pub unsafe fn get_func<F>(crate_name: &str, func_name: &str) -> Option<&'static F> {
    for f in GLOBAL_INTERFACES.iter() {
        let f = f();
        if f.name() == func_name && f.crate_name() == crate_name {
            return Some(transmute(&f.func()));
        }
    }
    None
}
