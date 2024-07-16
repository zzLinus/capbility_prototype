use super::cap::Cap;
use super::rights::Rights;
use std::sync::{Arc, Mutex, Weak};

pub struct CdtNode {
    pub cap: Weak<Option<Mutex<Cap>>>,
    pub child: Vec<Weak<CdtNode>>,
}

impl CdtNode {
    pub fn new() -> CdtNode {
        CdtNode {
            cap: Weak::new(),
            child: Vec::new(),
        }
    }

    //fn creat(&mut self, cap: Weak<Option<Cap>>, rights: Rights) -> Arc<CdtNode> {
    //let cdt_node = Arc::new(CdtNode { cap, child: vec![] });
    //self.child.push(Arc::downgrade(&cdt_node));
    //cdt_node
    //}

    pub fn revoke(&mut self) {
        // 删除所有子节点
        //for weak_child in self.child.drain(..) {
        //    if let Some(mut child) = weak_child.upgrade() {
        //        Arc::get_mut(&mut child).unwrap().revoke();
        //    }
        //}
        //// 清除自身的 Cap 引用
        //if let Some(cap) = self.cap.upgrade() {
        //    if let Some(mut cap_ref) = Arc::get_mut(&mut Arc::clone(&cap)) {
        //        cap_ref.take();
        //    }
        //}
        //drop(self);
        // 从父节点中删除自身
    }
}
