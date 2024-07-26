extern crate alloc;
use crate::physmemallocator_buddy::CalculateOf2;
use alloc::alloc::Layout;
use core::cmp::min;
use core::fmt;
use core::intrinsics::wrapping_sub;
use core::mem::{swap, transmute, MaybeUninit};
use core::ptr;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU64, Ordering};

const BALANCE_COUNT: usize = 64; // Balance the number of objects in the three slab_lists
const META_DATA_SIZE: usize = 80;
const MIN_BASE_OBJECTS_SIZE: usize = 8;
const MIN_BASE_OBJECTS_POWER: usize = 3;
const MAX_BASE_OBJECTS_SIZE: usize = 2048;
const MIN_LARGE_ALLOC_SIZE: usize = 2049;
const MAX_LARGE_OBJECTS_SIZE: usize = 131072;
const BASE_PAGE_SIZE: usize = 4096;
const BASE_PAGE_POWER: usize = 12;
const LARGE_PAGE_SIZE: usize = 2 * 1024 * 1024;

pub struct SlabPool<'a> {
    base_slabs: [SlabAllocator<'a, PageObject<'a>>; SlabPool::MAX_BASE_SLABALLOCATOR],
    large_slabs: [SlabAllocator<'a, LargePageObject<'a>>; SlabPool::MAX_LARGE_SLABALLOCATOR],
}

impl<'a> Default for SlabPool<'a> {
    fn default() -> SlabPool<'a> {
        SlabPool {
            base_slabs: [
                SlabAllocator::new(1 << 3),  // 8
                SlabAllocator::new(1 << 4),  // 16
                SlabAllocator::new(1 << 5),  // 32
                SlabAllocator::new(1 << 6),  // 64
                SlabAllocator::new(1 << 7),  // 128
                SlabAllocator::new(1 << 8),  // 256
                SlabAllocator::new(1 << 9),  // 512
                SlabAllocator::new(1 << 10), // 1024
                SlabAllocator::new(1 << 11), // 2048
            ],
            large_slabs: [
                SlabAllocator::new(1 << 12), // 4096
                SlabAllocator::new(1 << 13), // 8192
                SlabAllocator::new(1 << 14), // 16384
                SlabAllocator::new(1 << 15), // 32768
                SlabAllocator::new(1 << 16), // 65536
                SlabAllocator::new(1 << 17), // 131072
            ],
        }
    }
}

impl<'a> SlabPool<'a> {
    const MAX_BASE_SLABALLOCATOR: usize = 9;
    const MAX_LARGE_SLABALLOCATOR: usize = 6;

    pub const fn new() -> SlabPool<'a> {
        SlabPool {
            base_slabs: [
                SlabAllocator::new(1 << 3),  // 8
                SlabAllocator::new(1 << 4),  // 16
                SlabAllocator::new(1 << 5),  // 32
                SlabAllocator::new(1 << 6),  // 64
                SlabAllocator::new(1 << 7),  // 128
                SlabAllocator::new(1 << 8),  // 256
                SlabAllocator::new(1 << 9),  // 512
                SlabAllocator::new(1 << 10), // 1024
                SlabAllocator::new(1 << 11), // 2048
            ],
            large_slabs: [
                SlabAllocator::new(1 << 12), // 4096
                SlabAllocator::new(1 << 13), // 8192
                SlabAllocator::new(1 << 14), // 16384
                SlabAllocator::new(1 << 15), // 32768
                SlabAllocator::new(1 << 16), // 65536
                SlabAllocator::new(1 << 16), // 131072
            ],
        }
    }
    fn get_slab(requested_size: usize) -> Slab {
        match requested_size {
            0..=MAX_BASE_OBJECTS_SIZE => {
                if requested_size <= 4 {
                    Slab::Base(0)
                } else {
                    let size = CalculateOf2::next_power_of_2(requested_size);
                    let power_of_2: usize = CalculateOf2::log2_2(size) as usize;
                    let index: usize = wrapping_sub(power_of_2, MIN_BASE_OBJECTS_POWER);

                    Slab::Base(index)
                }
            }
            MIN_LARGE_ALLOC_SIZE..=MAX_LARGE_OBJECTS_SIZE => {
                let size = CalculateOf2::next_power_of_2(requested_size);
                let power_of_2: usize = CalculateOf2::log2_2(size) as usize;
                let index: usize = wrapping_sub(power_of_2, BASE_PAGE_POWER);

                Slab::Large(index)
            }
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
        for i in 0..SlabPool::MAX_BASE_SLABALLOCATOR {
            let slab = &mut self.base_slabs[i];
            let just_reclaimed = slab.try_reclaim_pages(reclaim, &mut dealloc);
            reclaim = reclaim.saturating_sub(just_reclaimed);
            if reclaim == 0 {
                break;
            }
        }
    }
    #[allow(unused)]
    fn try_reclaim_large_pages<F>(&mut self, mut reclaim: usize, mut dealloc: F)
    where
        F: Fn(*mut LargePageObject),
    {
        for i in 0..SlabPool::MAX_LARGE_SLABALLOCATOR {
            let slab = &mut self.large_slabs[i];
            let just_reclaimed = slab.try_reclaim_pages(reclaim, &mut dealloc);
            reclaim = reclaim.saturating_sub(just_reclaimed);
            if reclaim == 0 {
                break;
            }
        }
    }
}

/// # Safety
/// caller should guarantee that &PageObject is valid
pub unsafe trait Allocator<'a> {
    fn allocate(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocationError>;
    fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) -> Result<(), AllocationError>;
    unsafe fn refill_base(
        &mut self,
        layout: Layout,
        new_page: &'a mut PageObject<'a>,
    ) -> Result<(), AllocationError>;
    unsafe fn refill_large(
        &mut self,
        layout: Layout,
        new_page: &'a mut LargePageObject<'a>,
    ) -> Result<(), AllocationError>;
}

unsafe impl<'a> Allocator<'a> for SlabPool<'a> {
    fn allocate(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocationError> {
        match SlabPool::get_slab(layout.size()) {
            Slab::Base(index) => self.base_slabs[index].allocate(layout),
            Slab::Large(index) => self.large_slabs[index].allocate(layout),
            Slab::Unsupported => Err(AllocationError::InvalidLayout),
        }
    }

    fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) -> Result<(), AllocationError> {
        match SlabPool::get_slab(layout.size()) {
            Slab::Base(index) => self.base_slabs[index].deallocate(ptr, layout),
            Slab::Large(index) => self.large_slabs[index].deallocate(ptr, layout),
            Slab::Unsupported => Err(AllocationError::InvalidLayout),
        }
    }

    // Refills the SlabAllocator(include Pageobject)
    unsafe fn refill_base(
        &mut self,
        layout: Layout,
        new_page: &'a mut PageObject<'a>,
    ) -> Result<(), AllocationError> {
        match SlabPool::get_slab(layout.size()) {
            Slab::Base(index) => {
                self.base_slabs[index].refill(new_page);
                Ok(())
            }
            Slab::Large(_index) => Err(AllocationError::InvalidLayout),
            Slab::Unsupported => Err(AllocationError::InvalidLayout),
        }
    }

    // Refills the SlabAllocator(include Largepageobject)
    unsafe fn refill_large(
        &mut self,
        layout: Layout,
        new_page: &'a mut LargePageObject<'a>,
    ) -> Result<(), AllocationError> {
        match SlabPool::get_slab(layout.size()) {
            Slab::Base(_index) => Err(AllocationError::InvalidLayout),
            Slab::Large(index) => {
                self.large_slabs[index].refill(new_page);
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

const fn cmin(a: usize, b: usize) -> usize {
    [a, b][(a > b) as usize]
}

pub struct SlabAllocator<'a, P: AllocablePage> {
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

impl<'a, P: AllocablePage> SlabAllocator<'a, P> {
    pub const fn new(size: usize) -> SlabAllocator<'a, P> {
        SlabAllocator {
            size,
            allocation_count: 0,
            per_page_max_obj: cmin(
                (P::SIZE - META_DATA_SIZE) / size,
                BASE_PAGE_SIZE / MIN_BASE_OBJECTS_SIZE,
            ),
            free_slabs: PageList::new(),
            partial_slabs: PageList::new(),
            full_slabs: PageList::new(),
        }
    }

    pub fn allocate(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocationError> {
        let layout = unsafe { Layout::from_size_align_unchecked(self.size, layout.align()) };
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

        NonNull::new(ptr).ok_or(AllocationError::OutOfMemory)
    }

    // Deallocates a previously allocated `ptr` described by `Layout`
    // May return an error in case an invalid `layout` is provided
    pub fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) -> Result<(), AllocationError> {
        let page = (ptr.as_ptr() as usize) & { !(P::SIZE - 1) };
        let slab_page = unsafe { transmute::<usize, &mut P>(page) };
        let layout = unsafe { Layout::from_size_align_unchecked(self.size, layout.align()) };

        slab_page.deallocate(ptr, layout)
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
        page.bitfield_mut()
            .initialize(self.size, P::SIZE - META_DATA_SIZE);
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

impl<'a, P: AllocablePage> Iterator for PageObjectIter<'a, P> {
    type Item = &'a mut P;

    fn next(&mut self) -> Option<&'a mut P> {
        unsafe {
            self.head.resolve_mut().map(|next| {
                self.head = match next.next().resolve_mut() {
                    None => Link::none(),
                    Some(ref mut sp) => Link::some(*sp),
                };
                next
            })
        }
    }
}

#[repr(C)]
pub struct PageObject<'a> {
    // Holds memory objects.
    data: [u8; BASE_PAGE_SIZE - META_DATA_SIZE],
    // Next element in list (used by `PageList`).
    next: Link<PageObject<'a>>,
    prev: Link<PageObject<'a>>,
    // A bit-field to track free/allocated memory within data.
    bitfield: [AtomicU64; 8],
}

unsafe impl<'a> Send for PageObject<'a> {}
unsafe impl<'a> Sync for PageObject<'a> {}

impl<'a> AllocablePage for PageObject<'a> {
    const SIZE: usize = BASE_PAGE_SIZE;

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

#[repr(C)]
pub struct LargePageObject<'a> {
    // Holds memory objects.
    data: [u8; LARGE_PAGE_SIZE - META_DATA_SIZE],
    // Next element in list (used by `PageList`).
    next: Link<LargePageObject<'a>>,
    prev: Link<LargePageObject<'a>>,
    // A bit-field to track free/allocated memory within data.
    bitfield: [AtomicU64; 8],
}

unsafe impl<'a> Send for LargePageObject<'a> {}
unsafe impl<'a> Sync for LargePageObject<'a> {}

impl<'a> AllocablePage for LargePageObject<'a> {
    const SIZE: usize = LARGE_PAGE_SIZE;

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

impl<'a> Default for LargePageObject<'a> {
    fn default() -> LargePageObject<'a> {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

impl<'a> fmt::Debug for LargePageObject<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LargePageObject")
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
        let base_addr = (self as *const Self as *const u8) as usize;
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
    Large(usize),
    Unsupported,
}

// Link
pub struct Link<T> {
    p: *mut T,
}

impl<T> Default for Link<T> {
    fn default() -> Self {
        Link { p: ptr::null_mut() }
    }
}

impl<T> Link<T> {
    // Like Option::None for Link
    fn none() -> Link<T> {
        Link { p: ptr::null_mut() }
    }

    // Like Option::Some for Link
    fn some(n: &mut T) -> Link<T> {
        Link { p: n }
    }

    unsafe fn resolve_mut<'a>(&mut self) -> Option<&'a mut T> {
        self.p.as_mut()
    }
}

// bitfield
trait Bitfield {
    fn initialize(&mut self, object_size: usize, max_size_buffer: usize);
    fn first_fit(
        &self,
        base_addr: usize,
        layout: Layout,
        page_size: usize,
    ) -> Option<(usize, usize)>;
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
    fn first_fit(
        &self,
        base_addr: usize,
        layout: Layout,
        page_size: usize,
    ) -> Option<(usize, usize)> {
        for (base_index, b) in self.iter().enumerate() {
            let bitval = b.load(Ordering::Relaxed);
            if bitval == u64::MAX {
                continue;
            } else {
                let negated = !bitval;
                let first_free = negated.trailing_zeros() as usize;
                let index: usize = base_index * 64 + first_free;
                let offset = index * layout.size();
                let offset_inside_data = offset <= (page_size - META_DATA_SIZE - layout.size());
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

#[cfg(kernel_test)]
pub mod slab_tests {
    use crate::globalallocator_impl::PHYS_MEM_ALLOCATOR;
    use crate::test_framework::TestResult;
    use crate::{print, println};
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::alloc::{GlobalAlloc, Layout};
    use rand::RngCore;

    const MEANINGLESS_NUM: u64 = 0x1234_5678_90ab_cdef; // A meaningless numbers for read and write tests
    const MAX_CHUNK_SIZE: usize = 0x200_0000; // The maximum memory size that the system can allocate at one time

    enum SlabTestResult {
        Ok,
        Err,
    }
    struct SlabTestElem(fn() -> SlabTestResult, String);

    // Print 80 spaces to clear what was printed in this line,
    // then return to the beginning of this line
    fn clean_line() {
        print!("                                                                                ");
        print!("\r");
    }

    pub fn slab_test_main() -> TestResult {
        let tests = [
            // Add your test function here. In the form of:
            // SlabTestElem(your_test_name, String::from("your_test_name")),
            SlabTestElem(alloc_basic_size, String::from("alloc_basic_size")),
            SlabTestElem(alloc_critical_size, String::from("alloc_critical_size")),
            SlabTestElem(alloc_max_size_chunk, String::from("alloc_max_size_chunk")),
            SlabTestElem(alloc_random_size, String::from("alloc_random_size")),
            SlabTestElem(alloc_multiple_times, String::from("alloc_multiple_times")),
        ];

        let mut passed_count = 0;
        let mut failed_count = 0;

        for test in tests {
            println!("[test {}]", test.1);
            match test.0() {
                SlabTestResult::Ok => {
                    clean_line();
                    println!("\x1b[32mpassed\x1b[0m");
                    passed_count += 1;
                }
                SlabTestResult::Err => {
                    clean_line();
                    println!("\x1b[31mfailed\x1b[0m");
                    failed_count += 1;
                }
            }
        }

        TestResult {
            passed: passed_count,
            failed: failed_count,
        }
    }

    fn single_test(layout: Layout) -> SlabTestResult {
        print!("size:{}; align:{};", layout.size(), layout.align());
        unsafe {
            let p = PHYS_MEM_ALLOCATOR.alloc(layout);
            print!(" addr:{:p}", p);
            *(p as *mut u64) = MEANINGLESS_NUM;
            if *(p as *mut u64) != MEANINGLESS_NUM {
                println!("\nError: read or write failed.");
                return SlabTestResult::Err;
            }
            PHYS_MEM_ALLOCATOR.dealloc(p, layout);
        }
        clean_line();
        SlabTestResult::Ok
    }

    #[allow(unused)]
    fn alloc_basic_size() -> SlabTestResult {
        let mut is_passed = true;

        for size in 1..=65 {
            for align_exp in 1..=6 {
                let align = 1 << align_exp;
                if align > size {
                    continue;
                }
                match single_test(Layout::from_size_align(size, align).unwrap()) {
                    SlabTestResult::Err => {
                        is_passed = false;
                    }
                    SlabTestResult::Ok => {}
                }
            }
        }

        if is_passed {
            SlabTestResult::Ok
        } else {
            SlabTestResult::Err
        }
    }

    #[allow(unused)]
    fn alloc_critical_size() -> SlabTestResult {
        let layouts = [
            Layout::from_size_align(127usize, 1usize).unwrap(),
            Layout::from_size_align(128usize, 1usize).unwrap(),
            Layout::from_size_align(128usize, 128usize).unwrap(),
            Layout::from_size_align(129usize, 1usize).unwrap(),
            Layout::from_size_align(255usize, 1usize).unwrap(),
            Layout::from_size_align(256usize, 1usize).unwrap(),
            Layout::from_size_align(256usize, 256usize).unwrap(),
            Layout::from_size_align(127usize, 1usize).unwrap(),
            Layout::from_size_align(511usize, 1usize).unwrap(),
            Layout::from_size_align(512usize, 1usize).unwrap(),
            Layout::from_size_align(512usize, 512usize).unwrap(),
            Layout::from_size_align(513usize, 1usize).unwrap(),
            Layout::from_size_align(1023usize, 1usize).unwrap(),
            Layout::from_size_align(1024usize, 1usize).unwrap(),
            Layout::from_size_align(1024usize, 1024usize).unwrap(),
            Layout::from_size_align(1025usize, 1usize).unwrap(),
            Layout::from_size_align(2047usize, 1usize).unwrap(),
            Layout::from_size_align(2048usize, 1usize).unwrap(),
            Layout::from_size_align(2049usize, 1usize).unwrap(),
            Layout::from_size_align(4095usize, 1usize).unwrap(),
            Layout::from_size_align(4096usize, 1usize).unwrap(),
            Layout::from_size_align(4097usize, 1usize).unwrap(),
            Layout::from_size_align(8191usize, 1usize).unwrap(),
            Layout::from_size_align(8192usize, 1usize).unwrap(),
            Layout::from_size_align(8193usize, 1usize).unwrap(),
            Layout::from_size_align(16383usize, 1usize).unwrap(),
            Layout::from_size_align(16384usize, 1usize).unwrap(),
            Layout::from_size_align(16385usize, 1usize).unwrap(),
        ];
        let mut is_passed = true;

        for layout in layouts.iter() {
            match single_test(layout.clone()) {
                SlabTestResult::Err => {
                    is_passed = false;
                }
                SlabTestResult::Ok => {}
            }
        }

        if is_passed {
            SlabTestResult::Ok
        } else {
            SlabTestResult::Err
        }
    }

    #[allow(unused)]
    fn alloc_max_size_chunk() -> SlabTestResult {
        struct LargeChunk {
            chunk: [i8; MAX_CHUNK_SIZE],
        }
        single_test(Layout::new::<LargeChunk>())
    }

    #[allow(unused)]
    fn alloc_random_size() -> SlabTestResult {
        let mut rng = rand_pcg::Pcg32::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7); // PCG default values
        let mut is_passed = true;

        // Each round tests a random size in the range 1-n 10 times, where n grows
        // from 2^4 and is multiplied by 2 each round until the maximum size is reached.
        let mut shift = 4;
        while MAX_CHUNK_SIZE >= (1 << shift) {
            for _ in 0..=10 {
                match single_test(
                    Layout::from_size_align(((rng.next_u32() as usize) % (1 << shift)) + 1, 1)
                        .unwrap(),
                ) {
                    SlabTestResult::Err => {
                        is_passed = false;
                    }
                    SlabTestResult::Ok => {}
                }
            }
            shift += 1;
        }

        if is_passed {
            SlabTestResult::Ok
        } else {
            SlabTestResult::Err
        }
    }

    #[allow(unused)]
    fn alloc_multiple_times() -> SlabTestResult {
        // The current system can allocate 16384 pages. Since some pages may have
        // been allocated, 16000 pages are allocated here. Each TestChunk contains
        // 16 pages.
        struct TestChunk {
            data: [u8; 16384],
        }
        const MAX_CHUNKS: u32 = 1000;

        let layout = Layout::new::<TestChunk>();

        println!("chunk_size:{}; align:{}", layout.size(), layout.align());
        for _ in 0..=1 {
            let mut v = Vec::new();
            clean_line();
            for n in 0..=MAX_CHUNKS {
                unsafe {
                    let p = PHYS_MEM_ALLOCATOR.alloc(layout);
                    print!("alloc_chunk_count:{};", n);
                    print!(" addr:{:p}", p);
                    if *(p as *mut u64) == MEANINGLESS_NUM {
                        println!("\nError: memory contains previous data.");
                        return SlabTestResult::Err;
                    }
                    *(p as *mut u64) = MEANINGLESS_NUM;
                    if *(p as *mut u64) != MEANINGLESS_NUM {
                        println!("\nError: read or write failed.");
                        return SlabTestResult::Err;
                    }
                    v.push(p);
                    print!("\r");
                }
            }
            clean_line();
            println!("Complete {}-chunk allocation.", MAX_CHUNKS);
            let mut n = 0;
            for p in &v {
                unsafe {
                    print!("dealloc_chunk_count:{};", n);
                    print!(" addr:{:p}", *p);
                    PHYS_MEM_ALLOCATOR.dealloc(*p, layout);
                    print!("\r");
                    n += 1;
                }
            }
            clean_line();
            println!("Complete {}-chunk deallocation.", MAX_CHUNKS);
        }
        SlabTestResult::Ok
    }
}
