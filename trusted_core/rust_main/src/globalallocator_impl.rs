use crate::mutex::Mutex;
use crate::physmemallocator_buddy::{BuddyAllocator, PhysMemory};
use crate::physmemallocator_slab::{AllocationError, Allocator, SlabPool};
use core::alloc::{GlobalAlloc, Layout};
use core::mem::transmute;
use core::ptr::NonNull;

const BASE_PAGE_SIZE: usize = 4096;
const HUGE_PAGE_SIZE: usize = 2 * 1024 * 1024;
const BASE_ALLOC_SIZE: usize = 2048;
const HUGE_ALLOC_SIZE: usize = 2049;
const MAX_ALLOC_SIZE: usize = 1 << 17;

extern "C" {
    fn heap_start();
    fn heap_end();
}

#[allow(dead_code)]
pub struct PhysMemAllocator {
    slaballocator: Mutex<SlabPool<'static>>,
    buddyallocator: Mutex<BuddyAllocator>,
}

impl PhysMemAllocator {
    pub const fn new() -> PhysMemAllocator {
        Self {
            slaballocator: Mutex::new(SlabPool::new()),
            buddyallocator: Mutex::new(BuddyAllocator::new()),
        }
    }
}

// Crate GlobalAllocator
unsafe impl GlobalAlloc for PhysMemAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut buddyallocator = self.buddyallocator.lock();
        match layout.size() {
            BASE_PAGE_SIZE => {
                let frame: *mut u8 = buddyallocator.allocate(layout);
                frame
            }
            0..=BASE_ALLOC_SIZE => {
                let mut slaballocator = self.slaballocator.lock();
                match slaballocator.allocate(layout) {
                    Ok(ptr) => ptr.as_ptr(),
                    Err(AllocationError::OutOfMemory) => {
                        // If outofmemory, slaballocator needs to request
                        // new frames from the buddyallocator
                        if layout.size() <= BASE_ALLOC_SIZE {
                            let frame = buddyallocator.allocate(layout);
                            slaballocator
                                .refill_base(layout, transmute(frame as usize))
                                .expect("Failed to refill slaballocator")
                        }
                        slaballocator
                            .allocate(layout)
                            .expect("Still filed to allocate")
                            .as_ptr()
                    }
                    Err(AllocationError::InvalidLayout) => {
                        panic!("Invaild layout size")
                    }
                }
            }
            HUGE_ALLOC_SIZE..=MAX_ALLOC_SIZE => {
                let mut slaballocator = self.slaballocator.lock();
                match slaballocator.allocate(layout) {
                    Ok(ptr) => ptr.as_ptr(),
                    Err(AllocationError::OutOfMemory) => {
                        if layout.size() <= MAX_ALLOC_SIZE {
                            let huge_layout =
                                Layout::from_size_align_unchecked(HUGE_PAGE_SIZE, layout.align());
                            let frame = buddyallocator.allocate(huge_layout);
                            if frame.is_null() {
                                panic!("Out of Memory")
                            } else {
                                slaballocator
                                    .refill_large(layout, transmute(frame as usize))
                                    .expect("Failed to refill slaballocator")
                            }
                        }
                        slaballocator
                            .allocate(layout)
                            .expect("Still filed to allocate")
                            .as_ptr()
                    }
                    Err(AllocationError::InvalidLayout) => {
                        panic!("Invaild layout size")
                    }
                }
            }
            _ => {
                let frame = buddyallocator.allocate(layout);
                frame
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut buddyallocator = self.buddyallocator.lock();
        let frame = PhysMemory { base: ptr, size: 0 };
        match layout.size() {
            BASE_PAGE_SIZE => {
                buddyallocator.deallocate(frame, layout);
            }
            0..=MAX_ALLOC_SIZE => {
                if let Some(nptr) = NonNull::new(ptr) {
                    let mut slaballocator = self.slaballocator.lock();
                    slaballocator
                        .deallocate(nptr, layout)
                        .expect("Failed to deallocate");
                } else {
                }
            }
            _ => {
                buddyallocator.deallocate(frame, layout);
            }
        }
    }
}

#[global_allocator]
pub static PHYS_MEM_ALLOCATOR: PhysMemAllocator = PhysMemAllocator::new();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Allocation error, layout = {:?}", layout);
}

pub fn init_mm() {
    let physmem: PhysMemory = PhysMemory {
        base: heap_start as *mut u8,
        size: (heap_end as usize - heap_start as usize),
    };
    unsafe {
        PHYS_MEM_ALLOCATOR
            .buddyallocator
            .lock()
            .init_region(physmem)
    };
}
