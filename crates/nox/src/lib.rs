//#![warn(missing_docs)]

mod version;
mod attributes;
mod nox;

pub mod error;
pub mod sync;
pub mod gpu;

mod clipboard;

mod globals;
pub use globals::Globals;

mod misc;

pub use nox_mem as mem;
pub use nox_alloc as alloc;
pub use nox_log as log;
pub use nox_threads as threads;
pub use nox_ash as ash;

pub use version::Version;
pub use attributes::*;
pub use nox::*;
pub use mem::array_string;

pub use error::{Error, Result, EventError, EventResult};
