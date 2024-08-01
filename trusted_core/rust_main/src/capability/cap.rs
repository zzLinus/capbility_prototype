#![deny(clippy::perf, clippy::complexity)]

use super::cdt::CdtNode;
use super::object::Kobj;
use super::rights::Rights;
use crate::kernel_object::endpoint::{Endpoint, IPCBuffer};
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

lazy_static! {
    static ref BUF: Vec<u8> = vec![0; 1024]; // 1kb
}

#[derive(Debug)]
pub enum CapType {
    Untyped,
    PageTable,
    EndPoint,
}
impl TryFrom<usize> for CapType {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CapType::Untyped),
            1 => Ok(CapType::PageTable),
            2 => Ok(CapType::EndPoint),
            _ => Err(()),
        }
    }
}

pub enum CapInvLable {
    RETYPE,
    PG_MAP,
    NB_SEND,
}

#[derive(Clone)]
pub struct Cap {
    pub object: Arc<Mutex<Kobj>>,
    pub rights: Rights,
    pub cdt_node: Weak<Mutex<CdtNode>>,
}

impl Cap {
    pub fn decode_capinvok(&mut self, label: CapInvLable, thread: &TCB) {
        match &mut *self.object.lock() {
            Kobj::UntypedObj(x) => match label {
                CapInvLable::RETYPE => {
                    let typ = CapType::try_from(thread.ipc_buf.regs[0]).unwrap();
                    println!("mr : {} ,retypeing to {:?}", thread.ipc_buf.regs[0], typ);

                    let u = match typ {
                        CapType::Untyped => Arc::new(Mutex::new(Kobj::UntypedObj(
                            x.retype::<UntypedObj>().unwrap(),
                        ))),
                        CapType::PageTable => Arc::new(Mutex::new(Kobj::PageTableObj(
                            x.retype::<PageTable>().unwrap(),
                        ))),
                        CapType::EndPoint => {
                            let mut e = x.retype::<Endpoint<Box<IPCBuffer>, usize>>().unwrap();
                            e.ipc_buf = Some(thread.ipc_buf.clone());
                            Arc::new(Mutex::new(Kobj::EndPointObj(e)))
                        }
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

            Kobj::PageTableObj(x) => match label {
                CapInvLable::PG_MAP => {
                    let fake_ppn: PhysPageNum = Default::default();
                    x.page_table_map(0, fake_ppn);
                }
                _ => unreachable!(),
            },

            Kobj::EndPointObj(x) => match label {
                CapInvLable::NB_SEND => {
                    x.clone().send(thread.ipc_buf.clone());
                }
                _ => unreachable!(),
            },
        }
    }

    pub fn revoke(&self) {
        println!("revoke");
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
        println!("this is root!");
        let start = BUF.as_ptr() as usize;

        let mut root = UntypedObj::new(start, start + BUF.len());
        let mut tmp_r = root.retype::<UntypedObj>().unwrap();

        *tmp_r = root;

        let u = Arc::new(Mutex::new(Kobj::UntypedObj(tmp_r)));

        let r: Rights = Rights::WRITE | Rights::READ;
        let c = Arc::new(Some(Mutex::new(Cap::new(u, r, Weak::new()))));
        let cdt = Arc::new(Mutex::new(CdtNode::new(c.clone())));
        Option::as_ref(&c).unwrap().lock().cdt_node = Arc::downgrade(&cdt);

        (c, cdt.clone())
    }

    const fn new(object: Arc<Mutex<Kobj>>, rights: Rights, cdt_node: Weak<Mutex<CdtNode>>) -> Cap {
        Cap {
            object,
            rights,
            cdt_node,
        }
    }
}
