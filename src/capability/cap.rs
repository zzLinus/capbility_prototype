use super::cdt::CdtNode;
use super::object::{EndPointObj, Kobj, Region, UntypedObj};
use super::rights::{self, Rights};
use std::boxed;
use std::mem::size_of;
use std::ptr;
use std::sync::{Arc, Weak};

pub enum CapType {
    Untyped,
    EndPoint,
}

pub enum CapInvLable {
    RETYPE,
}

pub struct Cap {
    pub object: Arc<Kobj>, //对内核对象的引用
    pub rights: Rights,    //标志对于引用的内核对象拥有什么权限
    pub cdt_node: Arc<CdtNode>,
    //cap_info: CapInfo,
}

pub struct CapInvoke {
    cap: Arc<Cap>,
    label: CapInvLable,
}

static mut g_buffer: [u8; 256] = [0; 256];

impl Cap {

    pub fn decode_capinvok(&self, label: CapInvLable) {
        match &*self.object {
            Kobj::UntypedObj(x) => match label {
                CapInvLable::RETYPE => x.retype(),
                _ => unreachable!(),
            },

            Kobj::EndPointObj(x) => {
                x.get_queue();
            }
        }
    }


    pub fn create_new(typ: CapType) -> Arc<Cap> {
        match typ {
            CapType::Untyped => {
                let u;
                unsafe {
                    let ptr = ptr::addr_of_mut!(g_buffer) as *mut Kobj;
                    ptr.write(Kobj::UntypedObj(UntypedObj::new(0x00, 0x7ff)));
                    u = Arc::from_raw(ptr)
                };

                let r: Rights = Rights::WRITE | Rights::READ;

                let cdt_n = Arc::new(CdtNode::new());

                Arc::new(Cap::new(u, r, cdt_n))
            }

            CapType::EndPoint => {
                let u = Arc::new(Kobj::EndPointObj(EndPointObj::new(
                    super::object::EPState::Recv,
                )));
                let r: Rights = Rights::WRITE | Rights::READ;

                let cdt_n = Arc::new(CdtNode::new());

                Arc::new(Cap::new(u, r, cdt_n))
            }
        }
    }

    const fn new(object: Arc<Kobj>, rights: Rights, cdt_node: Arc<CdtNode>) -> Cap {
        Cap {
            object,
            rights,
            cdt_node,
        }
    }
}
