mod error;
mod vector;
mod pointer;
mod alloc_vec;
mod array_vec;
mod phantom_vec;

pub use error::VecError;
pub use vector::Vector;
pub use pointer::Pointer;
pub use alloc_vec::{DynVec, FixedVec, GlobalVec};
pub use array_vec::ArrayVec;
pub use phantom_vec::PhantomVec;

pub type Result<T> = core::result::Result<T, VecError>;

pub type AllocVec<T, Alloc, CapacityPol> = alloc_vec::AllocVecImpl<T, Alloc, CapacityPol>;
