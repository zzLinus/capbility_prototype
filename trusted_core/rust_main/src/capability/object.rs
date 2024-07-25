use crate::kernel_object::untype::UntypedObj;
use crate::capability::alloc::{DefaultKAllocator,KObjAllocator};
use crate::kernel_object::endpoint::{IPCBuffer,Endpoint};
use crate::println;
use core::ops::{Deref, DerefMut};
use core::alloc::Layout;
use core::ptr::NonNull;
use alloc::boxed::Box;

pub enum KObj {
    UntypedObj(KObj_inner<UntypedObj>),
    PageTableObj(KObj_inner<PageTableObj>),
    EndPointObj(KObj_inner<Endpoint<Box<IPCBuffer>, usize>>),
}

pub struct KObj_inner<T, A: KObjAllocator = DefaultKAllocator>(pub NonNull<T>,pub A);

impl<T, A: KObjAllocator> KObj_inner<T, A> {
    pub fn into_raw(self) -> *mut T {
        self.0.as_ptr()
    }
}

impl<T, A> Deref for KObj_inner<T, A>
where
    A: KObjAllocator,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T, A> DerefMut for KObj_inner<T, A>
where
    A: KObjAllocator,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl<T, A> Drop for KObj_inner<T, A>
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

#[derive(Default)]
#[repr(C)]
pub struct PageTableObj {
    start: usize,
    end: usize,
}

impl PageTableObj {
    pub fn clear(&self) {
        println!("clear this page from {} to {}", self.start, self.end);
    }
}

