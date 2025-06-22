#![feature(min_specialization)]
#[macro_use]

pub mod triv;
pub mod capacity_policy;
pub mod vec_types;
pub mod ser;
pub mod slot_alloc;

mod macros;
mod const_fn;
mod errors;
mod allocator;
mod global_alloc;

pub use errors::CapacityError;
pub use allocator::Allocator;
pub use slot_alloc::SlotAlloc;
