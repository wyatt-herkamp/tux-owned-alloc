use super::UninitAlloc;
use std::{
    alloc::{dealloc, Layout},
    fmt,
    mem,
    ops::{Deref, DerefMut},
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

    pub fn move_inner(self) -> (T, UninitAlloc<T>) {
        let val = unsafe { self.nnptr.as_ptr().read() };
        let alloc = unsafe { UninitAlloc::from_raw(self.nnptr) };
        mem::forget(self);
        (val, alloc)
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

impl<T> Drop for OwnedAlloc<T> {
    fn drop(&mut self) {
        unsafe {
            self.nnptr.as_ptr().drop_in_place();
            dealloc(self.nnptr.cast().as_ptr(), Layout::new::<T>());
        }
    }
}

impl<T> Deref for OwnedAlloc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.nnptr.as_ref() }
    }
}

impl<T> DerefMut for OwnedAlloc<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.nnptr.as_mut() }
    }
}

impl<T> fmt::Debug for OwnedAlloc<T> {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{:?}", self.nnptr)
    }
}
