use super::{OwnedAlloc, UninitAlloc};
use std::fmt;

pub enum MaybeUninitAlloc<T>
where
    T: ?Sized,
{
    Init(OwnedAlloc<T>),
    Uninit(UninitAlloc<T>),
}

impl<T> MaybeUninitAlloc<T> {
    pub fn or_init<F>(self, init: F) -> OwnedAlloc<T>
    where
        F: FnOnce() -> T,
    {
        match self {
            MaybeUninitAlloc::Init(ptr) => ptr,
            MaybeUninitAlloc::Uninit(ptr) => ptr.init(init()),
        }
    }
}

impl<T> MaybeUninitAlloc<T>
where
    T: ?Sized,
{
    pub fn init_as_ok(self) -> Result<OwnedAlloc<T>, UninitAlloc<T>> {
        match self {
            MaybeUninitAlloc::Init(ptr) => Ok(ptr),
            MaybeUninitAlloc::Uninit(ptr) => Err(ptr),
        }
    }

    pub fn uninit_as_ok(self) -> Result<UninitAlloc<T>, OwnedAlloc<T>> {
        match self {
            MaybeUninitAlloc::Init(ptr) => Err(ptr),
            MaybeUninitAlloc::Uninit(ptr) => Ok(ptr),
        }
    }

    pub unsafe fn or_init_in_place<F>(self, init: F) -> OwnedAlloc<T>
    where
        F: FnOnce(&mut T),
    {
        match self {
            MaybeUninitAlloc::Init(ptr) => ptr,
            MaybeUninitAlloc::Uninit(ptr) => ptr.init_in_place(init),
        }
    }
}

impl<T> From<T> for MaybeUninitAlloc<T> {
    fn from(val: T) -> Self {
        MaybeUninitAlloc::Init(OwnedAlloc::new(val))
    }
}

impl<T> From<OwnedAlloc<T>> for MaybeUninitAlloc<T>
where
    T: ?Sized,
{
    fn from(alloc: OwnedAlloc<T>) -> Self {
        MaybeUninitAlloc::Init(alloc)
    }
}

impl<T> From<UninitAlloc<T>> for MaybeUninitAlloc<T>
where
    T: ?Sized,
{
    fn from(alloc: UninitAlloc<T>) -> Self {
        MaybeUninitAlloc::Uninit(alloc)
    }
}

impl<T> fmt::Debug for MaybeUninitAlloc<T>
where
    T: ?Sized,
{
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaybeUninitAlloc::Init(ptr) => write!(fmtr, "Init({:?})", ptr),
            MaybeUninitAlloc::Uninit(ptr) => write!(fmtr, "Uninit({:?})", ptr),
        }
    }
}
