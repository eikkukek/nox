#[macro_use]

mod version;
mod attributes;
mod memory;

pub mod error;
mod nox;
mod event;
pub mod gpu;
mod interface;
mod utility;

mod clipboard;

pub mod dev;

mod on_init;
pub use on_init::OnInit;

mod export {

    use super::*;

    pub use nox_mem as mem;
    pub use nox_alloc as alloc;
    pub use nox_log as log;

    pub use version::Version;
    pub use attributes::*;
    pub use memory::Memory;
    pub use event::Event;
    pub use nox::*;
    pub use interface::Interface;
    pub use mem::array_string;
    pub use mem::GlobalAlloc;
}

pub use export::*;

pub use error::{Error, Result};
