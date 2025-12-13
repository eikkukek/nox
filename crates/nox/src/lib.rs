#[macro_use]

mod version;
mod init_settings;
mod memory;

pub mod error;
mod nox;
mod event;
pub mod gpu;
mod interface;
mod utility;

mod clipboard;

pub mod dev;

mod export {

    use super::*;

    pub use nox_mem as mem;
    pub use nox_alloc as alloc;
    pub use nox_log as log;

    pub use version::Version;
    pub use init_settings::InitSettings;
    pub use memory::Memory;
    pub use event::Event;
    pub use nox::*;
    pub use interface::{Initialize, ProcessEvent};
    pub use mem::array_string;
    pub use mem::GlobalAlloc;
    pub use mem::cell::{InitCell, CellToken};
}

pub use export::*;

pub use error::Error;
pub type Result<T> = core::result::Result<T, Error>;

pub use mem::singleton_cell_token;
