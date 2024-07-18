use super::cdt::CdtNode;
use super::object::*;
use super::rights::{self, Rights};
use super::structs::{IPCBuffer, TCB};
use std::ptr;
use std::sync::{Arc, Mutex, Weak};

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
    PG_CLR,
    NB_SEND,
}

#[derive(Clone)]
pub struct Cap {
    pub object: Arc<Mutex<KObj>>, //对内核对象的引用
    pub rights: Rights,           //标志对于引用的内核对象拥有什么权限
    pub cdt_node: Weak<Mutex<CdtNode>>,
    //cap_info: CapInfo,
}

fn callback1(_: Box<IPCBuffer>) -> usize {
    println!("callback with return gets called");
    // let dummy_buf = Box::new(IPCBuffer::default());
    // let ep = Endpoint::new(callback3);
    // ep.nb_send(dummy_buf);
    10usize
}

impl Cap {
    pub fn decode_capinvok(&mut self, label: CapInvLable, thread: &TCB) {
        match &mut *self.object.lock().unwrap() {
            KObj::UntypedObj(x) => match label {
                CapInvLable::RETYPE => {
                    let typ = CapType::try_from(thread.ipc_buf.mrs[0]).unwrap();
                    println!("mr : {} ,retypeing to {:?}", thread.ipc_buf.mrs[0], typ);
                    // TODO: allocate real memory for Kobj
                    // self.new_cap(typ);
                    let u;

                    match typ {
                        CapType::Untyped => {
                            u = Arc::new(Mutex::new(KObj::UntypedObj(
                                x.retype::<UntypedObj>().unwrap(),
                            )));
                        }
                        CapType::PageTable => {
                            u = Arc::new(Mutex::new(KObj::PageTableObj(
                                x.retype::<PageTableObj>().unwrap(),
                            )));
                        }
                        CapType::EndPoint => {
                            u = Arc::new(Mutex::new(KObj::EndPointObj(
                                x.retype::<EndPointObj<Box<IPCBuffer>, usize>>().unwrap(),
                            )));
                        }
                    }

                    let r: Rights = Rights::WRITE | Rights::READ;
                    let c = Arc::new(Some(Mutex::new(Cap::new(u, r, Weak::new()))));
                    let cdt = Arc::new(Mutex::new(CdtNode::new(c.clone())));
                    Option::as_ref(&c).unwrap().lock().unwrap().cdt_node = Arc::downgrade(&cdt);

                    Option::as_ref(&self.cdt_node.upgrade())
                        .unwrap()
                        .lock()
                        .unwrap()
                        .child
                        .push(cdt);
                }
                _ => unreachable!(),
            },

            KObj::PageTableObj(x) => match label {
                CapInvLable::PG_CLR => x.clear(),
                _ => unreachable!(),
            },

            KObj::EndPointObj(x) => match label {
                CapInvLable::NB_SEND => {
                    x.dummy_send();
                }
                _ => unreachable!(),
            },
        }
    }

    pub fn revoke(&self) {
        for node in &Option::as_ref(&self.cdt_node.upgrade())
            .unwrap()
            .lock()
            .unwrap()
            .child
        {
            node.lock().unwrap().revoke();
        }
    }

    pub fn get_new_child(&self) -> Arc<Option<Mutex<Cap>>> {
        Option::as_ref(&self.cdt_node.upgrade())
            .unwrap()
            .lock()
            .unwrap()
            .child
            .last()
            .unwrap()
            .lock()
            .unwrap()
            .cap
            .clone()
    }

    pub fn get_root_untpye() -> (Arc<Option<Mutex<Cap>>>, Arc<Mutex<CdtNode>>) {
        println!("this is root!");
        let buf = Box::new(vec![0u8; 1024]);
        let start = buf.as_ptr() as usize;

        // FIXME: see UntypeObj::new
        let u = Arc::new(Mutex::new(KObj::UntypedObj(UntypedObj::new(
            start,
            start + buf.len(),
        ))));

        let r: Rights = Rights::WRITE | Rights::READ;
        let c = Arc::new(Some(Mutex::new(Cap::new(u, r, Weak::new()))));
        let cdt = Arc::new(Mutex::new(CdtNode::new(c.clone())));
        Option::as_ref(&c).unwrap().lock().unwrap().cdt_node = Arc::downgrade(&cdt);

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
