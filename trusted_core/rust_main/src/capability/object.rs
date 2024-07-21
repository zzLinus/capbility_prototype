#![deny(clippy::perf, clippy::complexity)]

use crate::capability::alloc::*;
use crate::endpoint::{IPCBuffer,Endpoint};
use core::alloc::Layout;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use alloc::boxed::Box;

#[derive(Copy, Clone, Default)]
pub struct Region {
    pub start: usize,
    pub end: usize,
}

pub enum KObj {
    UntypedObj(KObj_inner<UntypedObj>),
    PageTableObj(KObj_inner<PageTableObj>),
    EndPointObj(KObj_inner<Endpoint<Box<IPCBuffer>, usize>>),
}

pub struct KObj_inner<T, A: KObjAllocator = DefaultKAllocator>(NonNull<T>, A);

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

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct UntypedObj {
    pub region: Region,
    pub used: Region,
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
            println!("alloc from {:#x} to {:#x}", self.region.start, self.region.end);
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
    pub fn new(_start: usize, _end: usize) -> Self {
        Self {
            region: Region {
                start: _start,
                end: _end,
            },
            used: Region {
                start: 0x0,
                end: 0x0,
            },
            inited: false,
        }
    }
}
