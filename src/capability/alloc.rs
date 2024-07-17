// `KObjAllocator` is equivalent to unstable core::alloc::Allocator API
// used internally within kernel to allow multiple strategy deployed for untyped allocation

use core::alloc::Layout;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use std::sync::{Arc, Mutex};

use super::object::UntypedObj;

#[derive(Debug)]
pub enum KObjAllocErr {
    AernaOom,
    BlockOom,
}
pub(crate) unsafe trait KObjAllocator {
    fn alloc(&self, layout: Layout) -> Result<NonNull<u8>, KObjAllocErr>;
    // SAFETY: caller should guarantee that ptr to be deallocated is previouly allocated by invoking the same KObjAllocator
    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout);
}

pub struct DefaultKAllocator {
    // struct to be allocated should be fitted into this block size(in byte)
    block_size: usize,
    start: usize,
    end: usize,
    head: Arc<Mutex<usize>>,
}
impl DefaultKAllocator {
    fn bind(untyped_obj: &UntypedObj) -> Self {
        let region = untyped_obj.region;
        let (start, end) = (region.start, region.end);
        // should at least be sizeof(usize), hardcode block_size to be 64 for now
        let block_size = 64;
        let head = Self::build_linked_free_block(start, end, block_size);
        Self {
            start,
            end,
            block_size,
            head: Arc::new(Mutex::new(head)),
        }
    }

    // store link block meta info directly in mem instead of binding to allocator struct
    fn build_linked_free_block(start: usize, end: usize, block_size: usize) -> usize {
        // truncate the unaligned prefix and suffix
        let usize_align = mem::align_of::<usize>();
        let head = Self::find_next_aligned(start, usize_align);
        let mut cur = head;
        while cur < end {
            let next_aligned = Self::find_next_aligned(cur + block_size, usize_align);
            if next_aligned >= end {
                unsafe {
                    // last block is set to point to 0x0 (null ptr)
                    *(cur as *mut usize) = 0x0;
                }
                break;
            }
            unsafe {
                // SAFETY: ptr is in boundary and usize aligned (8 bytes)
                *(cur as *mut usize) = next_aligned;
                cur = next_aligned;
            }
            cur += block_size
        }
        head
    }

    #[inline]
    fn find_next_aligned(addr: usize, align_req: usize) -> usize {
        (addr + align_req - 1) & !(align_req - 1)
    }

    #[inline]
    fn find_prev_aligned(addr: usize, align_req: usize) -> usize {
        addr & !(align_req - 1)
    }
}

unsafe impl KObjAllocator for DefaultKAllocator {
    fn alloc(&self, layout: Layout) -> Result<NonNull<u8>, KObjAllocErr> {
        let mut head = self.head.lock().unwrap();
        if *head == 0x0 {
            return Err(KObjAllocErr::AernaOom);
        }
        let end_of_block = *head + self.block_size;
        let obj_ptr = Self::find_next_aligned(*head, layout.align());
        if obj_ptr > end_of_block {
            Err(KObjAllocErr::BlockOom)
        } else {
            // SAFETY: the invariance of head: usize aligned is guaranteed during the entire (d)alloc process
            let next_block = unsafe { *(*head as *const usize) };
            *head = next_block;
            Ok(NonNull::new(obj_ptr as _).unwrap())
        }
    }

    // layout is not required when dealloc because this allocator maintains a fixed block size as basic unit
    // caller should guarantee that ptr passed in lies in one block
    unsafe fn dealloc(&self, ptr: NonNull<u8>, _: Layout) {
        let ptr = ptr.as_ptr() as usize;
        assert!(
            self.start <= ptr && ptr < self.end,
            "Abort: ptr passed in is not managed by this allocator"
        );
        let start_of_block = Self::find_prev_aligned(ptr, mem::align_of::<usize>());
        // reclaim this block by adding it to the free block list
        let mut head = self.head.lock().unwrap();

        unsafe {
            // SAFETY: start_of_block is usize aligned
            *(start_of_block as *mut usize) = *head;
            *head = start_of_block;
        }
    }
}

pub struct KObj<T, A: KObjAllocator = DefaultKAllocator>(NonNull<T>, A);

impl<T, A: KObjAllocator> KObj<T, A> {
    fn into_raw(self) -> *mut T {
        self.0.as_ptr()
    }
}

impl UntypedObj {
    pub fn retype<T>(&self) -> Result<KObj<T>, KObjAllocErr>
    where
        T: Default + Sized,
    {
        let default_allocator = DefaultKAllocator::bind(self);
        Self::retype_in::<T, DefaultKAllocator>(default_allocator)
    }
    pub fn retype_in<T, A>(allocator: A) -> Result<KObj<T, A>, KObjAllocErr>
    where
        T: Default + Sized,
        A: KObjAllocator,
    {
        let mut free_aligned_slot = allocator.alloc(Layout::new::<T>())?.cast::<T>();
        unsafe {
            // SAFETY: free_aligned_slot is well aligned, taking ref into this is safe
            *free_aligned_slot.as_mut() = T::default();
            Ok(KObj(free_aligned_slot, allocator))
        }
    }
}

impl<T, A> Deref for KObj<T, A>
where
    A: KObjAllocator,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T, A> DerefMut for KObj<T, A>
where
    A: KObjAllocator,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl<T, A> Drop for KObj<T, A>
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

#[cfg(test)]
mod tests {
    use crate::capability::object::UntypedObj;

    #[derive(Default)]
    struct PageTableObj {
        start: usize,
        end: usize,
    }

    impl PageTableObj {
        fn clear(&self) {
            println!("clear this page from {} to {}", self.start, self.end);
        }
    }

    #[test]
    fn test_alloc() {
        let page_size = 4096;
        let buf = vec![0u8; page_size * 4];
        let start = buf.as_ptr() as usize;
        let root_untyped = UntypedObj::new(start, start + buf.len());
        let bunch_of_kobj = (0..144)
            .map(|_| root_untyped.retype::<PageTableObj>().unwrap())
            .collect::<Vec<_>>();
        for pagetable_kobj in &bunch_of_kobj {
            pagetable_kobj.clear()
        }
    }
}
