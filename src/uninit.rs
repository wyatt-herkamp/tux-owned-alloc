use super::{AllocErr, OwnedAlloc, RawVec};
use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    fmt,
    marker::PhantomData,
    mem,
    ptr::NonNull,
};

pub struct UninitAlloc<T>
where
    T: ?Sized,
{
    nnptr: NonNull<T>,
    _marker: PhantomData<T>,
}

impl<T> UninitAlloc<T> {
    pub fn new() -> Self {
        Self::try_new().unwrap_or_else(|err| handle_alloc_error(err.layout))
    }

    pub fn try_new() -> Result<Self, AllocErr> {
        let layout = Layout::new::<T>();

        let res = if layout.size() == 0 {
            Ok(NonNull::dangling())
        } else {
            NonNull::new(unsafe { alloc(layout) })
                .map(NonNull::cast::<T>)
                .ok_or(AllocErr { layout })
        };

        res.map(|nnptr| Self { nnptr, _marker: PhantomData })
    }

    pub fn init(self, val: T) -> OwnedAlloc<T> {
        let raw = self.into_raw();
        unsafe {
            raw.as_ptr().write(val);
            OwnedAlloc::from_raw(raw)
        }
    }
}

impl<T> UninitAlloc<T>
where
    T: ?Sized,
{
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
        Self { nnptr, _marker: PhantomData }
    }
}

impl<T> Drop for UninitAlloc<T>
where
    T: ?Sized,
{
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::for_value(self.nnptr.as_ref());

            if layout.size() != 0 {
                dealloc(self.nnptr.cast().as_ptr(), layout);
            }
        }
    }
}

impl<T> fmt::Debug for UninitAlloc<T>
where
    T: ?Sized,
{
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{:?}", self.nnptr)
    }
}

impl<T> From<RawVec<T>> for UninitAlloc<[T]> {
    fn from(alloc: RawVec<T>) -> Self {
        Self { nnptr: alloc.into_raw_slice(), _marker: PhantomData }
    }
}
