#[macro_use]
pub mod location;
mod tracked;
mod error;
mod context;

pub use location::Location;
pub use tracked::{Tracked};
pub use error::{Error, build_error};
pub use nox_proc::Error;

pub use core::fmt::Display;
pub use nox_proc::Display;

pub use context::Context;

/// Type definition [`Result`][core::result::Result] with [`Error`] as the [`Err`] type.
pub type Result<T> = core::result::Result<T, Error>;
