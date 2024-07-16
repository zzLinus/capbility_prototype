use super::cdt::CdtNode;
use super::object::{EndPointObj, Kobj, UntypedObj};
use super::rights::{self, Rights};
use super::structs::{IPCBuffer, TCB};
use std::ptr;
use std::sync::{Arc, Mutex, Weak};

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
    pub object: Arc<Kobj>, //对内核对象的引用
    pub rights: Rights,    //标志对于引用的内核对象拥有什么权限
    pub cdt_node: Weak<Mutex<CdtNode>>,
    //cap_info: CapInfo,
}

static mut G_BUFFER: [u8; 256] = [0; 256];
static mut G_BUFFER2: [u8; 256] = [0; 256];

fn callback1(_: Box<IPCBuffer>) -> usize {
    println!("callback with return gets called");
    // let dummy_buf = Box::new(IPCBuffer::default());
    // let ep = Endpoint::new(callback3);
    // ep.nb_send(dummy_buf);
    10usize
}

impl Cap {
    pub fn decode_capinvok(&mut self, label: CapInvLable, thread: &TCB) {
        match &*self.object {
            Kobj::UntypedObj(x) => match label {
                CapInvLable::RETYPE => {
                    let typ = CapType::try_from(thread.ipc_buf.mrs[0]).unwrap();
                    println!("mr : {} ,retypeing to {:?}", thread.ipc_buf.mrs[0], typ);
                    self.new_cap(typ);
                }
                _ => unreachable!(),
            },

            Kobj::EndPointObj(x) => match label {
                CapInvLable::NB_SEND => {
                    x.dummy_send();
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
                Option::as_ref(&c).unwrap().lock().unwrap().cdt_node = Arc::downgrade(&cdt);

                Option::as_ref(&self.cdt_node.upgrade())
                    .unwrap()
                    .lock()
                    .unwrap()
                    .child
                    .push(cdt);
            }

            CapType::EndPoint => {
                println!("makeing new Endpoint");
                let u;
                unsafe {
                    let ptr = ptr::addr_of_mut!(G_BUFFER2) as *mut Kobj;
                    ptr.write(Kobj::EndPointObj(EndPointObj::new(callback1)));
                    u = Arc::from_raw(ptr)
                };
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

    // FIXME: should only create untype at the very beginning
    // and retype then after if needed
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
        Option::as_ref(&c).unwrap().lock().unwrap().cdt_node = Arc::downgrade(&cdt);

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
