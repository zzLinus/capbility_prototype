#![deny(clippy::perf, clippy::complexity)]

use super::cdt::CdtNode;
use super::object::KObj;
use super::rights::Rights;
use crate::kernel_object::{Frame, PageTable, Untyped, TCB};
use crate::sync::Mutex;
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;

#[derive(Clone)]
pub struct Cap {
    pub object: Arc<Mutex<KObj>>,
    pub rights: Rights,
    pub cdt_node: Weak<Mutex<CdtNode>>,
}

impl Cap {
    pub fn revoke(&self) {
        self.cdt_node.upgrade().unwrap().lock().revoke();
    }

    pub fn get_new_child(&self) -> Weak<Option<Mutex<Cap>>> {
        Arc::downgrade(
            &Option::as_ref(&self.cdt_node.upgrade())
                .unwrap()
                .lock()
                .child
                .last()
                .unwrap()
                .lock()
                .cap,
        )
    }

    const fn new(object: Arc<Mutex<KObj>>, rights: Rights, cdt_node: Weak<Mutex<CdtNode>>) -> Cap {
        Cap {
            object,
            rights,
            cdt_node,
        }
    }
}
