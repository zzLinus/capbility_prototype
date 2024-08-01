use crate::kernel_object::untype::UntypedObj;
use crate::kernel_object::page_table::PageTable;
use crate::capability::alloc::{DefaultKAllocator,KObjAllocator};
use crate::kernel_object::endpoint::{IPCBuffer,Endpoint};
use core::ops::{Deref, DerefMut};
use core::alloc::Layout;
use core::ptr::NonNull;
use alloc::boxed::Box;

pub enum Kobj {
    UntypedObj(KobjInner<UntypedObj>),
    PageTableObj(KobjInner<PageTable>),
    EndPointObj(KobjInner<Endpoint<Box<IPCBuffer>, usize>>),
}

pub struct KobjInner<T, A: KObjAllocator = DefaultKAllocator>(pub NonNull<T>,pub A);

impl<T, A: KObjAllocator> KobjInner<T, A> {
    pub fn into_raw(self) -> *mut T {
        self.0.as_ptr()
    }
}

impl<T, A> Deref for KobjInner<T, A>
where
    A: KObjAllocator,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T, A> DerefMut for KobjInner<T, A>
where
    A: KObjAllocator,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl<T, A> Drop for KobjInner<T, A>
where
    A: KObjAllocator,
{
    fn drop(&mut self) {
        unsafe {
            self.1
                .dealloc(NonNull::cast::<u8>(self.0), Layout::new::<T>())
        }
    }
}
