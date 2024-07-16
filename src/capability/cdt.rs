use super::cap::Cap;
use super::rights::Rights;
use std::sync::{Arc, Mutex, Weak};

pub struct CdtNode {
    pub cap: Arc<Option<Mutex<Cap>>>,
    pub child: Vec<Arc<Mutex<CdtNode>>>,
}

impl CdtNode {
    pub fn new(c : Arc<Option<Mutex<Cap>>>) -> CdtNode {
        CdtNode {
            cap: c,
            child: Vec::new(),
        }
    }

    pub fn revoke(&mut self) {
        self.child.clear();
    }
}
