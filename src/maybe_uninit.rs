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

    pub fn modify<F, A>(&mut self, visit: F) -> Option<A>
    where
        F: FnOnce(&mut T) -> A,
    {
        match self {
            MaybeUninitAlloc::Init(ptr) => Some(visit(&mut **ptr)),
            MaybeUninitAlloc::Uninit(_) => None,
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

#[cfg(test)]
mod test {
    use super::{super::UninitAlloc, MaybeUninitAlloc};

    #[test]
    fn or_init_is_noop_if_initialized() {
        let init = MaybeUninitAlloc::from(90);

        assert_eq!(*init.or_init(|| 50), 90);
    }

    #[test]
    fn or_init_calls_if_uninit() {
        let init = MaybeUninitAlloc::from(UninitAlloc::new());

        assert_eq!(*init.or_init(|| 50), 50);
    }

    #[test]
    fn modifies() {
        let mut init = MaybeUninitAlloc::from(20);

        assert!(init.modify(|addr| *addr = 2).is_some());
        assert_eq!(*init.init_as_ok().unwrap(), 2);
    }
}
