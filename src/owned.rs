use super::{AllocErr, UninitAlloc};
use std::{
    alloc::{dealloc, Layout},
    fmt,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub struct OwnedAlloc<T>
where
    T: ?Sized,
{
    nnptr: NonNull<T>,
    _marker: PhantomData<T>,
}

impl<T> OwnedAlloc<T> {
    pub fn new(val: T) -> Self {
        UninitAlloc::new().init(val)
    }

    pub fn try_new(val: T) -> Result<Self, AllocErr> {
        UninitAlloc::try_new().map(|alloc| alloc.init(val))
    }

    pub fn move_inner(self) -> (T, UninitAlloc<T>) {
        let val = unsafe { self.nnptr.as_ptr().read() };
        let alloc = unsafe { UninitAlloc::from_raw(self.nnptr) };
        mem::forget(self);
        (val, alloc)
    }
}

impl<T> OwnedAlloc<T>
where
    T: ?Sized,
{
    pub fn drop_in_place(self) -> UninitAlloc<T> {
        let alloc = unsafe {
            self.nnptr.as_ptr().drop_in_place();
            UninitAlloc::from_raw(self.nnptr)
        };
        mem::forget(self);
        alloc
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

impl<T> Drop for OwnedAlloc<T>
where
    T: ?Sized,
{
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::for_value(self.nnptr.as_ref());
            self.nnptr.as_ptr().drop_in_place();
            if layout.size() != 0 {
                dealloc(self.nnptr.cast().as_ptr(), layout);
            }
        }
    }
}

impl<T> Deref for OwnedAlloc<T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.nnptr.as_ref() }
    }
}

impl<T> DerefMut for OwnedAlloc<T>
where
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.nnptr.as_mut() }
    }
}

impl<T> fmt::Debug for OwnedAlloc<T>
where
    T: ?Sized,
{
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{:?}", self.nnptr)
    }
}

impl<T> Clone for OwnedAlloc<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self::new((**self).clone())
    }
}

impl<T> From<T> for OwnedAlloc<T> {
    fn from(val: T) -> Self {
        Self::new(val)
    }
}

#[cfg(test)]
mod test {
    use super::OwnedAlloc;

    #[test]
    fn inner_eq() {
        let mut alloc = OwnedAlloc::new(20);

        assert_eq!(*alloc, 20);

        *alloc = 30;

        assert_eq!(*alloc, 30);
    }

    #[test]
    fn move_inner_eq() {
        let alloc = OwnedAlloc::new(20);

        assert_eq!(alloc.move_inner().0, 20);
    }
}
