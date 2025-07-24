#![feature(min_specialization)]
#[macro_use]

pub mod nox;
pub mod renderer;
pub mod interface;
pub mod asset_importer;
pub mod byte_hash;

pub mod version;
pub mod utility;
pub mod serialization;
pub mod string_types;
pub mod marker_types;
//pub mod map_types;
//pub mod shader;

pub mod pod {
    pub use crate::serialization::{Pod, MaybePod, is_pod};
}

mod memory;
pub mod stack_alloc;

pub use nox_mem as mem;

pub use version::Version;
pub use nox::{Nox, InitSettings, AppName};
pub use renderer::frame_graph;
pub use memory::Memory;
pub use asset_importer::AssetType;
