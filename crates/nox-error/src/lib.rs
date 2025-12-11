#[macro_use]

pub mod location;
mod tracked;
mod error;
mod context;

pub use location::Location;
pub use tracked::{Tracked};
pub use error::{Error, BuildInternal};
pub use nox_derive::Error;

pub use core::fmt::Display;
pub use nox_derive::Display;

pub use context::Context;

pub type Result<T> = core::result::Result<T, Error>;
