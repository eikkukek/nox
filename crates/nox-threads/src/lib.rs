//! Provides [`executor`] and [`sync`].
//!
//! Re-exports the [`futures`] crate and [`nox_error`] as [`error`].

pub mod executor;
pub mod sync;

pub use futures;
pub use nox_error as error;
