use super::structs::IPCBuffer;
use crate::capability::alloc::*;
use core::alloc::Layout;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

#[derive(Copy, Clone, Default)]
pub struct Region {
    pub start: usize,
    pub end: usize,
}

pub enum KObj {
    UntypedObj(KObj_inner<UntypedObj>),
    PageTableObj(KObj_inner<PageTableObj>),
    EndPointObj(KObj_inner<EndPointObj<Box<IPCBuffer>, usize>>),
}

pub struct KObj_inner<T, A: KObjAllocator = DefaultKAllocator>(NonNull<T>, A);

impl<T, A: KObjAllocator> KObj_inner<T, A> {
    fn into_raw(self) -> *mut T {
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
pub struct PageTableObj {
    start: usize,
    end: usize,
}

impl PageTableObj {
    pub fn clear(&self) {
        println!("clear this page from {} to {}", self.start, self.end);
    }
}

#[derive(Copy, Clone, Default)]
pub struct UntypedObj {
    pub region: Region,
    used: Region,
    pub inited: bool,
}

impl UntypedObj {
    pub fn retype<T>(&mut self) -> Result<KObj_inner<T>, KObjAllocErr>
    where
        T: Default + Sized,
    {
        let default_allocator = if self.inited {
            DefaultKAllocator::bind(self)
        } else {
            self.inited = true;
            DefaultKAllocator::init_from_scratch(self)
        };
        Self::retype_in::<T, DefaultKAllocator>(default_allocator)
    }

    // allocator passed into should be logically binded to the upper UntypedObj type
    pub fn retype_in<T, A>(allocator: A) -> Result<KObj_inner<T, A>, KObjAllocErr>
    where
        T: Default + Sized,
        A: KObjAllocator,
    {
        let mut free_aligned_slot = allocator.alloc(Layout::new::<T>())?.cast::<T>();
        unsafe {
            // SAFETY: free_aligned_slot is well aligned, taking ref into this is safe
            *free_aligned_slot.as_mut() = T::default();
            Ok(KObj_inner(free_aligned_slot, allocator))
        }
    }

    pub fn new(start: usize, end: usize) -> KObj_inner<UntypedObj> {
        // FIXME: root untype is now live in kernel heap
        let root = Box::into_raw(Box::new(UntypedObj {
            region: Region {
                start: start,
                end: end,
            },
            used: Region {
                start: 0x0,
                end: 0x0,
            },
            inited: false,
        }));

        KObj_inner(NonNull::new(root).unwrap(), DefaultKAllocator::default())
    }
}

pub struct EndPointObj<P, R> {
    callback: fn(P) -> R,
    ipc_buf: Option<Box<IPCBuffer>>,
}

impl<P, R: std::default::Default> Default for EndPointObj<P, R> {
    fn default() -> Self {
        Self {
            callback: |_| Default::default(),
            ipc_buf: None,
        }
    }
}

impl<R> EndPointObj<Box<IPCBuffer>, R> {
    pub fn new(callback: fn(Box<IPCBuffer>) -> R) -> Self {
        Self {
            callback,
            ipc_buf: None,
        }
    }
    pub fn dummy_send(&self) {
        println!("edp dummy send");
    }

    // non-blocking send todo:implement real nb_send which returns a fut
    // pub fn nb_send(mut self, buf_ptr: Box<IPCBuffer>) {
    //     self.ipc_buf = Some(buf_ptr);
    //     IntoCapsule::add_to_job_queue(self);
    // }
    pub fn nb_send(&self, buf_ptr: Box<IPCBuffer>) {
        //let capsule = Self {
        //    callback: self.callback,
        //    ipc_buf: Some(buf_ptr),
        //};
        //// IntoCapsule::add_to_job_queue(capsule);
        //ReturnDataHook::new(IntoCapsule::add_to_job_queue(capsule))
    }

    /// blocking send
    /// in rCore,send will block this thread and schedule->nb_exec immdiately
    pub fn send(mut self, buf_ptr: Box<IPCBuffer>) {
        //self.ipc_buf = Some(buf_ptr);
        //match executor::block_on(IntoCapsule::add_to_job_queue(self)) {
        //    Ok(output) => output,
        //    Err(_) => {
        //        panic!("Failed send:");
        //    }
        //}
    }
}
