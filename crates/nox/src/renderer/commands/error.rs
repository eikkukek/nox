use ash::vk;

use nox_mem::{vec_types::VecError, slot_map::SlotMapError};

use crate::{
    ResourceError,
    memory_binder::MemoryBinderError,
    BufferError,
    ImageError,
    QueueSubmitError,
};

#[derive(Debug)]
pub enum CommandError {
    VecError(VecError),
    SlotMapError(SlotMapError),
    VulkanError(vk::Result),
    MemoryBinderError(MemoryBinderError),
    BufferError(BufferError),
    ImageError(ImageError),
    InvalidHostCopy { copy_size: u64, host_buffer_size: usize, },
    ResourceError(ResourceError),
    QueueSubmitError(QueueSubmitError),
}

impl core::fmt::Display for CommandError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::VecError(_) => write!(f, "vec error"),
            Self::SlotMapError(_) => write!(f, "slot map error"),
            Self::VulkanError(_) => write!(f, "vulkan error"),
            Self::MemoryBinderError(_) => write!(f, "memory binder error"),
            Self::BufferError(_) => write!(f, "buffer error"),
            Self::ImageError(_) => write!(f, "image error"),
            Self::InvalidHostCopy { copy_size, host_buffer_size } =>
                write!(f, "invalid host copy with copy size {copy_size} and host buffer size {host_buffer_size}"),
            Self::ResourceError(_) => write!(f, "resource error"),
            Self::QueueSubmitError(_) => write!(f, "queue submit error"),
        }
    }
}

impl core::error::Error for CommandError {

    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::VecError(err) => Some(err),
            Self::SlotMapError(err) => Some(err),
            Self::VulkanError(err) => Some(err),
            Self::MemoryBinderError(err) => Some(err),
            Self::BufferError(err) => Some(err),
            Self::ImageError(err) => Some(err),
            Self::InvalidHostCopy { copy_size: _, host_buffer_size: _ } => None,
            Self::ResourceError(err) => Some(err),
            Self::QueueSubmitError(err) => Some(err),
        }
    }
}

impl From<VecError> for CommandError {

    fn from(value: VecError) -> Self {
        Self::VecError(value)
    }
}

impl From<SlotMapError> for CommandError {

    fn from(value: SlotMapError) -> Self {
        Self::SlotMapError(value)
    }
}

impl From<vk::Result> for CommandError {

    fn from(value: vk::Result) -> Self {
        Self::VulkanError(value)
    }
}

impl From<MemoryBinderError> for CommandError {

    fn from(value: MemoryBinderError) -> Self {
        Self::MemoryBinderError(value)
    }
}

impl From<BufferError> for CommandError {

    fn from(value: BufferError) -> Self {
        Self::BufferError(value)
    }
}

impl From<ImageError> for CommandError {

    fn from(value: ImageError) -> Self {
        Self::ImageError(value)
    }
}

impl From<ResourceError> for CommandError {

    fn from(value: ResourceError) -> Self {
        Self::ResourceError(value)
    }
}

impl From<QueueSubmitError> for CommandError {

    fn from(value: QueueSubmitError) -> Self {
        Self::QueueSubmitError(value)
    }
}
