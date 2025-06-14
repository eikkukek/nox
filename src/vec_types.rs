mod traits;
mod iter;
mod alloc_vec;
mod array_vec;

pub use traits::{
    CapacityError, CapacityPolicy,
    MemoryStrategy, DuplicateStrategy,
    Dyn, Fixed,
    Vector,
};
pub use iter::{Iter, IterMut};
pub use alloc_vec::AllocVec;
pub use array_vec::ArrayVec;

pub type DynVec<'alloc, T, Alloc> = AllocVec<'alloc, T, Alloc, traits::Dyn>;
pub type FixedVec<'alloc, T, Alloc> = AllocVec<'alloc, T, Alloc, traits::Fixed>;
