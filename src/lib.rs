#![feature(min_specialization)]

pub mod nox;
pub mod renderer;

pub mod version;
pub mod utility;
pub mod string_types;
pub mod marker_types;
pub mod vec_types;
pub mod map_types;
pub mod interface;
pub mod shader;

mod backend;
mod allocator_traits;
mod stack_alloc;
mod dyn_alloc;
mod global_alloc;

pub use version::Version;
pub use nox::{Nox, InitSettings, AppName, Memory};
pub use renderer::{Renderer, DeviceName};
