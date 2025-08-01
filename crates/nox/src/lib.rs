#![feature(min_specialization)]
#[macro_use]

pub mod nox;
pub mod renderer;
pub mod interface;
//pub mod asset_importer;

pub mod version;
pub mod utility;
pub mod string_types;
pub mod byte_hash;

mod memory;
pub mod stack_alloc;

pub use nox_mem as mem;

pub use version::Version;
pub use nox::*;
pub use renderer::frame_graph;
pub use memory::Memory;
//pub use asset_importer::AssetType;

impl byte_hash::ByteHasher for blake3::Hasher {

    fn update(&mut self, input: &[u8]) {
        self.update(input);
    }
}
