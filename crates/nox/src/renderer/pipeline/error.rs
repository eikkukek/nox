use ash::vk;

use nox_mem::{
    vec_types::VecError,
    slot_map::SlotMapError,
};

use compact_str::CompactString;

#[derive(Debug)]
pub enum PipelineError {
    VecError(VecError),
    SlotMapError(SlotMapError),
    VulkanError(vk::Result),
    ShaderMismatch(CompactString),
}

impl core::fmt::Display for PipelineError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::VecError(_) => write!(f, "vec error"),
            Self::SlotMapError(_) => write!(f, "slot map error"),
            Self::VulkanError(_) => write!(f, "vulkan error"),
            Self::ShaderMismatch(err) => write!(f, "{err}"),
        }
    }
}

impl core::error::Error for PipelineError {

    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::VecError(err) => Some(err),
            Self::SlotMapError(err) => Some(err),
            Self::VulkanError(err) => Some(err),
            Self::ShaderMismatch(_) => None,
        }
    }
}

impl From<VecError> for PipelineError {

    fn from(value: VecError) -> Self {
        Self::VecError(value)
    }
}

impl From<SlotMapError> for PipelineError {

    fn from(value: SlotMapError) -> Self {
        Self::SlotMapError(value)
    }
}

impl From<vk::Result> for PipelineError {

    fn from(value: vk::Result) -> Self {
        Self::VulkanError(value)
    }
}
