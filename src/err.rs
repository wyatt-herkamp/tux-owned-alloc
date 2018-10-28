pub use std::alloc::LayoutErr;
use std::{alloc::Layout, fmt};

#[derive(Debug, Clone)]
pub struct AllocErr {
    pub layout: Layout,
}

impl fmt::Display for AllocErr {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmtr,
            "The allocator failed for the layout of size {}, align {}",
            self.layout.size(),
            self.layout.align()
        )
    }
}

#[derive(Debug, Clone)]
pub enum RawVecErr {
    Alloc(AllocErr),
    Layout(LayoutErr),
}

impl fmt::Display for RawVecErr {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RawVecErr::Alloc(err) => write!(fmtr, "{}", err),
            RawVecErr::Layout(err) => write!(fmtr, "{}", err),
        }
    }
}
