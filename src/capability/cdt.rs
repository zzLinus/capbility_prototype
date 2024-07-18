use super::cap::Cap;
use std::sync::{Arc, Mutex};

pub struct CdtNode {
    pub cap: Arc<Option<Mutex<Cap>>>,
    pub child: Vec<Arc<Mutex<CdtNode>>>,
}

impl CdtNode {
    pub fn new(c: Arc<Option<Mutex<Cap>>>) -> CdtNode {
        CdtNode {
            cap: c,
            child: Vec::new(),
        }
    }

    pub fn revoke(&mut self) {
        println!("cdt revoke");
        self.child.clear();
    }
}
