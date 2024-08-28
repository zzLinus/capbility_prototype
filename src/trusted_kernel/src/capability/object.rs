use crate::capability::alloc::{DefaultKAllocator, KObjAllocator};
use crate::kernel_object::page_table::PageTable;
use crate::kernel_object::untype::UntypedObj;
use core::alloc::Layout;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

pub enum KObj {
    UntypedObj(KObjInner<UntypedObj>),
    PageTableObj(KObjInner<PageTable>),
}

pub struct KObjInner<T, A: KObjAllocator = DefaultKAllocator>(pub NonNull<T>, pub A);

unsafe impl<T: Send, A: KObjAllocator> Send for KObjInner<T, A> {}
unsafe impl<T: Sync, A: KObjAllocator> Sync for KObjInner<T, A> {}

impl<T, A: KObjAllocator> KObjInner<T, A> {
    pub fn into_raw(self) -> *mut T {
        self.0.as_ptr()
    }
}

impl<T, A> Deref for KObjInner<T, A>
where
    A: KObjAllocator,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T, A> DerefMut for KObjInner<T, A>
where
    A: KObjAllocator,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl<T, A> Drop for KObjInner<T, A>
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
