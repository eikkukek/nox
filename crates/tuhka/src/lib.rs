#![no_std]

pub mod vk;
mod option;
mod library;
mod instance;
mod device;
mod result;
mod core_gen;
mod extension_gen;

pub(crate) use option::PtrOption;
pub use result::*;
pub use core_gen::*;

pub use library::*;
pub use instance::*;
pub use device::*;
pub use extension_gen::*;
mod macros;
