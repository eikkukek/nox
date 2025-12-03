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

#[derive(Debug)]
pub enum ResourceError {
    VecError(VecError),
    SlotMapError(SlotMapError),
    VulkanError(vk::Result),
    ShaderError(ShaderError),
    PipelineError(PipelineError),
    MemoryBinderError(MemoryBinderError),
    BufferError(BufferError),
    ImageError(ImageError),
    DescriptorPoolFull { max_sets: u32, },
    ResourceLocked,
    Other(Box<dyn core::error::Error>),
}

impl ResourceError {

    pub fn other(err: impl core::error::Error + 'static) -> Self {
        Self::Other(Box::new(err))
    }
}

impl core::fmt::Display for ResourceError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::VecError(_) => write!(f, "vec error"),
            Self::SlotMapError(_) => write!(f, "slot map error"),
            Self::VulkanError(_) => write!(f, "vulkan error"),
            Self::ShaderError(_) => write!(f, "shader error"),
            Self::PipelineError(_) => write!(f, "pipeline error"),
            Self::MemoryBinderError(_) => write!(f, "memory binder error"),
            Self::BufferError(_) => write!(f, "buffer error"),
            Self::ImageError(_) => write!(f, "image error"),
            Self::DescriptorPoolFull { max_sets, } => write!(f, "descriptor pool is full with max set count {max_sets}"),
            Self::ResourceLocked => write!(f, "resource locked"),
            Self::Other(_) => write!(f, "other error"),
        }
    }
}

impl core::error::Error for ResourceError {

    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::VecError(err) => Some(err),
            Self::SlotMapError(err) => Some(err),
            Self::VulkanError(err) => Some(err),
            Self::ShaderError(err) => Some(err),
            Self::PipelineError(err) => Some(err),
            Self::MemoryBinderError(err) => Some(err),
            Self::BufferError(err) => Some(err),
            Self::ImageError(err) => Some(err),
            Self::DescriptorPoolFull { max_sets: _ } => None,
            Self::ResourceLocked => None,
            Self::Other(err) => Some(&**err),
        }
    }
}

impl From<VecError> for ResourceError {

    fn from(value: VecError) -> Self {
        Self::VecError(value)
    }
}

impl From<SlotMapError> for ResourceError {

    fn from(value: SlotMapError) -> Self {
        Self::SlotMapError(value)
    }
}

impl From<vk::Result> for ResourceError {

    fn from(value: vk::Result) -> Self {
        Self::VulkanError(value)
    }
}

impl From<ShaderError> for ResourceError {

    fn from(value: ShaderError) -> Self {
        Self::ShaderError(value)
    }
}

impl From<PipelineError> for ResourceError {

    fn from(value: PipelineError) -> Self {
        Self::PipelineError(value)
    }
}

impl From<MemoryBinderError> for ResourceError {

    fn from(value: MemoryBinderError) -> Self {
        Self::MemoryBinderError(value)
    }
}

impl From<BufferError> for ResourceError {

    fn from(value: BufferError) -> Self {
        Self::BufferError(value)
    }
}

impl From<ImageError> for ResourceError {

    fn from(value: ImageError) -> Self {
        Self::ImageError(value)
    }
}
