use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    marker::PhantomData,
    ptr::NonNull,
};

pub struct OwnedAlloc<T, M>
where
    M: LayoutMolder<T>,
{
    nnptr: NonNull<T>,
    molder: M,
    _marker: PhantomData<T>,
}

impl<T, M> OwnedAlloc<T, M>
where
    M: LayoutMolder<T>,
{
    pub fn new<U>(init: U) -> Self
    where
        M: MolderInit<T, U>,
    {
        Self::try_new(init).unwrap_or_else(|e| handle_alloc_error(e))
    }

    pub fn try_new<U>(init: U) -> Result<Self, Layout>
    where
        M: MolderInit<T, U>,
    {
        let molder = M::create(&init);
        let layout = molder.mold_new();

        let res = if layout.size() == 0 {
            Ok(NonNull::dangling())
        } else {
            NonNull::new(unsafe { alloc(layout) })
                .ok_or(layout)
                .map(NonNull::cast::<T>)
                .map(|nnptr| unsafe {
                    molder.init(nnptr, init);
                    nnptr
                })
        };

        res.map(|nnptr| Self { nnptr, molder, _marker: PhantomData })
    }
}

impl<T, M> Drop for OwnedAlloc<T, M>
where
    M: LayoutMolder<T>,
{
    fn drop(&mut self) {
        let layout = unsafe { self.molder.mold_for(self.nnptr) };
        if layout.size() != 0 {
            unsafe {
                self.molder.deinit(self.nnptr);
                dealloc(self.nnptr.cast().as_ptr(), layout);
            }
        }
    }
}

pub unsafe trait MolderInit<T, U>: LayoutMolder<T> {
    fn create(arg: &U) -> Self;

    unsafe fn init(&self, ptr: NonNull<T>, arg: U);
}

pub unsafe trait LayoutMolder<T> {
    fn mold_new(&self) -> Layout;

    unsafe fn mold_for(&self, _ptr: NonNull<T>) -> Layout {
        self.mold_new()
    }

    unsafe fn deinit(&self, ptr: NonNull<T>);
}
