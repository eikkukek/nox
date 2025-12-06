use ash::vk;

use nox_mem::{vec_types::VecError, slot_map::SlotMapError};

use crate::{
    ResourceError,
    memory_binder::MemoryBinderError,
    BufferError,
    ImageError,
    QueueSubmitError,
};

use nox_error::{
    Error,
    any::AnyError,
};

#[derive(Debug, Error)]
pub enum CommandError {

    #[display("vec error")]
    VecError(#[source] #[from] VecError),

    #[display("slot map error")]
    SlotMapError(#[source] #[from] SlotMapError),

    #[display("vulkan error")]
    VulkanError(#[source] #[from] vk::Result),

    #[display("memory binder error")]
    MemoryBinderError(#[source] #[from] MemoryBinderError),

    #[display("buffer error")]
    BufferError(#[source] #[from] BufferError),

    #[display("image error")]
    ImageError(#[source] #[from] ImageError),

    #[display("invalid host copy with copy size {copy_size} and host buffer size {host_buffer_size}")]
    InvalidHostCopy { copy_size: u64, host_buffer_size: usize, },

    #[display("resource error")]
    ResourceError(#[source] #[from] ResourceError),

    #[display("queue submit error")]
    QueueSubmitError(#[source] #[from] QueueSubmitError),

    #[display("{0}")]
    Other(#[source] #[from] AnyError),
}
