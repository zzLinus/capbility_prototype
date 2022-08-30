#[warn(dead_code)]
extern crate alloc;
use core::cmp::min;
use alloc::alloc::Layout;
use core::mem::{transmute, MaybeUninit, swap};
use core::fmt;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU64, Ordering};
use core::ptr;

const BALANCE_COUNT: usize = 64;
const BASIC_SIZE: usize = 80;
const FOUR_K_SIZE: usize = 4096;

pub struct SlabPool<'a> {
    slabs: [SlabAllocator<'a, PageObject<'a>>; SlabPool::MAX_SLABALLOCATOR],
}

impl<'a> Default for SlabPool<'a> {
    fn default() -> SlabPool<'a> {
        SlabPool {
            slabs: [
                SlabAllocator::new(1 << 3), // 8
                SlabAllocator::new(1 << 4), // 16
                SlabAllocator::new(1 << 5), // 32
                SlabAllocator::new(1 << 6), // 64
                SlabAllocator::new(1 << 7), // 128
                SlabAllocator::new(1 << 8), // 256
                SlabAllocator::new(1 << 9),  // 512
                SlabAllocator::new(1 << 10), // 1024
                SlabAllocator::new(1 << 11), // 2048
                SlabAllocator::new(1 << 12), // 4096
                SlabAllocator::new(1 << 13), // 8192
                SlabAllocator::new(1 << 14), // 16384
                SlabAllocator::new(1 << 15), // 32767
                SlabAllocator::new(1 << 16), // 65536
                SlabAllocator::new(1 << 17), // 131072
            ],
        }
    }
}

impl<'a> SlabPool<'a> {

    const MAX_SLABALLOCATOR: usize = 15;

    pub const fn new() -> SlabPool<'a> {
         SlabPool {
            slabs: [
                SlabAllocator::new(1 << 3), // 8
                SlabAllocator::new(1 << 4), // 16
                SlabAllocator::new(1 << 5), // 32
                SlabAllocator::new(1 << 6), // 64
                SlabAllocator::new(1 << 7), // 128
                SlabAllocator::new(1 << 8), // 256
                SlabAllocator::new(1 << 9),  // 512
                SlabAllocator::new(1 << 10), // 1024
                SlabAllocator::new(1 << 11), // 2048
                SlabAllocator::new(1 << 12), // 4096
                SlabAllocator::new(1 << 13), // 8192
                SlabAllocator::new(1 << 14), // 16384
                SlabAllocator::new(1 << 15), // 32767
                SlabAllocator::new(1 << 16), // 65536
                SlabAllocator::new(1 << 17), // 131072
            ],
        }
    }
    fn get_slab(requested_size: usize) -> Slab {
        match requested_size {
            0..=8 => Slab::Base(0),
            9..=16 => Slab::Base(1),
            17..=32 => Slab::Base(2),
            33..=64 => Slab::Base(3),
            65..=128 => Slab::Base(4),
            129..=256 => Slab::Base(5),
            257..=512 => Slab::Base(6),
            513..=1024 => Slab::Base(7),
            1025..=2048 => Slab::Base(8),
            2049..=4096 => Slab::Base(9),
            4097..=8192 => Slab::Base(10),
            8193..=16384 => Slab::Base(11),
            16385..=32767 => Slab::Base(12),
            32768..=65536 => Slab::Base(13),
            65537..=131_072 => Slab::Base(14),
            _ => Slab::Unsupported,
        }
    }
    // Reclaims empty pages by calling `dealloc` on it and removing it from the
    // empty lists in the SlabAllocator.
    #[allow(unused)]
    fn try_reclaim_base_pages<F>(&mut self, mut reclaim: usize, mut dealloc: F)
    where
        F: Fn(*mut PageObject),
    {
        for i in 0..SlabPool::MAX_SLABALLOCATOR {
            let slab = &mut self.slabs[i];
            let just_reclaimed = slab.try_reclaim_pages(reclaim, &mut dealloc);
            reclaim = reclaim.checked_sub(just_reclaimed).unwrap_or(0);
            if reclaim == 0 {
                break;
            }
        }
    }
}

pub unsafe trait Allocator<'a> {
    fn allocate(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocationError>;
    fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) -> Result<(), AllocationError>;
    unsafe fn refill(
        &mut self,
        layout: Layout,
        new_page: &'a mut PageObject<'a>,
    ) -> Result<(), AllocationError>;
}

unsafe impl<'a> Allocator<'a> for SlabPool<'a> {
    fn allocate(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocationError> {
        match SlabPool::get_slab(layout.size()) {
            Slab::Base(index) => self.slabs[index].allocate(layout),
            Slab::Unsupported => Err(AllocationError::InvalidLayout),
        }
    }

    fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) -> Result<(), AllocationError> {
        match SlabPool::get_slab(layout.size()) {
            Slab::Base(index) => self.slabs[index].deallocate(ptr, layout),
            Slab::Unsupported => Err(AllocationError::InvalidLayout),
        }
    }

    // Refills the SlabAllocator
    unsafe fn refill(&mut self, layout: Layout, new_page: &'a mut PageObject<'a>) -> Result<(), AllocationError> {
        match SlabPool::get_slab(layout.size()) {
            Slab::Base(index) => {
                self.slabs[index].refill(new_page);
                Ok(())
            }
            Slab::Unsupported => Err(AllocationError::InvalidLayout),
        }
    }
}

// Erroe type
#[derive(Debug)]
pub enum AllocationError {
    // Allocator does not have enough memory
    OutOfMemory,
    // Allocator can't deal with the provided size of the Layout
    InvalidLayout,
}

pub struct SlabAllocator <'a, P: AllocablePage> {
    size: usize,
    // track of succeeded allocations
    allocation_count: usize,
    pub per_page_max_obj: usize,
    // List of empty PagesObject (nothing allocated in these)
    pub free_slabs: PageList<'a, P>,
    // List of partially used PageObject (some objects allocated but pages are not full)
    pub partial_slabs: PageList<'a, P>,
    // List of full PagesObject (everything allocated)
    pub full_slabs: PageList<'a, P>,
}

impl <'a, P: AllocablePage> SlabAllocator<'a, P> {
    pub const fn new(size: usize) -> SlabAllocator<'a, P> {
        SlabAllocator {
            size: size,
            allocation_count: 0,
            per_page_max_obj: 0,
            free_slabs: PageList::new(),
            partial_slabs: PageList::new(),
            full_slabs: PageList::new(),
        }
    }

    pub fn allocate(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocationError> {
        let layout = unsafe {
            Layout::from_size_align_unchecked(self.size, layout.align())
        };
        let ptr = {
            let ptr = self.try_allocate_from_pagelist(layout);
            if ptr.is_null() && self.free_slabs.head.is_some() {
                // Re-try allocation in empty_page
                let free_page = self.free_slabs.pop().expect("We checked head.is_some()");
                let ptr = free_page.allocate(layout);
                self.add_partial_slabs(free_page);
                ptr
            } else {
                ptr
            }
        };
        let result = NonNull::new(ptr).ok_or(AllocationError::OutOfMemory);
        result
    }
    // Deallocates a previously allocated `ptr` described by `Layout`
    // May return an error in case an invalid `layout` is provided
    pub fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) -> Result<(), AllocationError> {
        let page = (ptr.as_ptr() as usize) & !(P::SIZE - 1) as usize;
        let slab_page = unsafe { transmute::<usize, &mut P>(page) };
        let layout = unsafe { Layout::from_size_align_unchecked(self.size, layout.align()) };
        let result = slab_page.deallocate(ptr, layout);
        result
    }

    #[allow(unused)]
    pub fn try_reclaim_pages<F>(&mut self, to_reclaim: usize, dealloc: &mut F) -> usize
    where
        F: FnMut(*mut P),
    {
        self.move_page();
        let mut reclaimed = 0;
        while reclaimed < to_reclaim {
            if let Some(page) = self.free_slabs.pop() {
                dealloc(page as *mut P);
                reclaimed += 1;
            } else {
                break;
            }
        }
        reclaimed
    }

    // Add a new PageObject to partial_slabs.
    fn add_partial_slabs(&mut self, new_head: &'a mut P) {
        self.partial_slabs.insert_front(new_head);
    }

    // Add a new PageObject to empty_list.
    fn add_free_slabs(&mut self, new_head: &'a mut P) {
        self.free_slabs.insert_front(new_head);
    }

    fn move_page(&mut self) {
        for slab_page in self.full_slabs.iter_mut() {
            if !slab_page.is_full() {
                self.move_full_to_partial(slab_page);
            }
        }

        for slab_page in self.partial_slabs.iter_mut() {
            if slab_page.is_free(self.per_page_max_obj) {
                self.move_partial_to_free(slab_page);
            }
        }
    }

    fn move_partial_to_free(&mut self, page: &'a mut P) {
        let _page_ptr = page as *const P;
        self.partial_slabs.remove_from_list(page);
        self.free_slabs.insert_front(page);
    }

    fn move_partial_to_full(&mut self, page: &'a mut P) {
        let _page_ptr = page as *const P;
        self.partial_slabs.remove_from_list(page);
        self.full_slabs.insert_front(page);
    }

    fn move_full_to_partial(&mut self, page: &'a mut P) {
        let _page_ptr = page as *const P;
        self.full_slabs.remove_from_list(page);
        self.partial_slabs.insert_front(page);
    }
    // Searches within already allocated slab pages, if no suitable page is found
    // will try to use a page from the empty page list.
    fn try_allocate_from_pagelist(&mut self, sc_layout: Layout) -> *mut u8 {
        for slab_page in self.partial_slabs.iter_mut() {
            let ptr = slab_page.allocate(sc_layout);
            if !ptr.is_null() {
                if slab_page.is_full() {
                    self.move_partial_to_full(slab_page);
                }
                self.allocation_count += 1;
                return ptr;
            } else {
                continue;
            }
        }
        // Periodically balance page-lists
        if self.allocation_count > BALANCE_COUNT {
            self.move_page();
            self.allocation_count = 0;
        }
        ptr::null_mut()
    }
    // Refill the SlabAllocator
    // PageObject needs to be empty
    unsafe fn refill(&mut self, page: &'a mut P) {
        page.bitfield_mut().initialize(self.size, P::SIZE - BASIC_SIZE);
        *page.prev() = Link::none();
        *page.next() = Link::none();
        self.add_free_slabs(page);
    }
}

pub struct PageList<'a, T: AllocablePage> {
    pub head: Option<&'a mut T>,
    // Number of pages in the list.
    pub elements: usize,
}

impl<'a, T: AllocablePage> PageList<'a, T> {
    pub const fn new() -> PageList<'a, T> {
        PageList {
            head: None,
            elements: 0,
        }
    }

    pub fn insert_front<'b>(&'b mut self, mut new_head: &'a mut T) {
        match self.head {
            None => {
                *new_head.prev() = Link::none();
                self.head = Some(new_head);
            }
            Some(ref mut head) => {
                *new_head.prev() = Link::none();
                *head.prev() = Link::some(new_head);
                swap(head, &mut new_head);
                *head.next() = Link::some(new_head);
            }
        }
        self.elements += 1;
    }

    pub fn remove_from_list(&mut self, slab_page: &mut T) {
        unsafe {
            match slab_page.prev().resolve_mut() {
                None => {
                    self.head = slab_page.next().resolve_mut();
                }
                Some(prev) => {
                    *prev.next() = match slab_page.next().resolve_mut() {
                        None => Link::none(),
                        Some(next) => Link::some(next),
                    };
                }
            }
            match slab_page.next().resolve_mut() {
                None => (),
                Some(next) => {
                    *next.prev() = match slab_page.prev().resolve_mut() {
                        None => Link::none(),
                        Some(prev) => Link::some(prev),
                    };
                }
            }
        }
        *slab_page.prev() = Link::none();
        *slab_page.next() = Link::none();
        self.elements -= 1;
    }

    // Removes `slab_page` from the list.
    fn pop<'b>(&'b mut self) -> Option<&'a mut T> {
        match self.head {
            None => None,
            Some(ref mut head) => {
                let head_next = head.next();
                let mut new_head = unsafe { head_next.resolve_mut() };
                swap(&mut self.head, &mut new_head);
                let _ = self.head.as_mut().map(|n| {
                    *n.prev() = Link::none();
                });

                self.elements -= 1;
                new_head.map(|node| {
                    *node.prev() = Link::none();
                    *node.next() = Link::none();
                    node
                })
            }
        }
    }

    fn iter_mut<'b: 'a>(&mut self) -> PageObjectIter<'b, T> {
        let m = match self.head {
            None => Link::none(),
            Some(ref mut m) => Link::some(*m),
        };
        PageObjectIter {
            head: m,
            phantom: core::marker::PhantomData,
        }
    }
}

// Iterate over all the pages inside a slaballocator
struct PageObjectIter<'a, P: AllocablePage> {
    head: Link<P>,
    phantom: core::marker::PhantomData<&'a P>,
}

impl<'a, P: AllocablePage > Iterator for PageObjectIter<'a, P> {

    type Item = &'a mut P;

    fn next(&mut self) -> Option<&'a mut P> {
        unsafe {
            self.head.resolve_mut().map(|next| {
                self.head = match next.next().resolve_mut() {
                    None => Link::none(),
                    Some(ref mut sp) =>Link::some(*sp),
                };
                next
            })
        }
    }
}

#[repr(C)]
pub struct PageObject<'a> {
    // Holds memory objects.
    data: [u8; FOUR_K_SIZE - BASIC_SIZE],
    // Next element in list (used by `PageList`).
    next: Link<PageObject<'a>>,
    prev: Link<PageObject<'a>>,
    // A bit-field to track free/allocated memory within data.
    bitfield: [AtomicU64; 8],
}

unsafe impl<'a> Send for PageObject<'a> {}
unsafe impl<'a> Sync for PageObject<'a> {}

impl<'a> AllocablePage for PageObject<'a> {

    const SIZE: usize = FOUR_K_SIZE;

    fn bitfield(&self) -> &[AtomicU64; 8] {
        &self.bitfield
    }
    fn bitfield_mut(&mut self) -> &mut [AtomicU64; 8] {
        &mut self.bitfield
    }

    fn prev(&mut self) -> &mut Link<Self> {
        &mut self.prev
    }

    fn next(&mut self) -> &mut Link<Self> {
        &mut self.next
    }
}

impl<'a> Default for PageObject<'a> {
    fn default() -> PageObject<'a> {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

impl<'a> fmt::Debug for PageObject<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PageObject")
    }
}

// This trait is used to define a page from
// which objects are allocated in an `SlabAllocator`
pub trait AllocablePage {
    // The total size of the page.
    const SIZE: usize;

    fn bitfield(&self) -> &[AtomicU64; 8];
    fn bitfield_mut(&mut self) -> &mut [AtomicU64; 8];
    fn prev(&mut self) -> &mut Link<Self>
    where
        Self: core::marker::Sized;
    fn next(&mut self) -> &mut Link<Self>
    where
        Self: core::marker::Sized;

    // Tries to find a free block that satisfies requirement.
    fn first_fit(&self, layout: Layout) -> Option<(usize, usize)> {
        let base_addr = (&*self as *const Self as *const u8) as usize;
        self.bitfield().first_fit(base_addr, layout, Self::SIZE)
    }

    // Tries to allocate an object within this page.
    // In case the slab is full, returns a null ptr.
    fn allocate(&mut self, layout: Layout) -> *mut u8 {
        match self.first_fit(layout) {
            Some((index, addr)) => {
                self.bitfield().set_bit(index);
                addr as *mut u8
            }
            None => ptr::null_mut(),
        }
    }

    fn is_full(&self) -> bool {
        self.bitfield().is_full()
    }

    fn is_free(&self, bits: usize) -> bool {
        self.bitfield().all_free(bits)
    }

    // Deallocates a memory object within this page.
    fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) -> Result<(), AllocationError> {
        let page_offset = (ptr.as_ptr() as usize) & (Self::SIZE - 1);
        let index = page_offset / layout.size();
        self.bitfield().clear_bit(index);
        Ok(())
    }
}

enum Slab {
    Base(usize),
    Unsupported,
}

// Link
pub struct Link<T> {
    p: *mut T,
}

impl<T> Default for Link<T> {
    fn default() -> Self {
        Link {
            p: ptr::null_mut()
        }
    }
}

impl<T> Link<T> {
    // Like Option::None for Link
    fn none() -> Link<T> {
        Link {
            p: ptr::null_mut()
        }
    }

    // Like Option::Some for Link
    fn some(n: &mut T) -> Link<T> {
        Link {
            p: n
        }
    }

    unsafe fn resolve_mut<'a>(&mut self) -> Option<&'a mut T> {
        self.p.as_mut()
    }
}

// bitfield
trait Bitfield {
    fn initialize(&mut self, object_size: usize, max_size_buffer: usize);
    fn first_fit(&self, base_addr: usize, layout: Layout, page_size: usize) -> Option<(usize, usize)>;
    fn is_allocated(&self, index: usize) -> bool;
    fn is_full(&self) -> bool;
    fn all_free(&self, bits: usize) -> bool;
    fn set_bit(&self, index: usize);
    fn clear_bit(&self, index: usize);
}

// Implementation of bit operations on u64
impl Bitfield for [AtomicU64] {
    fn initialize(&mut self, object_size: usize, max_size_buffer: usize) {
        for bitmap in self.iter_mut() {
            *bitmap = AtomicU64::new(u64::MAX);
        }
        // Mark actual slots as free
        let bits = min(max_size_buffer / object_size, self.len() * 64);
        for index in 0..bits {
            self.clear_bit(index);
        }
    }

    // Find a free block of memory that satisfies requirement.
    fn first_fit(&self, base_addr: usize, layout: Layout, page_size: usize) -> Option<(usize, usize)> {
        for (base_index, b) in self.iter().enumerate() {
            let bitval = b.load(Ordering::Relaxed);
            if bitval == u64::MAX {
                continue;
            } else {
                let negated = !bitval;
                let first_free = negated.trailing_zeros() as usize;
                let index: usize = base_index * 64 + first_free;
                let offset = index * layout.size();
                let offset_inside_data = offset <= (page_size - BASIC_SIZE - layout.size());
                if !offset_inside_data {
                    return None;
                }
                let addr: usize = base_addr + offset;
                let alignment_ok = addr % layout.align() == 0;
                let block_is_free = bitval & (1 << first_free) == 0;
                if alignment_ok && block_is_free {
                    return Some((index, addr));
                }
            }
        }
        None
    }

    // Check if the bit index is set.
    fn is_allocated(&self, index: usize) -> bool {
        let base_index = index / 64;
        let bit_index = index % 64;
        (self[base_index].load(Ordering::Relaxed) & (1 << bit_index)) > 0
    }

    fn set_bit(&self, index: usize) {
        let base_index = index / 64;
        let bit_index = index % 64;
        self[base_index].fetch_or(1 << bit_index, Ordering::Relaxed);
    }

    fn clear_bit(&self, index: usize) {
        let base_index = index / 64;
        let bit_index = index % 64;
        self[base_index].fetch_and(!(1 << bit_index), Ordering::Relaxed);
    }

    fn is_full(&self) -> bool {
        self.iter()
            .filter(|&x| x.load(Ordering::Relaxed) != u64::MAX)
            .count()
            == 0
    }

    fn all_free(&self, bits: usize) -> bool {
        for (index, bitmap) in self.iter().enumerate() {
            let checking_bit_range = (index * 64, (index + 1) * 64);
            if bits >= checking_bit_range.0 && bits < checking_bit_range.1 {
                let bits_should_be_free = bits - checking_bit_range.0;
                let free_mask = (1 << bits_should_be_free) - 1;
                return (free_mask & bitmap.load(Ordering::Relaxed)) == 0;
            }
            if bitmap.load(Ordering::Relaxed) == 0 {
                continue;
            } else {
                return false;
            }
        }
        true
    }
}
