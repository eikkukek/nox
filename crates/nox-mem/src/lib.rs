#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[macro_use]
pub mod conditional;
pub mod collections;
pub mod num;
pub mod slice;
pub mod vec;
pub mod string;
pub mod slot_map;
pub mod dynamic;
pub mod cell;
pub mod option;
#[cfg(feature = "std")]
pub mod iter;
#[cfg(feature = "std")]
pub mod bit;
pub mod alloc;
pub mod borrow;

mod macros;
mod as_raw;
mod hashable;
mod const_fn;
mod plain;

pub use as_raw::AsRaw;
pub use hashable::Hashable;
pub use const_fn::*;
pub use plain::Plain;
pub use nox_proc::Display;

pub use paste::paste;
