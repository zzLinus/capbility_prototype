#![allow(unused_imports)]
pub mod cap;
pub mod cdt;
pub mod object;
pub mod rights;

use crate::kernel_object::Untyped;
use crate::sync::Mutex;
use cap::Cap;
use lazy_static::lazy_static;
use object::KObj;

lazy_static! {
    /// root untyped cap and cdt node, all other kernel objects are retyped from this
    pub static ref ROOT_SERVER: Mutex<Untyped> = {
        // untyped region reserved for root server during booting
        // symbols are provided in trusted_kernel/boot/kernel.ld
        extern "C" {
            fn untyped_start();
            fn untyped_end();
        }
        let root_untyped = Untyped::new(untyped_start as usize, untyped_end as usize);
        Mutex::new(root_untyped)
    };
}
