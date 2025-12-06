use ash::vk;

use nox_mem::{
    vec_types::VecError,
    slot_map::SlotMapError,
};

use crate::{
    ShaderError,
    PipelineError,
    memory_binder::MemoryBinderError,
    BufferError,
    ImageError,
};

use nox_error::{Error, any::AnyError};

#[derive(Error, Debug)]
pub enum ResourceError {

    #[display("vec error")]
    VecError(#[source] #[from] VecError),

    #[display("slot map error")]
    SlotMapError(#[source] #[from] SlotMapError),

    #[display("vulkan error")]
    VulkanError(#[source] #[from] vk::Result),

    #[display("shader error")]
    ShaderError(#[source] #[from] ShaderError),

    #[display("pipeline error")]
    PipelineError(#[source] #[from] PipelineError),

    #[display("memory binder error")]
    MemoryBinderError(#[source] #[from] MemoryBinderError),

    #[display("buffer error")]
    BufferError(#[source] #[from] BufferError),

    #[display("image error")]
    ImageError(#[source] #[from] ImageError),

    #[display("descriptor pool full (max sets: {max_sets})")]
    DescriptorPoolFull { max_sets: u32, },

    #[display("resource locked")]
    ResourceLocked,

    #[display("invalid shader resource binding {binding} for set {set}")]
    InvalidShaderResourceImageBinding {
        binding: u32,
        set: u32,
    },

    #[display("{0}")]
    Other(#[source] #[from] AnyError),
}
