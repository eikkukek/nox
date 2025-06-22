#![feature(min_specialization)]

pub mod nox;
pub mod renderer;
pub mod interface;
pub mod asset_importer;

pub mod version;
pub mod utility;
pub mod serialization;
pub mod string_types;
pub mod marker_types;
pub mod vec_types;
pub mod map_types;
//pub mod shader;

pub mod pod {
    pub use crate::serialization::{Pod, MaybePod, is_pod};
}

mod memory;
mod allocator_traits;
mod stack_alloc;
mod dyn_alloc;
mod global_alloc;

pub use version::Version;
pub use nox::{Nox, InitSettings, AppName};
pub use renderer::{Renderer, DeviceName, DeviceMemory};
pub use memory::Memory;
pub use asset_importer::AssetType;
pub use global_alloc::GlobalAlloc;
