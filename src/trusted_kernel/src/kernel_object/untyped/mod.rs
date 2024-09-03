use crate::capability::object::{KObj, ObjPtr};
use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::ptr::NonNull;

pub struct Untyped {
    pub start: usize,
    pub end: usize,
    pub used: usize,
}

pub trait RetypeInit {
    type StoredAs: CoerceUntypedRegion + ?Sized;
    fn retype_init_in(obj_ptr: ObjPtr<Self::StoredAs>) -> KObj;
}

pub trait CoerceUntypedRegion {
    fn from_untyped_region(region: NonNull<[u8]>) -> NonNull<Self>;
}

impl<T> CoerceUntypedRegion for [T] {
    fn from_untyped_region(region: NonNull<[u8]>) -> NonNull<Self> {
        let region_size = region.len();
        let region_ptr = region.as_ptr();
        let item_size = core::mem::size_of::<T>();
        assert!(region_size % item_size == 0);
        assert!((region_ptr as *const T).is_aligned());
        let num_items = region_size / core::mem::size_of::<T>();
        let coerced_slice_ptr =
            core::ptr::slice_from_raw_parts_mut(region_ptr as *mut T, num_items);
        unsafe {
            // SAFETY: input region is allotted by retype, which satifies the non zero contract
            // alignment and size of coerced slice are checked
            NonNull::new_unchecked(coerced_slice_ptr)
        }
    }
}
impl<T> CoerceUntypedRegion for T {
    fn from_untyped_region(region: NonNull<[u8]>) -> NonNull<Self> {
        region.cast::<T>()
    }
}

impl RetypeInit for Untyped {
    type StoredAs = [u8];
    fn retype_init_in(obj_ptr: ObjPtr<[u8]>) -> KObj {
        let start = obj_ptr.as_ptr() as usize;
        let end = start + obj_ptr.len();
        KObj::Untyped(Untyped::new(start, end))
    }
}

#[derive(Debug)]
pub enum RetypeErr {
    Oom,
}

impl Untyped {
    /// make different retype stragety explicit to enforce compiler check on associated type RetypeInit::StoredAs
    pub(crate) fn retype<T>(&mut self) -> Result<KObj, RetypeErr>
    where
        T: RetypeInit,
        T::StoredAs: Sized,
    {
        let layout = Layout::new::<T::StoredAs>();
        // [u8] slice is well aligned and has size equals to required object size
        let obj_ptr = {
            let coerced_slice_ptr = T::StoredAs::from_untyped_region(self.alloc(layout)?);
            ObjPtr::new(coerced_slice_ptr)
        };
        Ok(T::retype_init_in(obj_ptr))
    }

    pub(crate) fn retype_dyn_sized<T: RetypeInit>(
        &mut self,
        size: usize,
    ) -> Result<KObj, RetypeErr> {
        let layout = Layout::from_size_align(size, size.next_power_of_two()).unwrap();
        let obj_ptr = {
            let coerced_slice_ptr = T::StoredAs::from_untyped_region(self.alloc(layout)?);
            ObjPtr::new(coerced_slice_ptr)
        };
        Ok(T::retype_init_in(obj_ptr))
    }

    fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, RetypeErr> {
        let first_free_block = self.start + self.used;
        let obj_ptr = Self::find_next_aligned(first_free_block, layout.align());

        if obj_ptr + layout.size() > self.end {
            Err(RetypeErr::Oom)
        } else {
            self.used = obj_ptr - self.start + layout.size();
            let obj_slice = core::ptr::slice_from_raw_parts_mut(obj_ptr as *mut u8, layout.size());
            Ok(NonNull::new(obj_slice).unwrap())
        }
    }

    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            used: 0,
        }
    }

    #[inline(always)]
    fn find_next_aligned(addr: usize, align_req: usize) -> usize {
        (addr + align_req - 1) & !(align_req - 1)
    }
}
