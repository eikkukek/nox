#[macro_use]

mod nox;
mod renderer;
mod interface;

pub mod version;
pub mod utility;

mod memory;
mod errors;
mod init_settings;
mod clipboard;

pub use nox_mem as mem;
pub use nox_alloc as alloc;
pub use nox_log as log;

use errors::InitError;

pub use nox_error as error;

pub use errors::Error;
pub use nox_derive::Error;
pub use version::Version;
pub use nox::*;
pub use init_settings::*;
pub use renderer::*;
pub use memory::Memory;
pub use interface::Interface;
pub use mem::array_string;
pub use mem::GlobalAlloc;
