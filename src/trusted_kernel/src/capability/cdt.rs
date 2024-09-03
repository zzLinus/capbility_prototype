use super::cap::{Cap, CapInner};
use crate::println;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

pub struct CdtNode {
    pub cap: Arc<Mutex<CapInner>>,
    pub child: Vec<Arc<Mutex<CdtNode>>>,
}

impl CdtNode {
    pub fn new(inner: Arc<Mutex<CapInner>>) -> CdtNode {
        CdtNode {
            cap: inner,
            child: Vec::new(),
        }
    }

    pub fn revoke(&mut self) {
        self.child.clear();
    }
}
