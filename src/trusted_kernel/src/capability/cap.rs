#![deny(clippy::perf, clippy::complexity)]

use super::cdt::CdtNode;
use super::object::KObj;
use super::rights::Rights;
use crate::kernel_object::page_table::PageTable;
use crate::kernel_object::page_util::PhysPageNum;
use crate::kernel_object::untype::UntypedObj;
use crate::kernel_object::TCB;
use crate::println;
use crate::sync::Mutex;
use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use lazy_static::*;

#[derive(Debug)]
pub enum CapType {
    Untyped,
    PageTable,
}
impl TryFrom<usize> for CapType {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CapType::Untyped),
            1 => Ok(CapType::PageTable),
            _ => Err(()),
        }
    }
}

#[allow(non_camel_case_types)]
pub enum CapInvLable {
    RETYPE,
    PG_MAP,
    NB_SEND,
}

lazy_static! {
    static ref BUF: Vec<u8> = vec![0; 1024]; // 1kb
}

#[derive(Clone)]
pub struct Cap {
    pub object: Arc<Mutex<KObj>>,
    pub rights: Rights,
    pub cdt_node: Weak<Mutex<CdtNode>>,
}

impl Cap {
    pub fn decode_capinvok(&mut self, label: CapInvLable, thread: &TCB) {
        match &mut *self.object.lock() {
            KObj::UntypedObj(x) => match label {
                CapInvLable::RETYPE => {
                    let typ = CapType::try_from(thread.ipc_buf.regs[0]).unwrap();
                    println!("mr : {} ,retypeing to {:?}", thread.ipc_buf.regs[0], typ);

                    let u = match typ {
                        CapType::Untyped => Arc::new(Mutex::new(KObj::UntypedObj(
                            x.retype::<UntypedObj>().unwrap(),
                        ))),
                        CapType::PageTable => Arc::new(Mutex::new(KObj::PageTableObj(
                            x.retype::<PageTable>().unwrap(),
                        ))),
                    };

                    let r: Rights = Rights::WRITE | Rights::READ;
                    let c = Arc::new(Some(Mutex::new(Cap::new(u, r, Weak::new()))));
                    let cdt = Arc::new(Mutex::new(CdtNode::new(c.clone())));
                    Option::as_ref(&c).unwrap().lock().cdt_node = Arc::downgrade(&cdt);

                    Option::as_ref(&self.cdt_node.upgrade())
                        .unwrap()
                        .lock()
                        .child
                        .push(cdt);
                }
                _ => unreachable!(),
            },

            KObj::PageTableObj(x) => match label {
                CapInvLable::PG_MAP => {
                    let fake_ppn: PhysPageNum = Default::default();
                    x.page_table_map(0, fake_ppn);
                }
                _ => unreachable!(),
            },
        }
    }

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

    pub fn get_root_untype() -> (Arc<Option<Mutex<Cap>>>, Arc<Mutex<CdtNode>>) {
        let start = BUF.as_ptr() as usize;

        let mut root = UntypedObj::new(start, start + BUF.len());
        let mut tmp_r = root.retype::<UntypedObj>().unwrap();

        *tmp_r = root;

        let u = Arc::new(Mutex::new(KObj::UntypedObj(tmp_r)));

        let r: Rights = Rights::WRITE | Rights::READ;
        let c = Arc::new(Some(Mutex::new(Cap::new(u, r, Weak::new()))));
        let cdt = Arc::new(Mutex::new(CdtNode::new(c.clone())));
        Option::as_ref(&c).unwrap().lock().cdt_node = Arc::downgrade(&cdt);

        (c, cdt.clone())
    }

    const fn new(object: Arc<Mutex<KObj>>, rights: Rights, cdt_node: Weak<Mutex<CdtNode>>) -> Cap {
        Cap {
            object,
            rights,
            cdt_node,
        }
    }
}
