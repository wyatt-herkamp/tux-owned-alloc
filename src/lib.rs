mod uninit;
mod owned;

pub use self::{owned::OwnedAlloc, uninit::UninitAlloc};
