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
    pub static ref ROOT_CAP_INNER: (Cap, Arc<Mutex<CapInner>>) = {
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
    pub static ref ROOT_SERVER_CAP: (Cap, Arc<Mutex<CdtNode>>) = {
        let cap = &ROOT_CAP_INNER.0;
        let rootcdt = Arc::new(Mutex::new(CdtNode::new(ROOT_CAP_INNER.1.clone())));
        cap.get_inner().unwrap().lock().cdt_node = Arc::downgrade(&rootcdt);
        (cap.clone(), rootcdt)
    };
}

/// fetch reference of associated kernel object of a given Cap
/// we always provide regardlessly, ref mut will fall back to ref for those method only requires shared reference
/// this might be optimized if RwLock is introduced on KObj later
/// try to invoke method or access field from it
/// fails if:
///     weak arc upgrading fail
///     kernel object variants mismatch
macro_rules! kobj {
    (<$cap: ident as $kobj_ty: ident>.$accessor: ident$($may_call: tt)?) => {{
        $cap.get_inner().and_then(|inner| {
            if let $crate::capability::object::KObj::$kobj_ty(ref mut object) = *inner.lock().object.lock() {
                Ok(object.$accessor$($may_call)?)
            }else {
                Err($crate::capability::cap::CapErr::DecodeErr)
            }
        })
    }}

}

/// unchecked version of kobj!, no error propagation invoked
/// if any of the intermediate unwrap fails, kernel will panic
/// user should guarantee that weak can be upgraded and kernel object exactly matches
macro_rules! kobj_unchecked {
    (<$cap: ident as $kobj_ty: ident>.$accessor: ident$($may_call: tt)?) => {{
        if let $crate::capability::object::KObj::$kobj_ty(ref mut object) = *$cap.get_inner().unwrap().lock().object.lock() {
            object.$accessor$($may_call)?
        }else {
            panic!("kernel object type mismatches, queried: {}", stringify!($kobj_ty))
        }
    }}
}

// Proposal
// kobj_set_field!(<cap as Variant>.field_name = val)
// macro_rules! kobj_set_field {}
