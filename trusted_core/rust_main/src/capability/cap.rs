use super::{cdt::CdtNode, object::Kobj};
use super::rights::{Rights};
use alloc::boxed::Box;
use crate::kernel_object::endpoint::{IPCBuffer,Endpoint};
use crate::kernel_object::untype::UntypedObj;
use crate::kernel_object::TCB;
use crate::sync::Mutex;
use crate::println;
use alloc::sync::{Arc, Weak};
use core::ptr;

#[derive(Debug)]
pub enum CapType {
    Untyped,
    EndPoint,
}
impl TryFrom<usize> for CapType {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CapType::Untyped),
            1 => Ok(CapType::EndPoint),
            _ => Err(()),
        }
    }
}

pub enum CapInvLable {
    RETYPE,
    NB_SEND,
}

#[derive(Clone)]
pub struct Cap {
    pub object: Arc<Kobj>,
    pub rights: Rights,
    pub cdt_node: Weak<Mutex<CdtNode>>,
}

static mut G_BUFFER: [u8; 256] = [0; 256];
static mut G_BUFFER2: [u8; 256] = [0; 256];

fn callback1(_: Box<IPCBuffer>) -> usize {
    println!("callback with return gets called");
    10usize
}

impl Cap {
    pub fn decode_capinvok(&mut self, label: CapInvLable, thread: &TCB) {
        match &*self.object {
            Kobj::UntypedObj(x) => match label {
                CapInvLable::RETYPE => {
                    let typ = CapType::try_from(thread.mr.regs[0]).unwrap();
                    println!("mr : {} ,retypeing to {:?}", thread.mr.regs[0], typ);
                    // TODO: allocate real memory for Kobj
                    self.new_cap(typ);
                }
                _ => unreachable!(),
            },

            Kobj::EndPointObj(x) => match label {
                CapInvLable::NB_SEND => {
                    //x.dummy_send();
                }
                _ => unreachable!(),
            },
        }
    }

    fn new_cap(&mut self, typ: CapType) {
        match typ {
            CapType::Untyped => {
                println!("makeing new untype");
                let u;
                unsafe {
                    let ptr = ptr::addr_of_mut!(G_BUFFER2) as *mut Kobj;
                    ptr.write(Kobj::UntypedObj(UntypedObj::new(0x00, 0x7ff)));
                    u = Arc::from_raw(ptr)
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

            CapType::EndPoint => {
                println!("makeing new Endpoint");
                let u;
                unsafe {
                    let ptr = ptr::addr_of_mut!(G_BUFFER2) as *mut Kobj;
                    ptr.write(Kobj::EndPointObj(Endpoint::new(callback1)));
                    u = Arc::from_raw(ptr)
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
        }
    }

    pub fn revoke(&self) {
        for node in &Option::as_ref(&self.cdt_node.upgrade())
            .unwrap()
            .lock()
            .child
        {
            node.lock().revoke();
        }
    }

    pub fn get_new_child(&self) -> Arc<Option<Mutex<Cap>>> {
        Option::as_ref(&self.cdt_node.upgrade())
            .unwrap()
            .lock()
            .child
            .last()
            .unwrap()
            .lock()
            .cap
            .clone()
    }

    pub fn get_root_untpye() -> (Arc<Option<Mutex<Cap>>>, Arc<Mutex<CdtNode>>) {
        println!("this is root!");
        let u;
        unsafe {
            let ptr = ptr::addr_of_mut!(G_BUFFER) as *mut Kobj;
            ptr.write(Kobj::UntypedObj(UntypedObj::new(0x00, 0x7ff)));
            u = Arc::from_raw(ptr)
        };
        let r: Rights = Rights::WRITE | Rights::READ;
        let c = Arc::new(Some(Mutex::new(Cap::new(u, r, Weak::new()))));
        let cdt = Arc::new(Mutex::new(CdtNode::new(c.clone())));
        Option::as_ref(&c).unwrap().lock().cdt_node = Arc::downgrade(&cdt);

        (c, cdt.clone())
    }

    const fn new(object: Arc<Kobj>, rights: Rights, cdt_node: Weak<Mutex<CdtNode>>) -> Cap {
        Cap {
            object,
            rights,
            cdt_node,
        }
    }
}
