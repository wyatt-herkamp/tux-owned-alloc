use super::{AllocErr, LayoutErr, RawVecErr};
use std::{
    alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout},
    fmt,
    marker::PhantomData,
    mem,
    ptr::NonNull,
    slice,
};

pub struct RawVec<T> {
    nnptr: NonNull<T>,
    cap: usize,
    _marker: PhantomData<T>,
}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        Self { nnptr: NonNull::dangling(), cap: 0, _marker: PhantomData }
    }

    pub fn with_capacity(cap: usize) -> Self {
        match Self::try_with_capacity(cap) {
            Ok(this) => this,
            Err(RawVecErr::Alloc(err)) => handle_alloc_error(err.layout),
            Err(RawVecErr::Layout(err)) => {
                panic!("Capacity overflows memory size: {}", err)
            },
        }
    }

    pub fn try_with_capacity(cap: usize) -> Result<Self, RawVecErr> {
        let layout = Self::make_layout(cap)?;
        let res = if layout.size() == 0 {
            Ok(NonNull::dangling())
        } else {
            NonNull::new(unsafe { alloc(layout) })
                .map(NonNull::cast::<T>)
                .ok_or(AllocErr { layout }.into())
        };

        res.map(|nnptr| Self { nnptr, cap, _marker: PhantomData })
    }

    pub fn cap(&self) -> usize {
        self.cap
    }

    pub fn raw(&self) -> NonNull<T> {
        self.nnptr
    }

    pub unsafe fn as_slice(&self) -> &[T] {
        slice::from_raw_parts(self.nnptr.as_ptr(), self.cap())
    }

    pub unsafe fn as_mut_slice(&self) -> &mut [T] {
        slice::from_raw_parts_mut(self.nnptr.as_ptr(), self.cap())
    }

    pub fn realloc(&mut self, new_cap: usize) -> Result<(), RawVecErr> {
        let layout = Self::make_layout(new_cap)?;

        let res = if layout.size() == 0 {
            self.free();
            Ok(NonNull::dangling())
        } else {
            let old = Self::make_layout(self.cap).unwrap();
            NonNull::new(unsafe {
                realloc(self.nnptr.cast().as_ptr(), old, layout.size())
            })
            .map(NonNull::cast::<T>)
            .ok_or(AllocErr { layout }.into())
        };

        res.map(|nnptr| {
            self.nnptr = nnptr;
            self.cap = new_cap;
        })
    }

    fn free(&self) {
        if self.cap != 0 && mem::size_of::<T>() != 0 {
            let layout = Self::make_layout(self.cap).unwrap();
            unsafe {
                dealloc(self.nnptr.cast().as_ptr(), layout);
            }
        }
    }

    fn make_layout(cap: usize) -> Result<Layout, LayoutErr> {
        let total_size =
            mem::size_of::<T>().checked_mul(cap).ok_or(LayoutErr)?;
        Layout::from_size_align(total_size, mem::align_of::<T>())
            .map_err(Into::into)
    }
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

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        self.free();
    }
}
