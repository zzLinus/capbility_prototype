#[warn(dead_code)]
use core::alloc::Layout;
use core::cmp::{max, min};
use core::ptr;
use core::mem::size_of;
use core::num::Wrapping;
extern crate alloc;

const TWO_M_SIZE: usize = 2*1024*1024;
const MIN_HEAP_ALIGN: usize = TWO_M_SIZE;
const MAX_LISTS_NUM: usize = 27;
const MIN_BLOCK_SIZE: usize = 4096;
const MIN_BLOCK_SIZE_LOG2: u8 = 12;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct PhysMemory {
    pub base: *mut u8,
    pub size: usize,
}

pub struct BuddyAllocator {
    region: PhysMemory,
    free_lists: [*mut FreeBlock; MAX_LISTS_NUM],
    min_block_size: usize,
    min_block_size_log2: u8,
}

// Sync crate and Send must be implemented for the HeapAllocator
unsafe impl Send for BuddyAllocator {}
unsafe impl Sync for BuddyAllocator {}

impl BuddyAllocator {
    pub const fn new() -> BuddyAllocator {
        BuddyAllocator {
            region: PhysMemory {
                base: 0 as *mut u8,
                size: 0,
            },
            free_lists: [
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            ],
            min_block_size: MIN_BLOCK_SIZE,
            min_block_size_log2: MIN_BLOCK_SIZE_LOG2,
        }
    }

    pub unsafe fn init_region(&mut self, region: PhysMemory) {
        self.region = region;
        let order = self
            .allocation_order(Layout::from_size_align_unchecked(region.size, 1))
            .expect("Failed to calculate order for root heap block");
        self.free_list_dealloc(order, region.base);
    }

    pub unsafe fn allocate(&mut self, layout: Layout) -> *mut u8 {
        if let Some(order_needed) = self.allocation_order(layout) {
            for order in order_needed..self.free_lists.len() {
                if let Some(block) = self.free_list_alloc(order) {
                    if order > order_needed {
                        self.split_free_block(block, order, order_needed);
                    }
                    return block;
                }
            }
            ptr::null_mut()
        } else {
            ptr::null_mut()
        }
    }

    pub unsafe fn deallocate(&mut self, region: PhysMemory, layout: Layout) {
        let initial_order = self.allocation_order(layout)
            .expect("This is a invalid block");
        let mut block = region.base;
        for order in initial_order..self.free_lists.len() {
            if let Some(buddy) = self.buddy(order, block) {
                if self.free_list_remove(order, buddy) {
                    block = min(block, buddy);
                    continue;
                }
            }
            self.free_list_dealloc(order, block);
            return;
        }
    }

    fn allocation_size(&self, layout: Layout) -> Option<usize> {
        let align = layout.align();
        let mut size = layout.size();
        if !align.multiple_of_2() {
            return None;
        }
        if align > MIN_HEAP_ALIGN {
            return None;
        }
        size = max(size, self.min_block_size);
        size = size.next_power_of_2();
        if size > self.region.size {
            return None;
        }
        Some(size)
    }

    // Calculate order(2^order) and the index `free_lists[order]`.
    fn allocation_order(&self, layout: Layout) -> Option<usize> {
        self.allocation_size(layout)
            .map(|s| {(s.log2_2() - self.min_block_size_log2) as usize })
    }

    fn order_size(&self, order: usize) -> usize {
        1 << (self.min_block_size_log2 as usize + order)
    }

    unsafe fn free_list_alloc(&mut self, order: usize) -> Option<*mut u8> {
        let appropriate_list = self.free_lists[order];
        if !(appropriate_list.is_null()) {
            self.free_lists[order] = (*appropriate_list).next;
            Some(appropriate_list as *mut u8)
        } else {
            None
        }
    }

    unsafe fn free_list_dealloc(&mut self, order: usize, block: *mut u8) {
        let free_block_ptr = block as *mut FreeBlock;
        *free_block_ptr = FreeBlock::new(self.free_lists[order]);
        self.free_lists[order] = free_block_ptr;
    }

    unsafe fn split_free_block(&mut self, block: *mut u8, mut order: usize, order_needed: usize) {
        let mut size_of_splitblock = self.order_size(order);
        while order > order_needed {
            size_of_splitblock >>= 1;
            order -= 1;
            let split = block.offset(size_of_splitblock as isize);
            self.free_list_dealloc(order, split);
        }
    }

    // Find the "buddy" memory block, that is,
    // 1. find the buddy_block having the same size with the input block.
    // 2. their physical addresses are adjacent.
    // 3. they split from the same block.
    unsafe fn buddy(&self, order: usize, block: *mut u8) -> Option<*mut u8> {
        let buddy_block = (block as usize) - (self.region.base as usize);
        let size = self.order_size(order);
        if size >= self.region.size as usize {
            None
        } else {
            Some(self.region.base.offset((buddy_block ^ size) as isize))
        }
    }

    unsafe fn free_list_remove (&mut self, order: usize, block: *mut u8) -> bool {
        let block_ptr = block as *mut FreeBlock;
        let mut trace_ptr: *mut *mut FreeBlock = &mut self.free_lists[order];
        while !((*trace_ptr).is_null()) {
            if *trace_ptr == block_ptr {
                *trace_ptr = (*(*trace_ptr)).next;
                return true;
            }
            trace_ptr = &mut ((*(*trace_ptr)).next);
        }
        false
    }
}

struct FreeBlock {
    next: *mut FreeBlock,
}

impl FreeBlock {
    fn new(next: *mut FreeBlock) -> FreeBlock {
        FreeBlock { next }
    }
}

pub trait CalculateOf2 {
    fn multiple_of_2(self) -> bool;
    fn next_power_of_2(self) -> usize;
    fn log2_2(self) -> u8;
}

impl CalculateOf2 for usize {
    fn multiple_of_2(self) -> bool {
        self !=0 && (self & (self - 1)) == 0
    }
    //Find a power of 2 greater than the input
    fn next_power_of_2(self) -> usize {
        if self == 0 {
            return 1;
        }
        let mut v = Wrapping(self);
        v -= Wrapping(1);
        v = v | (v >> 1);
        v = v | (v >> 2);
        v = v | (v >> 4);
        v = v | (v >> 8);
        v = v | (v >> 16);
        if size_of::<usize>() > 4 {
            v = v | (v >> 32);
        }
        v += Wrapping(1);
        let result = match v { Wrapping(v) => v };
        assert!(result.multiple_of_2());
        assert!(result >= self && self > result >> 1);
        result
    }
    fn log2_2(self) -> u8 {
        let mut temp = self;
        let mut result = 0;
        temp >>= 1;
        while temp != 0 {
            result += 1;
            temp >>= 1;
        }
        result
    }
}
