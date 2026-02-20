#[macro_use]
mod version;
mod attributes;

pub mod error;
pub mod sync;
mod nox;
pub mod gpu;
mod interface;

pub use gpu::ash as nox_ash;

mod clipboard;

pub mod dev;

mod on_init;
pub use on_init::OnInit;

pub mod misc;

mod prelude {

    use super::*;

    pub use nox_mem as mem;
    pub use nox_alloc as alloc;
    pub use nox_log as log;
    pub use nox_threads as threads;

    pub use version::Version;
    pub use attributes::*;
    pub use nox::*;
    pub use interface::Interface;
    pub use mem::array_string;
}

pub use prelude::*;
pub use error::{Error, Result};
