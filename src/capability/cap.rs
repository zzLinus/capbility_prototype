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
            1 => Ok(CapType::Untyped),
            _ => Err(()),
        }
    }
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
                    println!("retypeing to {:?}", typ);
                    self.new_cap(typ);
                }
                _ => unreachable!(),
            },

            Kobj::EndPointObj(x) => {
                x.dummy_send();
            }
        }
    }

    fn new_cap(&mut self, typ: CapType) -> Arc<Option<Mutex<Cap>>> {
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
                let cdt = Arc::new(Mutex::new(CdtNode::new()));
                let c = Arc::new(Some(Mutex::new(Cap::new(u, r, cdt.clone()))));
                cdt.lock().expect("REASON").cap = Arc::downgrade(&c);
                self.cdt_node
                    .as_ref()
                    .lock()
                    .unwrap()
                    .child
                    .push(Arc::downgrade(&cdt));

                c
            }

            CapType::EndPoint => {
                let u;
                unsafe {
                    let ptr = ptr::addr_of_mut!(G_BUFFER2) as *mut Kobj;
                    ptr.write(Kobj::EndPointObj(EndPointObj::new(callback1)));
                    u = Arc::from_raw(ptr)
                };
                let r: Rights = Rights::WRITE | Rights::READ;
                let cdt = Arc::new(Mutex::new(CdtNode::new()));
                let c = Arc::new(Some(Mutex::new(Cap::new(u, r, cdt.clone()))));
                cdt.lock().expect("REASON").cap = Arc::downgrade(&c);
                self.cdt_node
                    .as_ref()
                    .lock()
                    .unwrap()
                    .child
                    .push(Arc::downgrade(&cdt));

                c
            }
        }
    }

    pub fn get_new_child(&self) {
        println!("{}", self.cdt_node.as_ref().lock().unwrap().child.len());

        let a = self.cdt_node.as_ref().lock().unwrap().child.last().unwrap().upgrade().is_some();
        println!("{}", a)
    }

    // FIXME: should only create untype at the very beginning
    // and retype then after if needed
    pub fn get_root_untpye() -> Arc<Option<Mutex<Cap>>> {
        println!("this is root!");
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

    const fn new(object: Arc<Kobj>, rights: Rights, cdt_node: Arc<Mutex<CdtNode>>) -> Cap {
        Cap {
            object,
            rights,
            cdt_node,
        }
    }
}
