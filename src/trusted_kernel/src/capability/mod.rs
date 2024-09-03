#![allow(unused_imports)]
pub mod cap;
pub mod cdt;
pub mod object;
pub mod rights;

use crate::kernel_object::Untyped;
use alloc::{
    sync::{Arc, Weak},
    vec,
};
use cap::{Cap, CapInner};
use cdt::CdtNode;
use lazy_static::lazy_static;
use object::KObj;
use spin::{lazy, Mutex};