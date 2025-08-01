#![feature(min_specialization)]
#[macro_use]

pub mod capacity_policy;
pub mod conditional;
pub mod vec_types;
pub mod slot_map;
pub mod const_fn;

mod macros;
mod as_raw;
mod errors;
mod allocator;
mod option_alloc;
mod global_alloc;
mod hashable;
mod mad_cell;

pub use errors::CapacityError;
pub use allocator::Allocator;
pub use global_alloc::{GlobalAlloc, GLOBAL_ALLOC};
pub use option_alloc::OptionAlloc;
pub use capacity_policy::CapacityPolicy;
pub use as_raw::AsRaw;
pub use nox_mem_derive::AsRaw;
pub use hashable::Hashable;
pub use mad_cell::MadCell;
pub use vec_types::Vector;
