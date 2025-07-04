mod iter;
mod vector;
mod strategies;
mod alloc_vec;
mod array_vec;

pub use vector::Vector;
pub use strategies::{MemoryStrategy, CloneStrategy};
pub use iter::{Iter, IterMut};
pub use alloc_vec::{DynVec, FixedVec, GlobalVec};
pub use array_vec::ArrayVec;

pub type AllocVec<'alloc, T, Alloc, CapacityPol> = alloc_vec::AllocVecImpl<'alloc, T, Alloc, CapacityPol>;
