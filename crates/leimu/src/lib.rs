//#![warn(missing_docs)]

pub mod error;
pub mod sync;
pub mod gpu;

mod macros;

pub use nox_mem as mem;
pub use nox_log as log;
pub use nox_threads as threads;
pub use nox_ash as ash;

#[cfg(feature = "event-loop")]
mod nox;

#[cfg(feature = "event-loop")]
pub use nox::*;

pub use error::{Error, Result, EventError, EventResult};
