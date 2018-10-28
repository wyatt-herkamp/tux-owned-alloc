use super::OwnedAlloc;
use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    mem,
    ptr::NonNull,
};

pub struct UninitAlloc<T> {
    nnptr: NonNull<T>,
}

impl<T> UninitAlloc<T> {
    pub fn new() -> Self {
        Self::try_new()
            .unwrap_or_else(|| handle_alloc_error(Layout::new::<T>()))
    }

    pub fn try_new() -> Option<Self> {
        NonNull::new(unsafe { alloc(Layout::new::<T>()) })
            .map(NonNull::cast::<T>)
            .map(|nnptr| Self { nnptr })
    }

    pub fn init(self, val: T) -> OwnedAlloc<T> {
        let raw = self.into_raw();
        unsafe {
            raw.as_ptr().write(val);
            OwnedAlloc::from_raw(raw)
        }
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

impl<T> Drop for UninitAlloc<T> {
    fn drop(&mut self) {
        unsafe { dealloc(self.nnptr.cast().as_ptr(), Layout::new::<T>()) }
    }
}
