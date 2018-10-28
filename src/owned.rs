use super::UninitAlloc;
use std::{
    alloc::{dealloc, Layout},
    mem,
    ptr::NonNull,
};

pub struct OwnedAlloc<T> {
    nnptr: NonNull<T>,
}

impl<T> OwnedAlloc<T> {
    pub fn new(val: T) -> Self {
        UninitAlloc::new().init(val)
    }

    pub fn try_new(val: T) -> Option<Self> {
        UninitAlloc::try_new().map(|alloc| alloc.init(val))
    }

    pub fn into_raw(self) -> NonNull<T> {
        let nnptr = self.nnptr;
        mem::forget(self);
        nnptr
    }

    pub unsafe fn from_raw(nnptr: NonNull<T>) -> Self {
        Self { nnptr }
    }
}

impl<T> Drop for OwnedAlloc<T> {
    fn drop(&mut self) {
        unsafe {
            self.nnptr.as_ptr().drop_in_place();
            dealloc(self.nnptr.cast().as_ptr(), Layout::new::<T>());
        }
    }
}
