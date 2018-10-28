use super::{AllocErr, OwnedAlloc, UninitAlloc};
use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    fmt,
    marker::PhantomData,
    mem,
    ptr::NonNull,
};

pub enum MaybeUninitAlloc<T>
where
    T: ?Sized,
{
    Init(OwnedAlloc<T>),
    Uninit(UninitAlloc<T>),
}
