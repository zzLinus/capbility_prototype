use crate::capability::alloc::{DefaultKAllocator, KObjAllocErr, KObjAllocator};
use crate::capability::object::KObjInner;
use crate::println;
use core::alloc::Layout;

#[derive(Copy, Clone, Default)]
pub struct Region {
    pub start: usize,
    pub end: usize,
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct UntypedObj {
    pub region: Region,
    pub used: Region,
    pub inited: bool,
}

impl UntypedObj {
    pub fn retype<T>(&mut self) -> Result<KObjInner<T>, KObjAllocErr>
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
    pub fn retype_in<T, A>(allocator: A) -> Result<KObjInner<T, A>, KObjAllocErr>
    where
        T: Default + Sized,
        A: KObjAllocator,
    {
        let mut free_aligned_slot = allocator.alloc(Layout::new::<T>())?.cast::<T>();
        unsafe {
            // SAFETY: free_aligned_slot is well aligned, taking ref into this is safe
            *free_aligned_slot.as_mut() = T::default();
            Ok(KObjInner(free_aligned_slot, allocator))
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
