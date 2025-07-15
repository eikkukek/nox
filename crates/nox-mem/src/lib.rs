#![feature(min_specialization)]
#[macro_use]

pub mod triv;
pub mod capacity_policy;
pub mod conditional;
pub mod vec_types;
//pub mod ser;
pub mod type_registery;
//pub mod slot_alloc;
pub mod const_fn;

mod macros;
mod as_raw;
mod errors;
mod allocator;
mod option_alloc;
mod global_alloc;

pub use errors::CapacityError;
pub use allocator::Allocator;
pub use global_alloc::{GlobalAlloc, GLOBAL_ALLOC};
pub use option_alloc::OptionAlloc;
//pub use slot_alloc::SlotAlloc;
pub use capacity_policy::CapacityPolicy;
pub use vec_types::{Vector, AllocVec, GlobalVec, DynVec, FixedVec, ArrayVec};
pub use as_raw::AsRaw;
pub use nox_mem_derive::AsRaw;
