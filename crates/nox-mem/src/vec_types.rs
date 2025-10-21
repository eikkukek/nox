mod vector;
mod pointer;
mod alloc_vec;
mod array_vec;
mod ghost_vec;

pub use vector::Vector;
pub use pointer::Pointer;
pub use alloc_vec::{DynVec, FixedVec, GlobalVec};
pub use array_vec::ArrayVec;
pub use ghost_vec::GhostVec;

pub type AllocVec<T, Alloc, CapacityPol> = alloc_vec::AllocVecImpl<T, Alloc, CapacityPol>;
