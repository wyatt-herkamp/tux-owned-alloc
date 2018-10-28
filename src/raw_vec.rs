use super::{AllocErr, LayoutErr, RawVecErr, UninitAlloc};
use std::{
    alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout},
    fmt,
    marker::PhantomData,
    mem,
    ptr::NonNull,
    slice,
};

/// Raw Vector allocation. This allocation, instead of holding a pointer to a
/// single `T`, holds a pointer to as many `T` are required. The allocation is
/// resizable and is freed on `drop`. No initialization or deinitialization of
/// the elements is performed. This type may be useful for `Vec`-like types.
pub struct RawVec<T> {
    nnptr: NonNull<T>,
    cap: usize,
    _marker: PhantomData<T>,
}

impl<T> RawVec<T> {
    /// Creates a new `RawVec` of capacity `0` and a dangling pointer. No
    /// allocation is performed.
    pub fn new() -> Self {
        Self { nnptr: NonNull::dangling(), cap: 0, _marker: PhantomData }
    }

    /// Creates a new `RawVec` with a given capacity. In case of allocation
    /// error, the handler registered via stdlib is called. In case of overflow
    /// calculating the total size, the function panics.
    pub fn with_capacity(cap: usize) -> Self {
        match Self::try_with_capacity(cap) {
            Ok(this) => this,
            Err(RawVecErr::Alloc(err)) => handle_alloc_error(err.layout),
            Err(RawVecErr::Layout(err)) => {
                panic!("Capacity overflows memory size: {}", err)
            },
        }
    }

    /// Creates a new `RawVec` with a given capacity. In case of allocation
    /// error or overflow calculating the total size, `Err` is returned.
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

    /// The requested capacity. It is guaranteed to be the capacity passed to
    /// the last capacity-modifier method. Those are `with_capacity`,
    /// `try_with_capacity` and `resize`. The methods `new` and `try_new`
    /// initialize the capacity to `0`.
    pub fn cap(&self) -> usize {
        self.cap
    }

    /// The raw non-null pointer to the first element.
    pub fn raw(&self) -> NonNull<T> {
        self.nnptr
    }

    /// The raw non-null pointer to the slice with length equal to the
    /// `RawVec`'s capacity.
    pub fn raw_slice(&self) -> NonNull<[T]> {
        unsafe { NonNull::from(self.as_slice()) }
    }

    /// "Forgets" dropping the allocation and returns a raw non-null pointer to
    /// the slice with length equal to the `RawVec`'s capacity.
    pub fn into_raw_slice(self) -> NonNull<[T]> {
        let ptr = self.raw_slice();
        mem::forget(self);
        ptr
    }

    /// Recreate the `RawVec` from a raw non-null pointer and a capacity.
    ///
    /// # Safety
    /// This functions is `unsafe` because passing the wrong pointer leads to
    /// undefined behaviour. Passing wrong capacity also leads to undefined
    /// behaviour.
    pub unsafe fn from_raw_parts(nnptr: NonNull<T>, cap: usize) -> Self {
        Self { nnptr, cap, _marker: PhantomData }
    }

    /// Recreate the `RawVec` from a raw non-null pointer to a slice with length
    /// equal to the `RawVec`'s capacity.
    ///
    /// # Safety
    /// This functions is `unsafe` because passing the wrong pointer leads to
    /// undefined behaviour, including passing a pointer with the wrong length.
    pub unsafe fn from_raw_slice(mut raw: NonNull<[T]>) -> Self {
        Self {
            nnptr: NonNull::new_unchecked(raw.as_mut().as_mut_ptr()),
            cap: raw.as_ref().len(),
            _marker: PhantomData,
        }
    }

    /// Encodes the `RawVec` as an immutable reference to a slice with length
    /// equal to the capacity.
    ///
    /// # Safety
    /// This function is `unsafe` because if the index of an uninitialized
    /// element is accessed incorrectly, undefined behavior occurs.
    pub unsafe fn as_slice(&self) -> &[T] {
        slice::from_raw_parts(self.nnptr.as_ptr(), self.cap())
    }

    /// Encodes the `RawVec` as an mutable reference to a slice with length
    /// equal to the capacity.
    ///
    /// # Safety
    /// This function is `unsafe` because if the index of an uninitialized
    /// element is accessed incorrectly, undefined behavior occurs.
    pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        slice::from_raw_parts_mut(self.nnptr.as_ptr(), self.cap())
    }

    /// Resizes the `RawVec` with a given capacity. In case of allocation
    /// error, the handler registered via stdlib is called. In case of overflow
    /// calculating the total size, the function panics.
    pub fn resize(&mut self, new_cap: usize) {
        match self.try_resize(new_cap) {
            Err(RawVecErr::Alloc(err)) => handle_alloc_error(err.layout),
            Err(RawVecErr::Layout(err)) => {
                panic!("Capacity overflows memory size: {}", err)
            },

            Ok(_) => (),
        }
    }

    /// Resizes the `RawVec` with a given capacity. In case of allocation
    /// error or overflow calculating the total size, `Err` is returned. In case
    /// of failure, the original allocation is untouched.
    pub fn try_resize(&mut self, new_cap: usize) -> Result<(), RawVecErr> {
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

impl<T> From<UninitAlloc<T>> for RawVec<T> {
    fn from(alloc: UninitAlloc<T>) -> Self {
        Self { nnptr: alloc.into_raw(), cap: 1, _marker: PhantomData }
    }
}

unsafe impl<T> Send for UninitAlloc<T> where T: Send {}
unsafe impl<T> Sync for UninitAlloc<T> where T: Sync {}

#[cfg(test)]
mod test {
    use super::RawVec;

    #[test]
    fn cap_is_the_one_passed() {
        let alloc = RawVec::<usize>::with_capacity(20);

        assert_eq!(alloc.cap(), 20);
    }
}
