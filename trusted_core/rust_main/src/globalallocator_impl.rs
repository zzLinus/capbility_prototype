use crate::physmemallocator_buddy::{PhysMemory, BuddyAllocator};
use crate::mutex::Mutex;
use core::alloc::{GlobalAlloc, Layout};

#[global_allocator]
static ALLOCATOR: Mutex<BuddyAllocator> = Mutex::new(BuddyAllocator::new());

const HEAP_BASE: usize = 0x8005_1520;
const HEAP_SIZE: usize = 0x400_0000;

// Crate GlobalAllocator
unsafe impl GlobalAlloc for Mutex<BuddyAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.lock().allocate(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let block = PhysMemory{base:ptr, size:0};
        self.lock().deallocate(block, layout);
    }
}

pub fn init_heap() {
    unsafe {
        let heap:PhysMemory = PhysMemory{base: HEAP_BASE as *mut u8, size: HEAP_SIZE};
        ALLOCATOR.lock().init_region(heap);
    }
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Physical memory allocation error, layout = {:?}", layout);
}
