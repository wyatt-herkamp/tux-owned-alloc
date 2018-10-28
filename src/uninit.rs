use super::OwnedAlloc;
use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    fmt,
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

    pub unsafe fn init_in_place<F>(self, init: F) -> OwnedAlloc<T>
    where
        F: FnOnce(&mut T),
    {
        let mut raw = self.into_raw();
        init(raw.as_mut());
        OwnedAlloc::from_raw(raw)
    }

    pub fn raw(&self) -> NonNull<T> {
        self.nnptr
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

impl<T> fmt::Debug for UninitAlloc<T> {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{:?}", self.nnptr)
    }
}
