use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    marker::PhantomData,
    ptr::NonNull,
};

pub struct OwnedAlloc<T, L>
where
    L: LayoutTracker<T>,
{
    nnptr: NonNull<T>,
    tracker: L,
    _marker: PhantomData<T>,
}

impl<T, L> OwnedAlloc<T, L>
where
    L: LayoutTracker<T>,
{
    pub fn new(tracker: L) -> Self {
        Self::try_new(tracker).unwrap_or_else(|e| handle_alloc_error(e))
    }

    pub fn try_new(tracker: L) -> Result<Self, Layout> {
        let layout = tracker.new_layout();

        let opt_nnptr = if layout.size() == 0 {
            Some(NonNull::dangling())
        } else {
            unsafe {
                NonNull::new(alloc(layout)).map(NonNull::cast::<T>).map(
                    |nnptr| {
                        tracker.init_alloc(nnptr);
                        nnptr
                    },
                )
            }
        };

        opt_nnptr
            .map(|nnptr| Self { nnptr, tracker, _marker: PhantomData })
            .ok_or(layout)
    }
}

impl<T, L> Drop for OwnedAlloc<T, L>
where
    L: LayoutTracker<T>,
{
    fn drop(&mut self) {
        let layout = unsafe { self.tracker.layout_of(self.nnptr) };
        if layout.size() != 0 {
            unsafe {
                self.tracker.deinit_alloc(self.nnptr);
                dealloc(self.nnptr.cast().as_ptr(), layout)
            }
        }
    }
}

pub unsafe trait LayoutTracker<T> {
    fn new_layout(&self) -> Layout;

    unsafe fn layout_of(&self, ptr: NonNull<T>) -> Layout;

    unsafe fn init_alloc(&self, ptr: NonNull<T>);

    unsafe fn deinit_alloc(&self, ptr: NonNull<T>);
}
