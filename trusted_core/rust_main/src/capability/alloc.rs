#![deny(clippy::perf, clippy::complexity)]
// `KObjAllocator` is equivalent to unstable core::alloc::Allocator API
// used internally within kernel to allow multiple strategy deployed for untyped allocation

use core::alloc::Layout;
use core::mem;
use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use crate::kernel_object::UntypedObj;
use crate::println;


#[derive(Debug)]
pub enum KObjAllocErr {
    AernaOom,
    BlockOom,
}

/// allocate memory in a region bounded to an UntypedOb
/// # Safety
/// caller should guarantee that ptr to be deallocated is previouly allocated
/// by invoking the same KObjAllocator
pub unsafe trait KObjAllocator {
    fn alloc(&self, layout: Layout) -> Result<NonNull<u8>, KObjAllocErr>;
    unsafe fn dealloc(&self, ptr: NonNull<u8>, layout: Layout);
}

#[derive(Default)]
pub struct DefaultKAllocator {
    // struct to be allocated should be fitted into this block size(in byte)
    block_size: usize,
    // start marks block start point = UntypedObj.start + head offset
    start: usize,
    end: usize,
    head: usize,
}

impl DefaultKAllocator {
    pub fn bind(untyped_obj: &UntypedObj) -> Self {
        let region = untyped_obj.region;
        let (start, end) = (region.start, region.end);
        // should at least be sizeof(usize), hardcode block_size to be 64 for now
        let block_size = 64;
        let head = Self::find_next_aligned(start, mem::align_of::<AtomicUsize>());
        let start = Self::find_next_aligned(
            head + mem::size_of::<AtomicUsize>(),
            mem::align_of::<usize>(),
        );
        Self {
            start,
            end,
            block_size,
            head,
        }
    }

    pub fn init_from_scratch(untyped_obj: &UntypedObj) -> Self {
        let allocator = Self::bind(untyped_obj);
        let first_free_block =
            Self::build_linked_free_block(allocator.start, allocator.end, allocator.block_size);
        unsafe {
            // SAFETY: head is usize aligned
            *(allocator.head as *mut _) = AtomicUsize::new(first_free_block);
        };
        allocator
    }

    // store link block meta info directly in mem instead of binding to allocator struct
    pub fn build_linked_free_block(start: usize, end: usize, block_size: usize) -> usize {
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

// this default allocator is designed to resolve all memory states in place
// does not impose any other state info to be saved as part of UntypedOb
unsafe impl KObjAllocator for DefaultKAllocator {
    fn alloc(&self, layout: Layout) -> Result<NonNull<u8>, KObjAllocErr> {
        // SAFETY: head is aligned so that taking ref is valid, similar op in this func follows through
        let head = unsafe { &*(self.head as *const AtomicUsize) };
        let first_free_block = head.load(Ordering::Acquire);
        if first_free_block == 0x0 {
            return Err(KObjAllocErr::AernaOom);
        }
        let end_of_block = first_free_block + self.block_size;
        let obj_ptr = Self::find_next_aligned(first_free_block, layout.align());
        if obj_ptr + layout.size() > end_of_block {
            Err(KObjAllocErr::BlockOom)
        } else {
            // SAFETY: the invariance of head: usize aligned is guaranteed during the entire (d)alloc process
            let next_block = unsafe { *(first_free_block as *const usize) };
            head.store(next_block, Ordering::Release);
            Ok(NonNull::new(obj_ptr as _).unwrap())
        }
    }

    // layout is not required when dealloc because this allocator maintains a fixed block size as basic unit
    // caller should guarantee that ptr passed in lies in one block
    unsafe fn dealloc(&self, ptr: NonNull<u8>, _: Layout) {
        println!("drop");
        let ptr = ptr.as_ptr() as usize;
        assert!(
            self.start <= ptr && ptr < self.end,
            "Abort: ptr passed in is not managed by this allocator"
        );
        let start_of_block = Self::find_prev_aligned(ptr, mem::align_of::<usize>());
        let head = unsafe { &*(self.head as *const AtomicUsize) };

        // reclaim this block by adding it to the free block list
        unsafe {
            // SAFETY: start_of_block is usize aligned
            *(start_of_block as *mut usize) = head.load(Ordering::Acquire);
            head.store(start_of_block, Ordering::Release);
        }
    }
}
