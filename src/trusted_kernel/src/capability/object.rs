use crate::kernel_object::{Frame, PageTable, Untyped, TCB};
use core::alloc::Layout;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

pub enum KObj {
    Untyped(Untyped),
    TCB(ObjPtr<TCB>),
    Frame(Frame),
    PageTable(PageTable),
}

pub struct ObjPtr<T: ?Sized>(pub NonNull<T>);
impl<T: ?Sized> ObjPtr<T> {
    pub fn new(ptr: NonNull<T>) -> Self {
        Self(ptr)
    }
}

/// # Safety
/// Capability system should guarantee that ObjPtr<T> outlives the cap binds to it
/// i.e the cap associated with a given ObjPtr should be revoked first before wipe out the memory ObjPtr points to
impl<T: ?Sized> Deref for ObjPtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: memory is allocated in retype method, alignment and size are checked
        unsafe { self.0.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for ObjPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: memory is allocated in retype method, alignment and size are checked
        unsafe { self.0.as_mut() }
    }
}
