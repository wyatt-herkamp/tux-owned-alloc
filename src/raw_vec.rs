use std::{
    alloc::{dealloc, Layout},
    fmt,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub struct RawVec<T> {
    nnptr: NonNull<T>,
    cap: usize,
    _marker: PhantomData<T>,
}

impl<T> fmt::Debug for RawVec<T> {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmtr,
            "RawVec {} pointer {:?}, cap: {} {}",
            '{', self.nnptr, self.cap, '}'
        )
    }
}
