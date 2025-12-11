#[macro_use]

pub mod tracked;
mod error;
mod context;

pub use tracked::{Location, Tracked};
pub use error::{Error, BuildInternal};
pub use nox_derive::Error;

pub use core::fmt::Display;
pub use nox_derive::Display;

pub use context::Context;

pub type Result<T> = core::result::Result<T, Error>;
