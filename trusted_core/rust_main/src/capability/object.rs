use super::alloc::{DefaultKAllocator, KObjAllocator};
use crate::kernel_object::endpoint::IPCBuffer;
use crate::kernel_object::untype::UntypedObj;
use alloc::boxed::Box;

#[derive(PartialEq, Copy, Clone, Eq, Debug)]
pub enum EPState {
    Idle = 0,
    Send = 1,
    Recv = 2,
}


pub enum Kobj {
    UntypedObj(UntypedObj),
    EndPointObj(Endpoint<Box<IPCBuffer>, usize>),
}
