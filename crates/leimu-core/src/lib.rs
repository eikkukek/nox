#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod option;
mod result;
mod iter;
pub mod hash;
pub mod slice;
pub mod borrow;
#[cfg(feature = "std")]
pub mod collections;

pub use option::OptionExt;
pub use result::ResultExt;
pub use iter::TryExtend;
