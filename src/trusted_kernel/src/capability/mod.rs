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



lazy_static! {
    pub static ref ROOT_CAP_INNER: (Cap,Arc<Mutex<CapInner>>) = {
        extern "C" {
            fn untyped_start();
            fn untyped_end();
        }
        let root_untyped = Untyped::new(untyped_start as usize, untyped_end as usize);
        Cap::new(
            Arc::new(Mutex::new(KObj::Untyped(root_untyped))),
            rights::Rights::default(),
            Weak::new(),
        )
    };
    pub static ref ROOT_SERVER_CAP: (Cap,Arc<Mutex<CdtNode>>) ={
        let cap=&ROOT_CAP_INNER.0;
        //ROOT_CDT.get_inner().lock().cdt_node = cap.clone();
        let rootcdt=Arc::new(Mutex::new(CdtNode::new(ROOT_CAP_INNER.1.clone())));
        cap.get_inner().unwrap().lock().cdt_node=Arc::downgrade(&rootcdt);
        (cap.clone(),rootcdt)
    };
}
