use super::cdt::CdtNode;
use super::object::{IPCBuffer, Kobj, UntypedObj,EndPointObj};
use super::rights::{self, Rights};
use std::ptr;
use std::sync::{Arc, Mutex, Weak};

pub enum CapType {
    Untyped,
    EndPoint,
}

pub enum CapInvLable {
    RETYPE,
}

#[derive(Clone)]
pub struct Cap {
    pub object: Arc<Kobj>, //对内核对象的引用
    pub rights: Rights,    //标志对于引用的内核对象拥有什么权限
    pub cdt_node: Arc<Mutex<CdtNode>>,
    //cap_info: CapInfo,
}

pub struct CapInvoke {
    cap: Arc<Cap>,
    label: CapInvLable,
}

static mut G_BUFFER: [u8; 256] = [0; 256];

fn callback1(_: Box<IPCBuffer>) -> usize {
    println!("callback with return gets called");
    // let dummy_buf = Box::new(IPCBuffer::default());
    // let ep = Endpoint::new(callback3);
    // ep.nb_send(dummy_buf);
    10usize
}

impl Cap {
    pub fn decode_capinvok(&mut self, label: CapInvLable) {
        match &*self.object {
            Kobj::UntypedObj(x) => match label {
                CapInvLable::RETYPE => x.retype(),
                _ => unreachable!(),
            },

            Kobj::EndPointObj(x) => {
                x.dummy_send();
            }
        }
    }

    pub fn create_new(typ: CapType) -> Arc<Option<Mutex<Cap>>> {
        match typ {
            CapType::Untyped => {
                let u;
                unsafe {
                    let ptr = ptr::addr_of_mut!(G_BUFFER) as *mut Kobj;
                    ptr.write(Kobj::UntypedObj(UntypedObj::new(0x00, 0x7ff)));
                    u = Arc::from_raw(ptr)
                };
                let r: Rights = Rights::WRITE | Rights::READ;
                let cdt = Arc::new(Mutex::new(CdtNode::new()));
                let c = Arc::new(Some(Mutex::new(Cap::new(u, r, cdt.clone()))));
                cdt.lock().expect("REASON").cap = Arc::downgrade(&c.clone());

                c
            }

            CapType::EndPoint => {
                let u;
                unsafe {
                    let ptr = ptr::addr_of_mut!(G_BUFFER) as *mut Kobj;
                    let dummy_buf = Box::new(IPCBuffer::default());
                    ptr.write(Kobj::EndPointObj(EndPointObj::new(callback1)));
                    u = Arc::from_raw(ptr)
                };
                let r: Rights = Rights::WRITE | Rights::READ;
                let cdt = Arc::new(Mutex::new(CdtNode::new()));
                let c = Arc::new(Some(Mutex::new(Cap::new(u, r, cdt.clone()))));
                cdt.lock().expect("REASON").cap = Arc::downgrade(&c.clone());

                c
            }
        }
    }

    const fn new(object: Arc<Kobj>, rights: Rights, cdt_node: Arc<Mutex<CdtNode>>) -> Cap {
        Cap {
            object,
            rights,
            cdt_node,
        }
    }
}
